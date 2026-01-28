use std::{
	collections::HashMap,
	fs,
	path::PathBuf,
	process::{Command, Stdio},
	sync::{Arc, Mutex},
	thread,
	time::Duration,
};

use colored::Colorize;
use log::{debug, error, trace, warn};
use toml::Value;

use crate::{config::ProgramConfig, dirs, ext::PathExt, logger, racky_error, racky_info, racky_warn};

pub type ArcProgram = Arc<Program>;

#[derive(Debug, Default)]
pub struct Program {
	pub name: String,
	pub paths: ProgramPaths,
	pub config: ProgramConfig,
	pub vars: HashMap<String, String>,
	pub attempts: Mutex<u64>,
}

#[derive(Debug, Default)]
pub struct ProgramPaths {
	pub executable: PathBuf,
	pub config: PathBuf,
	pub logs: PathBuf,
}

impl Program {
	pub fn new(name: &str) -> Option<ArcProgram> {
		let mut executable = dirs::bin().join(name);

		if executable.is_dir() {
			let mut path = executable.clone().join("racky.sh");

			if !path.exists() {
				path = executable.join("scripts").join("racky.sh");
			}

			executable = path
		} else if !executable.exists() {
			executable.pop();
			executable = executable.join(format!("{name}.sh"))
		}

		if !executable.exists() {
			return None;
		}

		let mut program = Self {
			name: name.to_owned(),
			paths: ProgramPaths {
				executable,
				config: dirs::config().join(format!("{name}.toml")),
				logs: dirs::logs().join(name),
			},
			attempts: Mutex::new(0),
			..Default::default()
		};

		program.load_config();

		Some(Arc::new(program))
	}

	pub fn load_config(&mut self) {
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

		for (key, value) in config {
			let value = value.to_string();

			if self.config.set(&key, &value).is_err() {
				self.vars.insert(key, value);
			}
		}

		debug!("{} config loaded successfully", self.name);
	}

	pub fn start(self: &ArcProgram) -> bool {
		let mut command = if self.paths.executable.get_ext() == "sh" {
			let mut command = Command::new("bash");
			command.arg(&self.paths.executable);
			command
		} else {
			Command::new(&self.paths.executable)
		};

		let name = self.name.bold();
		let result = command
			.envs(&self.vars)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn();

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

		logger::capture_output(&mut process, &self.paths.logs);

		let this = self.clone();

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

			if !this.config.auto_restart {
				racky_warn!("Program {} will not restart: {} disabled", name, "auto_restart".bold());
				return;
			}

			let mut attempts = this.attempts.lock().unwrap();

			if *attempts >= this.config.restart_attempts {
				racky_warn!(
					"Program {} will not restart: maximum number of restart attempts reached: {}",
					name,
					attempts.to_string().bold()
				);
				return;
			}

			if success {
				*attempts = 0;
			} else {
				*attempts += 1;
			}

			racky_info!(
				"Program {} will restart in {} seconds. Attempt {}/{}",
				name,
				this.config.restart_delay.to_string().bold(),
				attempts.to_string().bold(),
				this.config.restart_attempts.to_string().bold()
			);

			thread::sleep(Duration::from_secs(this.config.restart_delay));
			this.start();
		});

		true
	}
}
