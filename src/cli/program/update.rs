use std::path::PathBuf;

use anyhow::{Result, ensure};
use clap::Parser;
use colored::Colorize;

use crate::{
	client::Client,
	core::program::Paths,
	ext::{PathExt, ResultExt},
	servers, zip,
};

/// Update a program on the server
#[derive(Parser)]
pub struct Update {
	/// Path to program file or directory
	#[arg()]
	path: PathBuf,
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Update {
	pub fn main(self) -> Result<()> {
		self.update().desc("Failed to update program")
	}

	fn update(self) -> Result<()> {
		let path = self.path.resolve().desc("Failed to resolve path")?;

		ensure!(
			Paths::from_path(&path).validate(),
			"Path {} does not point to a valid program",
			path.to_string().bold()
		);

		Client::new(&servers::get(self.server)?)
			.binary("file", zip::compress(&path).desc("Failed to zip program")?)
			.post("program/update")?
			.handle()
	}
}
