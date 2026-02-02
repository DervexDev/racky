use std::{env, fs, path::Path};

use anyhow::{Result, bail};
use clap::Parser;
use log::trace;

use crate::{
	command::Command,
	config::Config,
	dirs,
	ext::{PathExt, ResultExt},
	logger, racky_error, racky_info, racky_warn, util,
};

const SYSTEMD_SERVICE: &str = "[Unit]\n\
Description=Racky ($1)\n\
After=network.target\n\
\n\
[Service]\n\
ExecStart=/home/$1/.racky/bin/racky server start -vvvv -y\n\
Environment=HOME=/home/$1 SUDO_USER=$1\n\
Restart=always\n\
\n\
[Install]\n\
WantedBy=default.target\n";

/// Install and/or verify Racky installation
#[derive(Parser)]
pub struct Install {
	/// Install the server side of Racky (requires sudo)
	#[arg(short, long)]
	server: bool,
	/// Rewrite all relevant files even if they already exist
	#[arg(short, long)]
	force: bool,
}

impl Install {
	pub fn main(self) -> Result<()> {
		let side = if self.server { "server" } else { "client" };

		self.install().with_desc(|| format!("Failed to install Racky {side}"))?;
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
			&& let Err(err) = Config::default().save()
		{
			racky_warn!("Failed to create config file at {config_path:?}: {err}");
		}

		#[cfg(windows)]
		let exe_path = bin_dir.join("racky.exe");
		#[cfg(unix)]
		let exe_path = bin_dir.join("racky");

		if !exe_path.exists() || self.force {
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

		let service_name = util::get_service();
		let service_dir = Path::new("/etc/systemd/system");
		let service_path = service_dir.join(format!("{service_name}.service"));

		if !service_dir.exists() {
			fs::create_dir_all(service_dir).desc("Failed to create service directory")?;
		}

		if !service_path.exists() || self.force {
			fs::write(&service_path, SYSTEMD_SERVICE.replace("$1", &util::get_user()?))
				.desc("Failed to create service file")?;
		}

		match Command::new("systemctl").args(["enable", &service_name]).run() {
			Ok(output) => trace!("Racky service enabled successfully: {output}"),
			Err(err) => {
				racky_error!(
					"Failed to enable Racky service: {err}! Try running `sudo systemctl enable {service_name}` manually"
				)
			}
		}

		match Command::new("systemctl").args(["start", &service_name]).run() {
			Ok(output) => trace!("Racky service started successfully: {output}"),
			Err(err) => {
				racky_error!(
					"Failed to start Racky service: {err}! Try running `sudo systemctl start {service_name}` manually"
				)
			}
		}

		Ok(())
	}
}
