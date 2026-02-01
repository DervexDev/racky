use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;
use colored::Colorize;

use crate::{
	client::Client,
	core::program::{self},
	ext::{PathExt, ResultExt},
	racky_info, servers, zip,
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

		if program::find_executable(&path).is_none() {
			bail!("Path {} does not point to a valid program", path.to_string().bold());
		}

		let (status, body) = Client::new(&servers::get(self.server)?)
			.file("file", zip::compress(&path).desc("Failed to zip program")?)
			.text("auto_start", self.auto_start)
			.post("program/add")?;

		if status.is_success() {
			racky_info!("{body}");
		} else {
			bail!("{body} ({status})");
		}

		Ok(())
	}
}
