use anyhow::{Result, ensure};
use clap::Parser;
use colored::Colorize;

use crate::{ext::ResultExt, racky_info, servers};

/// Remove a saved server
#[derive(Parser)]
pub struct Remove {
	/// Alias of the server to remove
	#[arg()]
	server: String,
}

impl Remove {
	pub fn main(self) -> Result<()> {
		self.remove().desc("Failed to remove the server")
	}

	fn remove(self) -> Result<()> {
		let mut servers = servers::read()?;

		ensure!(
			servers.remove(&self.server).is_some(),
			"Server with alias {} does not exist",
			self.server.bold()
		);

		servers::write(&servers)?;

		racky_info!("Server {} removed successfully", self.server.bold());

		Ok(())
	}
}
