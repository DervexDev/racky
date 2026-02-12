use anyhow::Result;
use clap::Parser;

use crate::{ext::ResultExt, installer, logger, racky_info};

/// Uninstall Racky and all its contents
#[derive(Parser)]
pub struct Uninstall {}

impl Uninstall {
	pub fn main(self) -> Result<()> {
		if !logger::prompt(
			"Are you sure you want to uninstall Racky? All programs, configs and logs it stores will be permanently deleted!",
			false,
		) {
			return Ok(());
		}

		installer::uninstall()
			.desc("Failed to uninstall Racky")
			.map(|_| racky_info!("Racky has been uninstalled successfully"))
	}
}
