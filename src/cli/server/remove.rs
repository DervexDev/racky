use anyhow::{Result, bail};
use clap::Parser;
use colored::Colorize;

use crate::{
	cli::server::{read_servers, write_servers},
	ext::ResultExt,
	racky_info,
};

/// Remove the existing server
#[derive(Parser)]
pub struct Remove {
	/// Server alias to remove
	#[arg()]
	alias: String,
}

impl Remove {
	pub fn main(self) -> Result<()> {
		self.remove().desc("Failed to remove server")
	}

	fn remove(self) -> Result<()> {
		let mut servers = read_servers()?;

		if !servers.contains_key(&self.alias) {
			bail!("Server with alias {} does not exist", self.alias.bold());
		}

		servers.remove(&self.alias);
		write_servers(&servers)?;

		racky_info!("Server {} removed successfully", self.alias.bold(),);

		Ok(())
	}
}
