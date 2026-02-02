use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, logger, servers};

/// Stop the server (software)
#[derive(Parser)]
pub struct Stop {
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Stop {
	pub fn main(self) -> Result<()> {
		if !logger::prompt(
			"Are you sure you want to stop the server? This will only stop system service but you will need to start it manually to use Racky again!",
			true,
		) {
			return Ok(());
		}

		self.stop().desc("Failed to stop the server")
	}

	fn stop(self) -> Result<()> {
		Client::new(&servers::get(self.server)?).post("server/stop")?.handle()
	}
}
