use anyhow::{Result, ensure};
use clap::Parser;
use colored::Colorize;
use log::{trace, warn};

use crate::{
	config::{self, Config},
	core::Core,
	ext::ResultExt,
	racky_error, racky_info, racky_warn,
	servers::{self, Server},
	web::Web,
};

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
			.as_ref()
			.and_then(|p| config::hash_password(p).ok())
			.or_else(|| {
				let p = config.password.clone();
				if p.is_empty() || !config::is_password_hash(&p) {
					None
				} else {
					Some(p)
				}
			});

		let core = Core::new();
		let web = Web::new(core.clone(), &address, port, password.clone());

		ensure!(web.is_port_free(), "Port {} is already in use", port.to_string().bold());

		match Self::save_server(&address, port, self.password.filter(|p| !p.is_empty())) {
			Ok(true) => trace!("Saved local server details"),
			Err(err) => warn!("Failed to save local server: {err}"),
			_ => (),
		}

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
			format!("https://{address}:{port}").bold()
		);

		drop(config);
		drop(core);

		web.serve().desc("Could not start the serve session")
	}

	fn save_server(address: &str, port: u16, password: Option<String>) -> Result<bool> {
		let mut servers = servers::read()?;

		if servers.contains_key("local") {
			return Ok(false);
		}

		servers.insert(
			String::from("local"),
			Server {
				address: address.to_owned(),
				port,
				password: password.unwrap_or_default(),
				default: !servers.values().any(|s| s.default),
			},
		);

		servers::write(&servers)?;

		Ok(true)
	}
}
