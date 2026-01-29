use std::{
	fmt::Debug,
	fs,
	path::Path,
	sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::Result;
use config_derive::{Get, Iter, Set, Val};
use documented::DocumentedFields;
use lazy_static::lazy_static;
use optfield::optfield;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use toml;

use crate::{dirs, logger::Table};

lazy_static! {
	static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

#[optfield(OptConfig, merge_fn, attrs = (derive(Deserialize)))]
#[derive(Debug, Clone, Deserialize, DocumentedFields, Val, Iter, Get, Set)]
pub struct Config {
	/// Default server alias
	pub alias: String,
	/// Default server address
	pub address: String,
	/// Default server port
	pub port: u16,
	/// Default server password
	pub password: String,
	/// Maximum size of a log file in megabytes
	pub log_size_limit: usize,
	/// Maximum number of log files to keep
	pub log_file_limit: usize,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			alias: String::from("default"),
			address: String::from("localhost"),
			port: 5000,
			password: String::new(),
			log_size_limit: 10,
			log_file_limit: 8,
		}
	}
}

impl Config {
	pub fn new() -> RwLockReadGuard<'static, Self> {
		CONFIG.read().unwrap()
	}

	pub fn new_mut() -> RwLockWriteGuard<'static, Self> {
		CONFIG.try_write().expect("Failed to acquire write lock on config")
	}

	pub fn load() -> Result<()> {
		let mut config = Self::default();

		let path = dirs::config().join("racky.toml");

		if path.exists() {
			config.merge_opt(toml::from_str(&fs::read_to_string(path)?)?);
		}

		*CONFIG.write().unwrap() = config;

		Ok(())
	}

	pub fn save(&self, path: &Path) -> Result<()> {
		fs::write(path, toml::to_string(self)?)?;
		Ok(())
	}

	pub fn has_setting(&self, setting: &str) -> bool {
		self.get(setting).is_some()
	}

	pub fn list(&self) -> Table {
		let defaults = Self::default();
		let mut table = Table::new();
		let defaults_only = self == &defaults;

		if defaults_only {
			table.set_header(vec!["Setting", "Default", "Description"]);
		} else {
			table.set_header(vec!["Setting", "Default", "Current", "Description"]);
		}

		for (setting, default) in &defaults {
			if let Ok(doc) = Self::get_field_docs(setting) {
				if defaults_only {
					table.add_row(vec![setting.to_owned(), default.to_string(), doc.trim().to_owned()]);
				} else {
					let default = default.to_string();
					let mut current = self.get(setting).map(|v| v.to_string()).unwrap();

					if current == default {
						current = String::new();
					}

					table.add_row(vec![setting.to_owned(), default, current, doc.trim().to_owned()]);
				}
			}
		}

		table
	}
}

impl PartialEq for Config {
	fn eq(&self, other: &Self) -> bool {
		for (k, v) in self {
			if other.get(k) != Some(v) {
				return false;
			}
		}

		true
	}
}

impl Serialize for Config {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut map = serializer.serialize_map(None)?;
		let defaults = Self::default();

		for (k, v) in self {
			if v == defaults.get(k).unwrap() {
				continue;
			}

			map.serialize_entry(&k, &v)?;
		}

		map.end()
	}
}

#[derive(Debug, Serialize, Deserialize, Set)]
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
