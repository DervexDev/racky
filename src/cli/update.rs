use anyhow::Result;
use clap::Parser;

use crate::{ext::ResultExt, installer, racky_info};

/// Update Racky to the latest version
#[derive(Parser)]
pub struct Update {}

impl Update {
	pub fn main(self) -> Result<()> {
		installer::update(true)
			.desc("Failed to update Racky")
			.map(|_| racky_info!("Racky has been updated successfully"))
	}
}
