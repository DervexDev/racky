use anyhow::{Result, bail};
use clap::Parser;

use crate::{client::Client, ext::ResultExt, logger, racky_info, servers};

/// Restart the server (software)
#[derive(Parser)]
pub struct Restart {
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Restart {
	pub fn main(self) -> Result<()> {
		if !logger::prompt(
			"Are you sure you want to restart the server? This will only restart the system service but you may still need to wait a few seconds before you can use Racky again!",
			true,
		) {
			return Ok(());
		}

		self.restart().desc("Failed to restart the server")
	}

	fn restart(self) -> Result<()> {
		let (status, body) = Client::new(&servers::get(self.server)?).post("server/restart")?;

		if status.is_success() {
			racky_info!("{body}");
		} else {
			bail!("{body} ({status})");
		}

		Ok(())
	}
}
