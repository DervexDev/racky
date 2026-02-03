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

/// Add a new program to the server
#[derive(Parser)]
pub struct Add {
	/// Path to the program file or directory
	#[arg()]
	path: PathBuf,
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
	/// Automatically start the program
	#[arg(short, long)]
	auto_start: bool,
}

impl Add {
	pub fn main(self) -> Result<()> {
		self.add().desc("Failed to add program")
	}

	fn add(self) -> Result<()> {
		let path = self.path.resolve().desc("Failed to resolve path")?;

		ensure!(
			Paths::from_path(&path).validate(),
			"Path {} does not point to a valid program",
			path.to_string().bold()
		);

		Client::new(&servers::get(self.server)?)
			.file("file", zip::compress(&path).desc("Failed to zip program")?)
			.text("auto_start", self.auto_start)
			.post("program/add")?
			.handle()
	}
}
