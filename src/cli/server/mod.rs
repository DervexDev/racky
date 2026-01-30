use anyhow::Result;
use clap::{Parser, Subcommand};

mod add;
mod list;
mod remove;
mod serve;
mod update;

/// Manage and configure Racky servers
#[derive(Parser)]
pub struct Server {
	#[command(subcommand)]
	command: ServerCommand,
}

impl Server {
	pub fn main(self) -> Result<()> {
		match self.command {
			ServerCommand::Add(command) => command.main(),
			ServerCommand::List(command) => command.main(),
			ServerCommand::Remove(command) => command.main(),
			ServerCommand::Serve(command) => command.main(),
			ServerCommand::Update(command) => command.main(),
		}
	}
}

#[derive(Subcommand)]
enum ServerCommand {
	Add(add::Add),
	List(list::List),
	Remove(remove::Remove),
	Serve(serve::Serve),
	Update(update::Update),
}
