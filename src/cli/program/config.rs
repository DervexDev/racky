use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, servers};

/// Update program configuration
#[derive(Parser)]
pub struct Config {
	/// Name of the program to update the configuration of
	#[arg()]
	program: String,
	/// Key=Value pairs to update
	#[arg()]
	data: Vec<String>,
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
	/// Restore all settings to their default values
	#[arg(short, long)]
	default: bool,
	/// List current configuration
	#[arg(short, long)]
	list: bool,
}

impl Config {
	pub fn main(self) -> Result<()> {
		self.config().desc("Failed to update/list program config")
	}

	fn config(self) -> Result<()> {
		Client::new(&servers::get(self.server)?)
			.text("program", self.program)
			.text("data", self.data.join(","))
			.text("default", self.default)
			.text("list", self.list)
			.post("program/config")?
			.handle()
	}
}
