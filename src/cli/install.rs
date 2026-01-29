use std::{env, fs, process::Command};

use anyhow::{Result, bail};
use clap::Parser;

use crate::{
	dirs,
	ext::{PathExt, ResultExt},
	logger, racky_error, racky_info, racky_warn,
};

const SYSTEMD_SERVICE: &str = "[Unit]\n\
Description=Racky\n\
After=network.target\n\
\n\
[Service]\n\
ExecStart=/home/$1/.racky/bin/racky start --yes\n\
Environment=HOME=/home/$1\n\
Restart=always\n\
\n\
[Install]\n\
WantedBy=default.target\n";

/// Install and/or verify Racky installation
#[derive(Parser)]
pub struct Install {
	/// Whether to install the server side of Racky
	#[arg(short, long)]
	server: bool,
}

impl Install {
	pub fn main(self) -> Result<()> {
		let side = if self.server { "server" } else { "client" };

		self.install().desc("Failed to install Racky {side}")?;
		racky_info!("Racky {side} has been installed successfully!");

		Ok(())
	}

	fn install(self) -> Result<()> {
		let bin_dir = dirs::bin();
		let config_dir = dirs::config();

		if !bin_dir.exists() {
			fs::create_dir_all(&bin_dir).desc("Failed to create bin directory")?;
		}

		if !config_dir.exists() {
			fs::create_dir_all(&config_dir).desc("Failed to create config directory")?;
		}

		if let Err(err) = globenv::set_path(&bin_dir.to_string()) {
			racky_error!(
				"Failed to update PATH environment variable: {err}. You might not be able to use Racky from your shell!"
			);
		}

		let config_path = config_dir.join("racky.toml");

		if !config_path.exists()
			&& let Err(err) = fs::write(&config_path, "")
		{
			racky_warn!("Failed to create config file at {}: {err}", config_path.to_string());
		}

		#[cfg(target_os = "windows")]
		let exe_path = bin_dir.join("racky.exe");
		#[cfg(not(target_os = "windows"))]
		let exe_path = bin_dir.join("racky");

		if !exe_path.exists() {
			fs::copy(env::current_exe()?, &exe_path).desc("Failed to copy Racky executable to bin directory")?;

			if logger::prompt("Installation completed! Do you want to remove this executable?", true) {
				self_replace::self_delete()?;
			}
		}

		if !self.server {
			return Ok(());
		} else if !cfg!(target_os = "linux") {
			bail!("Racky server is currently only supported on Linux!");
		}

		let service_dir = dirs::home().join(".config/systemd/user");
		let service_path = service_dir.join("racky.service");

		if !service_dir.exists() {
			fs::create_dir_all(&service_dir).desc("Failed to create service directory")?;
		}

		if !service_path.exists() {
			fs::write(
				&service_path,
				SYSTEMD_SERVICE.replace("$1", &env::var("USER").desc("Failed to get current user")?),
			)
			.desc("Failed to create service file")?;
		}

		if let Err(err) = Command::new("systemctl").args(["--user", "enable", "racky"]).spawn() {
			racky_error!("Failed to enable Racky service: {err}! Try running `systemctl --user enable racky` manually");
		}

		if let Err(err) = Command::new("systemctl").args(["--user", "start", "racky"]).spawn() {
			racky_error!("Failed to start Racky service: {err}! Try running `systemctl --user start racky` manually");
		}

		Ok(())
	}
}
