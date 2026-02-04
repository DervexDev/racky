use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, servers};

/// Stop a program on the server
#[derive(Parser)]
pub struct Stop {
	/// Program name to stop
	#[arg()]
	program: String,
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Stop {
	pub fn main(self) -> Result<()> {
		self.stop().desc("Failed to stop program")
	}

	fn stop(self) -> Result<()> {
		Client::new(&servers::get(self.server)?)
			.text("program", self.program)
			.post("program/stop")?
			.handle()
	}
}
