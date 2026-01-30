use anyhow::{Result, bail};
use clap::Parser;
use colored::Colorize;

use crate::{ext::ResultExt, racky_info, servers};

/// Remove the existing server
#[derive(Parser)]
pub struct Remove {
	/// Server alias to remove
	#[arg()]
	server: String,
}

impl Remove {
	pub fn main(self) -> Result<()> {
		self.remove().desc("Failed to remove server")
	}

	fn remove(self) -> Result<()> {
		let mut servers = servers::read()?;

		if !servers.contains_key(&self.server) {
			bail!("Server with alias {} does not exist", self.server.bold());
		}

		servers.remove(&self.server);
		servers::write(&servers)?;

		racky_info!("Server {} removed successfully", self.server.bold(),);

		Ok(())
	}
}
