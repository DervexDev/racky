use anyhow::Result;
use clap::{Parser, Subcommand};

mod add;
mod remove;
mod restart;
mod start;
mod stop;

/// Run and setup programs on Racky servers
#[derive(Parser)]
pub struct Program {
	#[command(subcommand)]
	command: Command,
}

impl Program {
	pub fn main(self) -> Result<()> {
		match self.command {
			Command::Add(command) => command.main(),
			Command::Remove(command) => command.main(),
			Command::Restart(command) => command.main(),
			Command::Start(command) => command.main(),
			Command::Stop(command) => command.main(),
		}
	}
}

#[derive(Subcommand)]
enum Command {
	Add(add::Add),
	Remove(remove::Remove),
	Restart(restart::Restart),
	Start(start::Start),
	Stop(stop::Stop),
}
