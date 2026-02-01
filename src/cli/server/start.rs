use anyhow::{Result, bail};
use clap::Parser;
use colored::Colorize;

use crate::{config::Config, core::Core, ext::ResultExt, racky_error, racky_info, racky_warn, server::Server};

/// Start actual Racky server (used by systemd service)
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
		self.start().desc("Failed to start server")
	}

	pub fn start(self) -> Result<()> {
		let config = Config::new();

		let address = self.address.unwrap_or(config.address.clone());
		let port = self.port.unwrap_or(config.port);
		let password = self
			.password
			.or(Some(config.password.clone()))
			.filter(|p| !p.is_empty());

		let core = Core::new();
		let server = Server::new(core.clone(), &address, port, password);

		if !server.is_port_free() {
			bail!("Port {} is already in use", port.to_string().bold());
		}

		let result = core.start().desc("Failed to start core")?;
		let message = format!(
			"Started {} of {} autostart programs",
			result.0.to_string().bold(),
			result.1.to_string().bold()
		);

		if result.0 == result.1 {
			racky_info!("{message}");
		} else if result.0 == 0 {
			racky_error!("{message}");
		} else {
			racky_warn!("{message}");
		}

		racky_info!(
			"Racky server is running on {}",
			format!("http://{address}:{port}").bold()
		);

		drop(core);
		server.start().desc("Could not start the serve session")
	}
}
