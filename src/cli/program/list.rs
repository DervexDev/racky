use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, servers};

/// List all programs on the server
#[derive(Parser)]
pub struct List {
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl List {
	pub fn main(self) -> Result<()> {
		self.list().desc("Failed to list server programs")
	}

	fn list(self) -> Result<()> {
		Client::new(&servers::get(self.server)?)
			.get("program/list")?
			.with_prefix("Program list:\n")
			.handle()
	}
}
