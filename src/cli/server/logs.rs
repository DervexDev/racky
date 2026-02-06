use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, servers};

/// Read logs from the server
#[derive(Parser)]
pub struct Logs {
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
	/// Page number (higher values mean older logs)
	#[arg(short, long)]
	page: Option<usize>,
}

impl Logs {
	pub fn main(self) -> Result<()> {
		self.logs().desc("Failed to get server logs")
	}

	fn logs(self) -> Result<()> {
		Client::new(&servers::get(self.server)?)
			.text("page", self.page.unwrap_or_default())
			.get("server/logs")?
			.with_prefix("Server logs:\n")
			.handle()
	}
}
