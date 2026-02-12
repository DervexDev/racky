use anyhow::Result;
use clap::{Parser, Subcommand};

mod add;
mod change;
mod config;
mod list;
mod logs;
mod reboot;
mod remove;
mod restart;
mod shutdown;
mod start;
mod status;
mod stop;
mod update;

/// Manage and configure Racky servers
#[derive(Parser)]
pub struct Server {
	#[command(subcommand)]
	command: Command,
}

impl Server {
	pub fn is_start(&self) -> bool {
		matches!(self.command, Command::Start(_))
	}

	pub fn main(self) -> Result<()> {
		match self.command {
			Command::Add(command) => command.main(),
			Command::Change(command) => command.main(),
			Command::Config(command) => command.main(),
			Command::List(command) => command.main(),
			Command::Logs(command) => command.main(),
			Command::Reboot(command) => command.main(),
			Command::Remove(command) => command.main(),
			Command::Restart(command) => command.main(),
			Command::Shutdown(command) => command.main(),
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
	Change(change::Change),
	Config(config::Config),
	List(list::List),
	Logs(logs::Logs),
	Reboot(reboot::Reboot),
	Remove(remove::Remove),
	Restart(restart::Restart),
	Shutdown(shutdown::Shutdown),
	Start(start::Start),
	Status(status::Status),
	Stop(stop::Stop),
	Update(update::Update),
}
