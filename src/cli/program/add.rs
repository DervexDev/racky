use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;
use colored::Colorize;
use reqwest::blocking::{
	Client,
	multipart::{Form, Part},
};

use crate::{
	constants::USER_AGENT,
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
	/// Whether to automatically start the program
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

		let server = servers::get(self.server)?;
		let zip = zip::compress(&path).desc("Failed to zip program")?;

		let mut request = Client::builder()
			.build()
			.desc("Failed to create HTTP client")?
			.post(format!("http://{}:{}/program/add", server.address, server.port))
			.header("User-Agent", USER_AGENT)
			.multipart(
				Form::new()
					.part("file", Part::bytes(zip))
					.text("auto_start", self.auto_start.to_string()),
			);

		if !server.password.is_empty() {
			request = request.header("Authorization", &server.password);
		}

		let response = request.send().desc("Failed to connect to server")?;
		let status = response.status();
		let body = response.text().unwrap_or_default();

		if !status.is_success() {
			bail!("{body} ({status})");
		}

		racky_info!("{body}");

		Ok(())
	}
}
