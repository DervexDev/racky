use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, logger, servers};

/// Reboot the server (hardware)
#[derive(Parser)]
pub struct Reboot {
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Reboot {
	pub fn main(self) -> Result<()> {
		if !logger::prompt(
			"Are you sure you want to reboot the server? This will reboot your actual hardware and you will need to wait until it boots up before you can use Racky again!",
			true,
		) {
			return Ok(());
		}

		self.reboot().desc("Failed to reboot the server")
	}

	fn reboot(self) -> Result<()> {
		Client::new(&servers::get(self.server)?).post("server/reboot")?.handle()
	}
}
