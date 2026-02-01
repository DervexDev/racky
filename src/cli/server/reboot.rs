use anyhow::{Result, bail};
use clap::Parser;

use crate::{client::Client, ext::ResultExt, logger, racky_info, servers};

/// Reboot the server (hardware)
#[derive(Parser)]
pub struct Reboot {
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Reboot {
	pub fn main(self) -> Result<()> {
		logger::prompt(
			"Are you sure you want to reboot the server? This will reboot your actual hardware and you will need to wait until it boots up before you can use Racky again!",
			true,
		);

		self.reboot().desc("Failed to reboot the server")
	}

	fn reboot(self) -> Result<()> {
		let (status, body) = Client::new(&servers::get(self.server)?).post("server/reboot")?;

		if status.is_success() {
			racky_info!("{body}");
		} else {
			bail!("{body} ({status})");
		}

		Ok(())
	}
}
