use anyhow::{Result, bail};
use clap::Parser;
use colored::Colorize;

use crate::{config::Config, core::Core, ext::ResultExt, racky_info, server::Server};

/// Start a Racky server
#[derive(Parser)]
pub struct Start {
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

impl Start {
	pub fn main(self) -> Result<()> {
		let config = Config::new();

		let address = self.address.unwrap_or(config.address.clone());
		let port = self.port.unwrap_or(config.port);
		let password = self
			.password
			.or(Some(config.password.clone()))
			.filter(|p| !p.is_empty());

		let mut core = Core::new();
		let result = core.start().desc("Failed to start core")?;

		racky_info!(
			"Started {} of {} autostart programs",
			result.0.to_string().bold(),
			result.1.to_string().bold()
		);

		let server = Server::new(core, &address, port, password);

		if !server.is_port_free() {
			bail!("Port {} is already in use", port.to_string().bold());
		}

		racky_info!(
			"Racky server is running on {}",
			format!("http://{address}:{port}").bold()
		);

		server.start().desc("Failed to start server")
	}
}
