use anyhow::{Result, ensure};
use clap::Parser;

use crate::{ext::ResultExt, logger::Table, racky_info, servers};

/// List all saved servers
#[derive(Parser)]
pub struct List {}

impl List {
	pub fn main(self) -> Result<()> {
		self.list().desc("Failed to list servers")
	}

	fn list(self) -> Result<()> {
		let servers = servers::read()?;

		ensure!(!servers.is_empty(), "There are no saved Racky servers");

		let mut table = Table::new();
		table.set_header(vec!["Alias", "Address", "Port", "Password", "Default"]);

		for (alias, server) in servers {
			table.add_row(vec![
				alias,
				server.address,
				server.port.to_string(),
				server.password,
				server.default.to_string(),
			]);
		}

		racky_info!("Saved Racky servers:\n{table}");

		Ok(())
	}
}
