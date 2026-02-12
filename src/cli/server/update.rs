use anyhow::Result;
use clap::Parser;

use crate::{client::Client, ext::ResultExt, servers};

/// Update the server to the latest version
#[derive(Parser)]
pub struct Update {
	/// Target server alias
	#[arg(short, long)]
	server: Option<String>,
}

impl Update {
	pub fn main(self) -> Result<()> {
		self.update().desc("Failed to update the server")
	}

	fn update(self) -> Result<()> {
		Client::new(&servers::get(self.server)?).post("server/update")?.handle()
	}
}
