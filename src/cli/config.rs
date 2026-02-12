use anyhow::Result;
use clap::Parser;

use crate::{config::Config as RackyConfig, ext::ResultExt, racky_info};

/// Update Racky configuration
#[derive(Parser)]
pub struct Config {
	/// Key=Value pairs to update
	#[arg()]
	data: Vec<String>,
	/// Restore all settings to their default values
	#[arg(short, long)]
	default: bool,
	/// List current configuration
	#[arg(short, long)]
	list: bool,
}

impl Config {
	pub fn main(self) -> Result<()> {
		RackyConfig::new_mut()
			.apply_user_data(self.data, self.default, self.list)
			.desc("Failed to update/list Racky config")
			.map(|message| racky_info!("{message}"))
	}
}
