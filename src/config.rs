use std::{
	fmt::Debug,
	fs,
	sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::Result;
use config_derive::{Get, Iter, Set, Val};
use documented::DocumentedFields;
use lazy_static::lazy_static;
use optfield::optfield;
use serde::{Deserialize, Serialize};
use toml;

use crate::{dirs, logger::Table};

lazy_static! {
	static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

#[optfield(OptConfig, merge_fn, attrs = (derive(Deserialize)))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, DocumentedFields, Val, Iter, Get, Set)]
pub struct Config {
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
			address: String::from("0.0.0.0"),
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

	pub fn save(&self) -> Result<()> {
		fs::write(dirs::config().join("racky.toml"), toml::to_string_pretty(self)?)?;
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
