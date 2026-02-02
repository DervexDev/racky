use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, logger, servers};

/// Shutdown the server (hardware)
#[derive(Parser)]
pub struct Shutdown {
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Shutdown {
	pub fn main(self) -> Result<()> {
		if !logger::prompt(
			"Are you sure you want to shutdown the server? This will shutdown your actual hardware and you will need to start it manually to use Racky again!",
			true,
		) {
			return Ok(());
		}

		self.shutdown().desc("Failed to shutdown the server")
	}

	fn shutdown(self) -> Result<()> {
		Client::new(&servers::get(self.server)?)
			.post("server/shutdown")?
			.handle()
	}
}
