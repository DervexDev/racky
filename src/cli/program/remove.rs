use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, servers};

/// Remove a program from the server
#[derive(Parser)]
pub struct Remove {
	/// Program name to remove
	#[arg()]
	program: String,
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Remove {
	pub fn main(self) -> Result<()> {
		self.remove().desc("Failed to remove program")
	}

	fn remove(self) -> Result<()> {
		Client::new(&servers::get(self.server)?)
			.text("program", self.program)
			.post("program/remove")?
			.handle()
	}
}
