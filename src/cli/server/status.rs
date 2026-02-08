use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, servers};

/// Get the status of the server
#[derive(Parser)]
pub struct Status {
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Status {
	pub fn main(self) -> Result<()> {
		self.status().desc("Failed to get server status")
	}

	fn status(self) -> Result<()> {
		Client::new(&servers::get(self.server)?)
			.get("server/status")?
			.with_prefix("Server status:\n")
			.handle()
	}
}
