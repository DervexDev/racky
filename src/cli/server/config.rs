use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, servers};

/// Update server configuration
#[derive(Parser)]
pub struct Config {
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
		self.config().desc("Failed to update/list server config")
	}

	fn config(self) -> Result<()> {
		Client::new(&servers::get(self.server)?)
			.text("list", self.list)
			.text("default", self.default)
			.text("data", self.data.join(","))
			.post("server/config")?
			.handle()
	}
}
