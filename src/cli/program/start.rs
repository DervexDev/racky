use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, servers};

/// Start a program on the server
#[derive(Parser)]
pub struct Start {
	/// Name of the program to start
	#[arg()]
	program: String,
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Start {
	pub fn main(self) -> Result<()> {
		self.start().desc("Failed to start program")
	}

	fn start(self) -> Result<()> {
		Client::new(&servers::get(self.server)?)
			.text("program", self.program)
			.post("program/start")?
			.handle()
	}
}
