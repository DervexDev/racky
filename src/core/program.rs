use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
	process::{Command, Stdio},
	sync::{Arc, RwLock},
	thread,
	time::Duration,
};

use anyhow::Result;
use colored::Colorize;
use config_derive::{Get, Iter, Set, Val};
use log::{debug, error, trace, warn};
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::{
	dirs,
	ext::{PathExt, ResultExt},
	logger, racky_error, racky_info, racky_warn, rlock, wlock,
};

pub type ProgramPtr = Arc<Program>;

#[derive(Debug, Default)]
pub struct Program {
	name: String,
	paths: ProgramPaths,
	state: RwLock<ProgramState>,
}

impl Program {
	pub fn new(name: &str) -> ProgramPtr {
		Arc::new(Self {
			name: name.to_owned(),
			paths: ProgramPaths::new(name),
			..Default::default()
		})
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn paths(&self) -> &ProgramPaths {
		&self.paths
	}

	pub fn config(&self) -> ProgramConfig {
		rlock!(self.state).config.clone()
	}

	pub fn is_valid(&self) -> bool {
		!self.paths.executable.is_empty()
	}

	pub fn load_config(self: &ProgramPtr) {
		if !self.paths.config.exists() {
			warn!("No config file found for program {}", self.name);
			return;
		}

		let contents = match fs::read(&self.paths.config) {
			Ok(contents) => {
				trace!("{} config file read successfully", self.name);
				contents
			}
			Err(err) => {
				error!("Failed to read {} config file: {err}", self.name);
				return;
			}
		};

		let config = match toml::from_slice::<Value>(contents.as_slice()).map(|v| v.as_table().cloned()) {
			Ok(Some(config)) => {
				trace!("{} config file parsed successfully", self.name);
				config
			}
			Ok(None) => {
				trace!("{} config file is empty", self.name);
				return;
			}
			Err(err) => {
				error!("Failed to parse {} config file: {err}", self.name);
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

		debug!("{} config loaded successfully", self.name);
	}

	pub fn save_config(self: &ProgramPtr) -> Result<()> {
		toml::to_string_pretty(&self.config())
			.with_desc(|| format!("Failed to serialize {} config", self.name))
			.and_then(|contents| {
				fs::write(&self.paths.config, contents)
					.with_desc(|| format!("Failed to write {} config to {:?}", self.name, self.paths.config))
			})
	}

	pub fn update_config(self: &ProgramPtr, key: &str, value: &str) -> Result<()> {
		wlock!(self.state)
			.config
			.set(key, value)
			.with_desc(|| format!("Failed to set `{}` to `{}`", key, value))?;

		Ok(())
	}

	pub fn start(self: &ProgramPtr) -> bool {
		let mut command = if self.paths.executable.get_ext() == "sh" {
			let mut command = Command::new("bash");
			command.arg(&self.paths.executable);
			command
		} else {
			Command::new(&self.paths.executable)
		};

		let mut state = wlock!(self.state);
		let result = command
			.envs(&state.vars)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn();

		let name = self.name.bold();
		let mut process = match result {
			Ok(process) => {
				racky_info!("Program {} started successfully", name);
				process
			}
			Err(err) => {
				racky_error!("Failed to start program {}: {err}", name);
				return false;
			}
		};

		state.pid = Some(process.id());
		state.executions += 1;

		let this = self.clone();

		logger::capture_output(&mut process, &self.paths.logs);
		thread::spawn(move || {
			let success = match process.wait() {
				Ok(status) => {
					if status.success() {
						racky_info!("Program {} exited successfully", name);
						true
					} else {
						racky_error!(
							"Program {} exited with status code {}",
							name,
							status.code().unwrap_or(-1).to_string().bold()
						);
						false
					}
				}
				Err(err) => {
					racky_error!("Unexpected error while waiting for program {}: {err}", name);
					false
				}
			};

			let mut state = wlock!(this.state);
			state.pid = None;

			if !state.config.auto_restart {
				racky_warn!("Program {} will not restart: {} disabled", name, "auto_restart".bold());
				return;
			}

			if state.attempts >= state.config.restart_attempts {
				racky_warn!(
					"Program {} will not restart: maximum number of restart attempts reached: {}",
					name,
					state.attempts.to_string().bold()
				);
				return;
			}

			if success {
				state.attempts = 0;
			} else {
				state.attempts += 1;
			}

			racky_info!(
				"Program {} will restart in {} seconds. Attempt {}/{}",
				name,
				state.config.restart_delay.to_string().bold(),
				state.attempts.to_string().bold(),
				state.config.restart_attempts.to_string().bold()
			);

			let delay = state.config.restart_delay;
			drop(state);

			thread::sleep(Duration::from_secs(delay));
			this.start();
		});

		true
	}

	pub fn stop(self: &ProgramPtr) -> bool {
		let mut state = wlock!(self.state);

		let pid = if let Some(pid) = state.pid {
			pid.to_string()
		} else {
			return false;
		};

		state.pid = None;
		drop(state);

		#[cfg(unix)]
		{
			// Kill main process
			Command::new("kill").arg(&pid).output().ok();

			// Kill child processes
			Command::new("pkill").arg("-P").arg(&pid).output().ok();
		}

		// Kill both main and child processes
		#[cfg(windows)]
		Command::new("taskkill")
			.arg("/F")
			.arg("/T")
			.args(["/PID", &pid])
			.output()
			.ok();

		true
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Val, Iter, Get, Set)]
pub struct ProgramConfig {
	/// Whether to automatically start the program when the Racky server starts
	pub auto_start: bool,
	/// Whether to automatically restart the program after it exits
	pub auto_restart: bool,
	/// The delay in seconds before restarting the program after it exits
	pub restart_delay: u64,
	/// The maximum number of restart attempts after the program exits with an error code
	pub restart_attempts: u64,
}

impl Default for ProgramConfig {
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
pub struct ProgramPaths {
	pub executable: PathBuf,
	pub config: PathBuf,
	pub logs: PathBuf,
}

impl ProgramPaths {
	pub fn new(name: &str) -> Self {
		Self {
			executable: find_executable(&dirs::bin().join(name)).unwrap_or_default(),
			config: dirs::config().join(format!("{name}.toml")),
			logs: dirs::logs().join(name),
		}
	}
}

#[derive(Debug, Default)]
struct ProgramState {
	pub config: ProgramConfig,
	pub vars: HashMap<String, String>,
	pub pid: Option<u32>,
	pub executions: u64,
	pub attempts: u64,
}

pub fn find_executable(path: &Path) -> Option<PathBuf> {
	let mut executable = path.to_owned();

	if executable.is_dir() {
		let mut path = executable.clone().join("racky.sh");

		if !path.exists() {
			path = executable.join("scripts").join("racky.sh");
		}

		executable = path
	} else if !executable.exists() {
		executable.pop();
		executable = executable.join(format!("{}.sh", path.get_name()))
	}

	if executable.exists() { Some(executable) } else { None }
}
