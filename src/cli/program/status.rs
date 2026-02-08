use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, servers};

/// Get the status of a program on the server
#[derive(Parser)]
pub struct Status {
	/// Name of the program to get the status of
	#[arg()]
	program: String,
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Status {
	pub fn main(self) -> Result<()> {
		self.status().desc("Failed to get program status")
	}

	fn status(self) -> Result<()> {
		Client::new(&servers::get(self.server)?)
			.text("program", self.program)
			.get("program/status")?
			.with_prefix("Program status:\n")
			.handle()
	}
}
