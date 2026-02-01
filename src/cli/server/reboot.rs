use anyhow::{Result, bail};
use clap::Parser;

use crate::{client::Client, ext::ResultExt, racky_info, servers};

/// Reboot the server
#[derive(Parser)]
pub struct Reboot {
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Reboot {
	pub fn main(self) -> Result<()> {
		self.reboot().desc("Failed to reboot server")
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
