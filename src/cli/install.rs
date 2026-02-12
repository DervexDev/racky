use anyhow::Result;
use clap::Parser;

use crate::{ext::ResultExt, installer, racky_info};

/// Install and/or verify Racky installation
#[derive(Parser)]
pub struct Install {
	/// Install Racky server (requires sudo)
	#[arg(short, long)]
	server: bool,
	/// Overwrite all files even if they already exist (including config)
	#[arg(short, long)]
	force: bool,
}

impl Install {
	pub fn main(self) -> Result<()> {
		let side = if self.server { "server" } else { "client" };

		installer::install(self.server, self.force)
			.with_desc(|| format!("Failed to install Racky {side}"))
			.map(|_| racky_info!("Racky {side} has been installed successfully"))
	}
}
