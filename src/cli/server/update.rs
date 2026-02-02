use anyhow::{Result, bail, ensure};
use clap::Parser;
use colored::Colorize;

use crate::{ext::ResultExt, racky_info, servers};

/// Update the configuration of an existing server
#[derive(Parser)]
pub struct Update {
	/// Server alias to update
	#[arg()]
	server: String,
	/// New server alias
	#[arg(short, long)]
	alias: Option<String>,
	/// New server address
	#[arg(short = 'A', long)]
	address: Option<String>,
	/// New server port
	#[arg(short = 'P', long)]
	port: Option<u16>,
	/// New server password
	#[arg(short, long)]
	password: Option<String>,
	/// Set the server as default
	#[arg(short, long)]
	default: Option<bool>,
}

impl Update {
	pub fn main(self) -> Result<()> {
		self.update().desc("Failed to update the server")
	}

	fn update(self) -> Result<()> {
		let mut servers = servers::read()?;
		let mut server = if let Some(server) = servers.remove(&self.server) {
			server
		} else {
			bail!("Server with alias {} does not exist", self.server.bold());
		};

		let mut updated = false;

		if let Some(address) = self.address
			&& address != server.address
		{
			server.address = address;
			updated = true;
		}

		if let Some(port) = self.port
			&& port != server.port
		{
			server.port = port;
			updated = true;
		}

		if let Some(password) = self.password
			&& password != server.password
		{
			server.password = password;
			updated = true;
		}

		if let Some(default) = self.default
			&& default != server.default
		{
			server.default = default;
			updated = true;

			if default && let Some((alias, _)) = servers.iter().find(|(_, s)| s.default) {
				bail!("A default server already exists: {}", alias.bold());
			}
		};

		let alias = if let Some(alias) = self.alias {
			updated = true;
			alias
		} else {
			self.server
		};

		ensure!(updated, "No changes detected");

		servers.insert(alias.clone(), server);
		servers::write(&servers)?;

		racky_info!("Server {} updated successfully", alias.bold(),);

		Ok(())
	}
}
