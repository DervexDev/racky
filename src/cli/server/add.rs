use anyhow::{Result, bail};
use clap::Parser;
use colored::Colorize;

use crate::{
	cli::server::{ServerEntry, read_servers, write_servers},
	config::Config,
	ext::ResultExt,
	racky_info,
};

/// Configure a new server
#[derive(Parser)]
pub struct Add {
	/// Server alias (must be unique)
	#[arg()]
	alias: String,
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
		self.add().desc("Failed to add server")
	}

	fn add(self) -> Result<()> {
		let config = Config::default();

		let address = self.address.unwrap_or(config.address);
		let port = self.port.unwrap_or(config.port);
		let password = self.password.unwrap_or(config.password);

		let mut servers = read_servers()?;

		if servers.contains_key(&self.alias) {
			bail!("Server with alias {} already exists", self.alias.bold());
		}

		if servers.values().any(|s| s.address == address && s.port == port) {
			bail!(
				"Server with address {} and port {} already exists",
				address.bold(),
				port.to_string().bold()
			);
		}

		servers.insert(
			self.alias.clone(),
			ServerEntry {
				address: address.clone(),
				port,
				password,
				default: !servers.values().any(|s| s.default),
			},
		);

		write_servers(&servers)?;

		racky_info!(
			"Server {} with URL {} added successfully",
			self.alias.bold(),
			format!("http://{address}:{port}").bold()
		);

		Ok(())
	}
}
