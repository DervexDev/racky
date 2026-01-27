use std::{collections::HashMap, fs, path::PathBuf, process::Command, thread};

use anyhow::Result;
use config_derive::Set;
use log::{debug, error, trace, warn};
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::{dirs, ext::PathExt, racky_error, racky_info};

#[derive(Debug)]
pub struct Program {
	pub name: String,
	pub paths: ProgramPaths,
	pub config: ProgramConfig,
	pub vars: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ProgramPaths {
	pub executable: PathBuf,
	pub config: PathBuf,
	pub logs: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize, Set)]
pub struct ProgramConfig {
	pub auto_start: bool,
	pub auto_restart: bool,
	pub restart_delay: u64,
	pub restart_attempts: u64,
}

impl Program {
	pub fn new(name: &str) -> Option<Program> {
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
			config: ProgramConfig::default(),
			vars: HashMap::new(),
		};

		program.load_config();

		Some(program)
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

	pub fn start(&self) -> bool {
		let mut command = if self.paths.executable.get_ext() == "sh" {
			let mut command = Command::new("bash");
			command.arg(&self.paths.executable);
			command
		} else {
			Command::new(&self.paths.executable)
		};

		let result = command.envs(&self.vars).spawn();

		let mut process = match result {
			Ok(process) => {
				racky_info!("Program {} started successfully", self.name);
				process
			}
			Err(err) => {
				racky_error!("Failed to start program {}: {err}", self.name);
				return false;
			}
		};

		thread::spawn(move || {
			let result = process.wait();
			println!("{:#?}", result);
		});

		true
	}
}
