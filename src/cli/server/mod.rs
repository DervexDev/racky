use anyhow::Result;
use clap::{Parser, Subcommand};

mod start;

/// Manage Racky servers
#[derive(Parser)]
pub struct Server {
	#[command(subcommand)]
	command: ServerCommand,
}

impl Server {
	pub fn main(self) -> Result<()> {
		match self.command {
			ServerCommand::Start(command) => command.main(),
		}
	}
}

#[derive(Subcommand)]
enum ServerCommand {
	Start(start::Start),
}
