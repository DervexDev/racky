use anyhow::{Result, ensure};
use clap::Parser;
use colored::Colorize;

use crate::{config::Config, core::Core, ext::ResultExt, racky_error, racky_info, racky_warn, web::Web};

/// Start the Racky server (used by systemd service)
#[derive(Parser)]
pub struct Start {
	/// Address to bind the server to
	#[arg(short = 'A', long)]
	address: Option<String>,
	/// Port to bind the server to
	#[arg(short = 'P', long)]
	port: Option<u16>,
	/// Password for the server API
	#[arg(short, long)]
	password: Option<String>,
}

impl Start {
	pub fn main(self) -> Result<()> {
		self.start().desc("Failed to start the server")
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
		let web = Web::new(core.clone(), &address, port, password);

		ensure!(web.is_port_free(), "Port {} is already in use", port.to_string().bold());

		let (started, total) = core.start().desc("Failed to start core")?;
		let message = format!(
			"Started {} of {} autostart programs",
			started.to_string().bold(),
			total.to_string().bold()
		);

		if total != 0 {
			if started == total {
				racky_info!("{message}");
			} else if started == 0 {
				racky_error!("{message}");
			} else {
				racky_warn!("{}", message.clone().red());
			}
		}

		racky_info!(
			"Racky server is running on {}",
			format!("http://{address}:{port}").bold()
		);

		drop(config);
		drop(core);

		web.serve().desc("Could not start the serve session")
	}
}
