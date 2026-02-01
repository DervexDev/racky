use anyhow::{Result, bail};
use clap::Parser;
use colored::Colorize;

use crate::{
	config::Config,
	ext::ResultExt,
	racky_info,
	servers::{self, Server},
};

/// Configure a new server
#[derive(Parser)]
pub struct Add {
	/// Server alias (must be unique)
	#[arg()]
	server: String,
	/// Server address
	#[arg(short = 'A', long)]
	address: Option<String>,
	/// Server port
	#[arg(short = 'P', long)]
	port: Option<u16>,
	/// Server password
	#[arg(short, long)]
	password: Option<String>,
}

impl Add {
	pub fn main(self) -> Result<()> {
		self.add().desc("Failed to add a new server")
	}

	fn add(self) -> Result<()> {
		let config = Config::default();

		let address = self.address.unwrap_or(config.address);
		let port = self.port.unwrap_or(config.port);
		let password = self.password.unwrap_or(config.password);

		let mut servers = servers::read()?;

		if servers.contains_key(&self.server) {
			bail!("Server with alias {} already exists", self.server.bold());
		}

		if servers.values().any(|s| s.address == address && s.port == port) {
			bail!(
				"Server with address {} and port {} already exists",
				address.bold(),
				port.to_string().bold()
			);
		}

		servers.insert(
			self.server.clone(),
			Server {
				address: address.clone(),
				port,
				password,
				default: !servers.values().any(|s| s.default),
			},
		);

		servers::write(&servers)?;

		racky_info!(
			"Server {} with URL {} added successfully",
			self.server.bold(),
			format!("http://{address}:{port}").bold()
		);

		Ok(())
	}
}
