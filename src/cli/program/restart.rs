use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, servers};

/// Restart a program on the server
#[derive(Parser)]
pub struct Restart {
	/// Program name to restart
	#[arg()]
	program: String,
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Restart {
	pub fn main(self) -> Result<()> {
		self.restart().desc("Failed to restart program")
	}

	fn restart(self) -> Result<()> {
		Client::new(&servers::get(self.server)?)
			.text("program", self.program)
			.post("program/restart")?
			.handle()
	}
}
