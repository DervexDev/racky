use anyhow::Result;
use clap::{Parser, Subcommand};

mod add;
mod list;
mod reboot;
mod remove;
mod restart;
mod shutdown;
mod start;
mod stop;
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
			ServerCommand::Reboot(command) => command.main(),
			ServerCommand::Remove(command) => command.main(),
			ServerCommand::Restart(command) => command.main(),
			ServerCommand::Shutdown(command) => command.main(),
			ServerCommand::Start(command) => command.main(),
			ServerCommand::Stop(command) => command.main(),
			ServerCommand::Update(command) => command.main(),
		}
	}
}

#[derive(Subcommand)]
enum ServerCommand {
	Add(add::Add),
	List(list::List),
	Reboot(reboot::Reboot),
	Remove(remove::Remove),
	Restart(restart::Restart),
	Shutdown(shutdown::Shutdown),
	Start(start::Start),
	Stop(stop::Stop),
	Update(update::Update),
}
