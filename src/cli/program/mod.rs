use anyhow::Result;
use clap::{Parser, Subcommand};

mod add;

/// Run and setup programs on Racky servers
#[derive(Parser)]
pub struct Program {
	#[command(subcommand)]
	command: ServerCommand,
}

impl Program {
	pub fn main(self) -> Result<()> {
		match self.command {
			ServerCommand::Add(command) => command.main(),
		}
	}
}

#[derive(Subcommand)]
enum ServerCommand {
	Add(add::Add),
}
