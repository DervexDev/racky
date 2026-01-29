use std::{collections::HashMap, fs};

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::{dirs, ext::ResultExt};

mod add;
mod list;
mod remove;
mod start;
mod update;

/// Manage and configure Racky servers
#[derive(Parser)]
pub struct Server {
	#[command(subcommand)]
	command: ServerCommand,
}

impl Server {
	pub fn main(self) -> Result<()> {
		match self.command {
			ServerCommand::Add(command) => command.main(),
			ServerCommand::List(command) => command.main(),
			ServerCommand::Remove(command) => command.main(),
			ServerCommand::Start(command) => command.main(),
			ServerCommand::Update(command) => command.main(),
		}
	}
}

#[derive(Subcommand)]
enum ServerCommand {
	Add(add::Add),
	List(list::List),
	Remove(remove::Remove),
	Start(start::Start),
	Update(update::Update),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ServerEntry {
	pub address: String,
	pub port: u16,
	pub password: String,
	pub default: bool,
}

pub type Servers = HashMap<String, ServerEntry>;

pub fn read_servers() -> Result<Servers> {
	let path = dirs::racky().join("servers.toml");

	if !path.exists() {
		return Ok(HashMap::new());
	}

	fs::read_to_string(path)
		.map_err(anyhow::Error::from)
		.and_then(|s| toml::from_str(&s).map_err(anyhow::Error::from))
		.desc("Failed to read servers file")
}

pub fn write_servers(servers: &Servers) -> Result<()> {
	toml::to_string(&servers)
		.map_err(anyhow::Error::from)
		.and_then(|s| fs::write(dirs::racky().join("servers.toml"), s).map_err(anyhow::Error::from))
		.desc("Failed to write servers file")
}
