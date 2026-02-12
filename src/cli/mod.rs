use std::env;

use anyhow::Result;
use clap::{ColorChoice, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use env_logger::WriteStyle;
use log::LevelFilter;

use crate::util;

mod config;
mod install;
mod program;
mod server;
mod uninstall;
mod update;

macro_rules! about {
	() => {
		concat!("Racky ", env!("CARGO_PKG_VERSION"))
	};
}

macro_rules! long_about {
	() => {
		concat!(
			"Racky ",
			env!("CARGO_PKG_VERSION"),
			"\n",
			env!("CARGO_PKG_DESCRIPTION"),
			"\n",
			"Made with <3 by ",
			env!("CARGO_PKG_AUTHORS")
		)
	};
}

#[derive(Parser)]
#[clap(about = about!(), long_about = long_about!(), version)]
pub struct Cli {
	#[command(subcommand)]
	command: Commands,

	#[command(flatten)]
	verbose: Verbosity,

	/// Automatically answer to any prompts
	#[arg(short, long, global = true)]
	yes: bool,

	/// Print full backtrace on panic
	#[arg(short = 'B', long, global = true)]
	backtrace: bool,

	/// Output coloring mode
	#[arg(
		long,
		short = 'C',
		global = true,
		value_name = "WHEN",
		default_value = "auto",
		hide_default_value = true
	)]
	color: ColorChoice,
}

impl Cli {
	pub fn new() -> Cli {
		Cli::parse()
	}

	pub fn yes(&self) -> bool {
		if env::var("RUST_YES").is_ok() {
			return util::env_yes();
		}

		self.yes
	}

	pub fn backtrace(&self) -> bool {
		if env::var("RUST_BACKTRACE").is_ok() {
			return util::env_backtrace();
		}

		self.backtrace
	}

	pub fn verbosity(&self) -> LevelFilter {
		if env::var("RUST_VERBOSE").is_ok() {
			return util::env_verbosity();
		}

		self.verbose.log_level_filter()
	}

	pub fn log_style(&self) -> WriteStyle {
		if env::var("RUST_LOG_STYLE").is_ok() {
			return util::env_log_style();
		}

		match self.color {
			ColorChoice::Always => WriteStyle::Always,
			ColorChoice::Never => WriteStyle::Never,
			_ => WriteStyle::Auto,
		}
	}

	pub fn is_server_start(&self) -> bool {
		matches!(&self.command, Commands::Server(server) if server.is_start())
	}

	pub fn main(self) -> Result<()> {
		match self.command {
			Commands::Config(command) => command.main(),
			Commands::Install(command) => command.main(),
			Commands::Program(command) => command.main(),
			Commands::Server(command) => command.main(),
			Commands::Uninstall(command) => command.main(),
			Commands::Update(command) => command.main(),
		}
	}
}

#[derive(Subcommand)]
pub enum Commands {
	Config(config::Config),
	Install(install::Install),
	Program(program::Program),
	Server(server::Server),
	Uninstall(uninstall::Uninstall),
	Update(update::Update),
}
