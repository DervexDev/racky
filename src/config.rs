use std::{
	fmt::Debug,
	fs,
	sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::{Context, Error, Result, bail};
use config_derive::{Get, Iter, Set, Val};
use documented::DocumentedFields;
use lazy_static::lazy_static;
use log::{error, info, warn};
use optfield::optfield;
use serde::{Deserialize, Serialize};
use toml;

use crate::{dirs, ext::ResultExt, logger::Table};

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
			log_file_limit: 20,
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

	pub fn load() {
		let mut config = Self::default();
		let path = dirs::config().join("racky.toml");

		let result = if path.exists() {
			fs::read_to_string(path)
				.map_err(Error::from)
				.and_then(|contents| toml::from_str(&contents).map_err(Error::from))
				.map(|optional| config.merge_opt(optional))
		} else {
			Ok(())
		};

		match result {
			Ok(()) => info!("Racky config loaded"),
			Err(err) => error!("Racky config could not be loaded: {err}"),
		}

		*CONFIG.write().unwrap() = config;
	}

	pub fn save(&self) -> Result<()> {
		let result = toml::to_string_pretty(self)
			.desc("Failed to serialize config")
			.and_then(|contents| fs::write(dirs::config().join("racky.toml"), contents).desc("Failed to write config"));

		match &result {
			Ok(()) => info!("Racky config saved"),
			Err(err) => warn!("Racky config could not be saved: {err}"),
		}

		result
	}

	pub fn update(&mut self, key: &str, value: &str) -> Result<()> {
		let result = self
			.set(key, value)
			.with_context(|| format!("Failed to set `{key}` to `{value}`"));

		match &result {
			Ok(()) => info!("Racky config updated: `{key}` = `{value}`"),
			Err(err) => warn!("Racky config could not be updated: {err}"),
		}

		result
	}

	pub fn list(&self) -> Table {
		let mut table = Table::new();
		let defaults = Self::default();
		let defaults_only = self == &defaults;

		if defaults_only {
			table.set_header(vec!["Setting", "Default", "Description"]);
		} else {
			table.set_header(vec!["Setting", "Default", "Current", "Description"]);
		}

		for (setting, default) in &defaults {
			let doc = Self::get_field_docs(setting).unwrap_or_default().trim().to_owned();

			if defaults_only {
				table.add_row(vec![setting.to_owned(), default.to_string(), doc]);
				continue;
			}

			let default = default.to_string();
			let mut current = self.get(setting).map(|v| v.to_string()).unwrap();

			if current == default {
				current = String::new();
			}

			table.add_row(vec![setting.to_owned(), default, current, doc]);
		}

		table
	}

	pub fn apply_user_data(&mut self, data: Vec<String>, restore: bool, list: bool) -> Result<String> {
		let defaults = Self::default();

		if list {
			return Ok(format!("Racky configuration:\n{}", self.list()));
		} else if restore {
			return defaults
				.save()
				.map(|_| String::from("Configuration restored to defaults successfully"));
		}

		if data.is_empty() {
			bail!("No key=value pairs provided");
		}

		let mut changed = 0;

		for pair in data {
			let (key, value) = pair
				.split_once('=')
				.with_context(|| format!("Invalid key=value or key= pair: {}", pair))?;

			let original = if let Some(original) = self.get(key) {
				original
			} else {
				bail!("Setting `{key}` does not exist");
			};

			if value.is_empty() {
				self.update(key, &defaults.get(key).unwrap().to_string()).unwrap();
			} else {
				self.update(key, value)?;
			}

			if self.get(key).unwrap() != original {
				changed += 1;
			}
		}

		self.save()?;

		Ok(format!("Configuration updated successfully ({changed} changed)"))
	}
}
