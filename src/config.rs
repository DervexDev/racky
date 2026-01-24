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
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use toml;

use crate::{logger::Table, racky_error, util};

lazy_static! {
	static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

#[optfield(OptConfig, merge_fn, attrs = (derive(Deserialize)))]
#[derive(Debug, Clone, Deserialize, DocumentedFields, Val, Iter, Get, Set)]
pub struct Config {
	/// Default server host name
	pub host: String,
	/// Default server port number
	pub port: u16,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			host: String::from("localhost"),
			port: 5000,
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

		let result = || -> Result<()> {
			let path = util::get_racky_dir()?.join("config").join("racky.toml");

			if path.exists() {
				config.merge_opt(toml::from_str(&fs::read_to_string(path)?)?);
			}

			Ok(())
		}();

		match result {
			Ok(()) => *CONFIG.write().unwrap() = config,
			Err(err) => racky_error!("Failed to load config: {err}"),
		}

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
