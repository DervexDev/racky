use anyhow::Result;
use clap::{Parser, Subcommand};

mod add;
mod config;
mod list;
mod logs;
mod remove;
mod restart;
mod start;
mod status;
mod stop;
mod update;

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
			Command::Config(command) => command.main(),
			Command::List(command) => command.main(),
			Command::Logs(command) => command.main(),
			Command::Remove(command) => command.main(),
			Command::Restart(command) => command.main(),
			Command::Start(command) => command.main(),
			Command::Status(command) => command.main(),
			Command::Stop(command) => command.main(),
			Command::Update(command) => command.main(),
		}
	}
}

#[derive(Subcommand)]
enum Command {
	Add(add::Add),
	Config(config::Config),
	List(list::List),
	Logs(logs::Logs),
	Remove(remove::Remove),
	Restart(restart::Restart),
	Start(start::Start),
	Status(status::Status),
	Stop(stop::Stop),
	Update(update::Update),
}
