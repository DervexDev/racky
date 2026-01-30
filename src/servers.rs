use std::{collections::HashMap, fs};

use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};

use crate::{dirs, ext::ResultExt};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Server {
	pub address: String,
	pub port: u16,
	pub password: String,
	pub default: bool,
}

pub type Servers = HashMap<String, Server>;

pub fn read() -> Result<Servers> {
	let path = dirs::racky().join("servers.toml");

	if !path.exists() {
		return Ok(HashMap::new());
	}

	fs::read_to_string(path)
		.map_err(Error::from)
		.and_then(|s| toml::from_str(&s).map_err(Error::from))
		.desc("Failed to read servers file")
}

pub fn write(servers: &Servers) -> Result<()> {
	toml::to_string(&servers)
		.map_err(Error::from)
		.and_then(|s| fs::write(dirs::racky().join("servers.toml"), s).map_err(Error::from))
		.desc("Failed to write servers file")
}

pub fn get(alias: Option<String>) -> Result<Server> {
	read()?
		.into_iter()
		.find(|(a, s)| match &alias {
			Some(alias) => a == alias,
			None => s.default,
		})
		.map(|(_, s)| s)
		.context("No matching server found")
}
