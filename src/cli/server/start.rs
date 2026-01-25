use anyhow::{bail, Result};
use clap::Parser;
use colored::Colorize;

use crate::{config::Config, core::Core, ext::ResultExt, racky_info, server::Server};

/// Start a Racky server
#[derive(Parser)]
pub struct Start {
	/// Server hostname
	#[arg(short = 'H', long)]
	host: Option<String>,
	/// Server port
	#[arg(short = 'P', long)]
	port: Option<u16>,
	/// Server password
	#[arg(short, long)]
	password: Option<String>,
}

impl Start {
	pub fn main(self) -> Result<()> {
		let config = Config::new();

		let host = self.host.unwrap_or(config.host.clone());
		let port = self.port.unwrap_or(config.port);
		let password = self
			.password
			.or(Some(config.password.clone()))
			.filter(|p| !p.is_empty());

		let core = Core::new();
		core.start().desc("Failed to start core")?;

		let server = Server::new(core, &host, port, password);

		if !server.is_port_free() {
			bail!("Port {port} is already in use");
		}

		racky_info!("Racky server is running on {}", server.get_address().bold());

		server.start().desc("Failed to start server")?;

		Ok(())
	}
}
