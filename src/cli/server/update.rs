use anyhow::{Result, bail};
use clap::Parser;
use colored::Colorize;

use crate::{
	cli::server::{read_servers, write_servers},
	ext::ResultExt,
	racky_info,
};

/// Update the configuration of an existing server
#[derive(Parser)]
pub struct Update {
	/// Server alias to update
	#[arg()]
	alias: String,
	/// New server alias
	#[arg(short = 'a', long = "alias")]
	new_alias: Option<String>,
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
		self.update().desc("Failed to update server")
	}

	fn update(self) -> Result<()> {
		let mut servers = read_servers()?;
		let mut server = if let Some(server) = servers.remove(&self.alias) {
			server
		} else {
			bail!("Server with alias {} does not exist", self.alias.bold());
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

		let alias = if let Some(new_alias) = self.new_alias {
			updated = true;
			new_alias
		} else {
			self.alias
		};

		if !updated {
			bail!("No changes detected");
		}

		servers.insert(alias.clone(), server);
		write_servers(&servers)?;

		racky_info!("Server {} updated successfully", alias.bold(),);

		Ok(())
	}
}
