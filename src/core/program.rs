use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
	process::{Command as StdCommand, Stdio},
	sync::{Arc, RwLock},
	thread,
	time::Duration,
};

use anyhow::Result;
use colored::Colorize;
use config_derive::{Get, Iter, Set, Val};
use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::{
	command::Command,
	dirs,
	ext::{PathExt, ResultExt},
	logger, racky_error, racky_info, racky_warn, rlock, util, wlock,
};

pub type ProgramPtr = Arc<Program>;

#[derive(Debug, Default)]
pub struct Program {
	name: String,
	paths: Paths,
	state: RwLock<State>,
}

impl Program {
	pub fn new(name: &str) -> ProgramPtr {
		Arc::new(Self {
			name: name.to_owned(),
			paths: Paths::from_name(name),
			..Default::default()
		})
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn paths(&self) -> &Paths {
		&self.paths
	}

	pub fn config(&self) -> Config {
		rlock!(self.state).config.clone()
	}

	pub fn status(&self) -> Status {
		rlock!(self.state).status.clone()
	}

	pub fn is_active(&self) -> bool {
		matches!(self.status(), Status::Running(_) | Status::Restarting)
	}

	pub fn load_config(self: &ProgramPtr) {
		if !self.paths.config.exists() {
			warn!("Config of {} not found", self.name);
			return;
		}

		let contents = match fs::read(&self.paths.config) {
			Ok(contents) => {
				trace!("Config of {} read", self.name);
				contents
			}
			Err(err) => {
				error!("Config of {} could not be read: {err}", self.name);
				return;
			}
		};

		let config = match toml::from_slice::<Value>(contents.as_slice()).map(|v| v.as_table().cloned()) {
			Ok(Some(config)) => {
				trace!("Config of {} parsed", self.name);
				config
			}
			Ok(None) => {
				trace!("Config of {} is empty", self.name);
				return;
			}
			Err(err) => {
				error!("Config of {} could not be parsed: {err}", self.name);
				return;
			}
		};

		let mut state = wlock!(self.state);

		for (key, value) in config {
			let value = value.to_string();

			if state.config.set(&key, &value).is_err() {
				state.vars.insert(key, value);
			}
		}

		info!("Config of {} loaded", self.name);
	}

	pub fn save_config(self: &ProgramPtr) -> Result<()> {
		let result = toml::to_string_pretty(&self.config())
			.desc("Failed to serialize config")
			.and_then(|contents| {
				fs::write(&self.paths.config, contents)
					.with_desc(|| format!("Failed to write config to {:?}", self.paths.config))
			});

		match &result {
			Ok(()) => info!("Config of {} saved", self.name),
			Err(err) => warn!("Config of {} could not be saved: {err}", self.name),
		};

		result
	}

	pub fn update_config(self: &ProgramPtr, key: &str, value: &str) -> Result<()> {
		let result = wlock!(self.state)
			.config
			.set(key, value)
			.with_desc(|| format!("Failed to set `{key}` to `{value}`"));

		match &result {
			Ok(()) => info!("Config of {} updated: `{key}` = `{value}`", self.name),
			Err(err) => warn!(
				"Config of {} could not be updated: `{key}` = `{value}`: {err}",
				self.name
			),
		}

		result
	}

	pub(super) fn start(self: &ProgramPtr) -> Result<()> {
		let mut command = if self.paths.executable.get_ext() == "sh" {
			let mut command = StdCommand::new("bash");
			command.arg(&self.paths.executable);
			command
		} else {
			StdCommand::new(&self.paths.executable)
		};

		let mut state = wlock!(self.state);
		let result = command
			.current_dir(self.paths.get_working_directory())
			.envs(&state.vars)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn();

		let name = self.name.bold();
		let mut process = match result {
			Ok(process) => {
				racky_info!("Program {name} started successfully");
				process
			}
			Err(err) => {
				racky_error!("Program {name} failed to start: {err}");
				state.status = Status::Failed(err.to_string());
				return Err(err.into());
			}
		};

		state.status = Status::Running(process.id());
		state.executions += 1;
		state.index += 1;

		let index = state.index;
		let this = self.clone();

		drop(state);

		logger::capture_output(&mut process, &self.paths.logs);
		thread::spawn(move || {
			let status = match process.wait_with_output() {
				Ok(output) => {
					if output.status.success() {
						racky_info!("Program {name} exited successfully");
						Status::Finished(String::from_utf8_lossy(&output.stdout).to_string())
					} else {
						let code = util::get_exit_code(&output.status);
						// Ignore SIGTERM
						if code != 15 {
							racky_error!("Program {name} exited with status code {}", code.to_string().bold());
						}

						Status::Errored(String::from_utf8_lossy(&output.stderr).to_string())
					}
				}
				Err(err) => {
					racky_error!("Program {name} encountered an unexpected error: {err}");
					Status::Errored(err.to_string())
				}
			};

			let success = matches!(status, Status::Finished(_));
			let mut state = wlock!(this.state);

			if state.index != index {
				return;
			}

			state.status = status;

			if !state.config.auto_restart {
				racky_warn!("Program {name} will not restart: {} disabled", "auto_restart".bold());
				return;
			}

			if state.attempts >= state.config.restart_attempts {
				racky_warn!(
					"Program {name} will not restart: Maximum number of restart attempts reached: {}",
					state.attempts.to_string().bold()
				);
				return;
			}

			state.status = Status::Restarting;

			if success {
				state.attempts = 0;
			} else {
				state.attempts += 1;
			}

			racky_info!(
				"Program {name} will restart in {} seconds{}",
				state.config.restart_delay.to_string().bold(),
				if state.attempts > 0 {
					format!(
						". Attempt {}/{}",
						state.attempts.to_string().bold(),
						state.config.restart_attempts.to_string().bold()
					)
				} else {
					String::new()
				}
			);

			let delay = state.config.restart_delay;
			drop(state);

			thread::sleep(Duration::from_secs(delay));

			if rlock!(this.state).index == index {
				this.start().ok();
			}
		});

		Ok(())
	}

	pub(super) fn stop(self: &ProgramPtr) -> Result<()> {
		let mut state = wlock!(self.state);

		let pid = if let Status::Running(pid) = &state.status {
			pid.to_string()
		} else {
			state.status = Status::Stopped;
			return Ok(());
		};
		let name = self.name.bold();

		state.status = Status::Stopped;
		state.index += 1;

		drop(state);

		#[cfg(unix)]
		let result = {
			Command::new("pkill").args(["-P", &pid]).run().ok();
			Command::new("kill").arg(&pid).run()
		};

		#[cfg(windows)]
		let result = Command::new("taskkill").args(["/f", "/t", "/pid", &pid]).run();

		match result {
			Ok(_) => {
				racky_info!("Program {name} stopped successfully");
				Ok(())
			}
			Err(err) => {
				racky_error!("Program {name} failed to stop: {err}");
				Err(err)
			}
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Val, Iter, Get, Set)]
pub struct Config {
	/// Whether to automatically start the program when the Racky server starts
	pub auto_start: bool,
	/// Whether to automatically restart the program after it exits
	pub auto_restart: bool,
	/// The delay in seconds before restarting the program after it exits
	pub restart_delay: u64,
	/// The maximum number of restart attempts after the program exits with an error code
	pub restart_attempts: u64,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			auto_start: false,
			auto_restart: true,
			restart_delay: 3,
			restart_attempts: 5,
		}
	}
}

#[derive(Debug, Default)]
pub struct Paths {
	pub executable: PathBuf,
	pub config: PathBuf,
	pub logs: PathBuf,
}

impl Paths {
	pub fn from_path(path: &Path) -> Self {
		let executable = if path.is_dir() {
			let script = path.join("racky.sh");

			if script.exists() {
				script
			} else {
				path.join("scripts").join("racky.sh")
			}
		} else if !path.exists() {
			path.get_parent().join(format!("{}.sh", path.get_name()))
		} else {
			path.to_owned()
		};

		Self {
			executable,
			..Default::default()
		}
	}

	pub fn from_name(name: &str) -> Self {
		let path = dirs::bin().join(name);

		Self {
			executable: Self::from_path(&path).executable,
			config: dirs::config().join(format!("{name}.toml")),
			logs: dirs::logs().join(name),
		}
	}

	pub fn validate(&self) -> bool {
		self.executable.exists()
	}

	pub fn get_program_root(&self) -> PathBuf {
		if self.executable.get_name() == "racky.sh" {
			let parent = self.executable.get_parent();

			if parent.get_name() == "scripts" {
				parent.get_parent().to_owned()
			} else {
				parent.to_owned()
			}
		} else {
			self.executable.clone()
		}
	}

	pub fn get_working_directory(&self) -> PathBuf {
		let root = self.get_program_root();

		if root.is_dir() {
			root
		} else {
			root.get_parent().to_owned()
		}
	}
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Status {
	#[default]
	Idle,
	Running(u32),
	Restarting,
	Stopped,
	Finished(String),
	Errored(String),
	Failed(String),
}

#[derive(Debug, Default)]
struct State {
	pub vars: HashMap<String, String>,
	pub config: Config,
	pub status: Status,
	pub executions: u64,
	pub attempts: u64,
	pub index: u64,
}
