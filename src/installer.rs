use std::{env, fs, path::Path};

use anyhow::{Result, bail};
use log::trace;
use self_update::{backends::github::Update, version::bump_is_greater};

use crate::{
	command::Command,
	config::Config,
	dirs,
	ext::{PathExt, ResultExt},
	racky_error, racky_warn, util,
};

const SYSTEMD_SERVICE: &str = "[Unit]\n\
Description=Racky Server ($1)\n\
After=network.target\n\
\n\
[Service]\n\
ExecStart=/home/$1/.racky/bin/racky server start -vvvv -y\n\
Environment=HOME=/home/$1 SUDO_USER=$1\n\
Restart=always\n\
\n\
[Install]\n\
WantedBy=default.target\n";

pub fn install(server: bool, force: bool) -> Result<()> {
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

	if (!config_path.exists() || force)
		&& let Err(err) = Config::default().save()
	{
		racky_warn!("Failed to create config file at {config_path:?}: {err}");
	}

	let current_exe = env::current_exe()?;
	#[cfg(windows)]
	let exe_path = bin_dir.join("racky.exe");
	#[cfg(unix)]
	let exe_path = bin_dir.join("racky");

	if (!exe_path.exists() || force) && current_exe.resolve()? != exe_path.resolve()? {
		fs::copy(&current_exe, &exe_path).desc("Failed to copy Racky executable to bin directory")?;
	}

	if !server {
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

	if !service_path.exists() || force {
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

pub fn uninstall() -> Result<()> {
	fs::remove_dir_all(dirs::racky())
		.desc("Failed to remove `.racky` directory")
		.and_then(|_| globenv::remove_path(&dirs::bin().to_string()).desc("Failed to update PATH environment variable"))
}

pub fn update(show_progress: bool) -> Result<()> {
	let version = env!("CARGO_PKG_VERSION");
	let update = Update::configure()
		.current_version(version)
		.repo_owner("DervexDev")
		.repo_name("racky")
		.bin_name("racky")
		.auth_token(&env::var("GITHUB_TOKEN").unwrap_or_default())
		.show_download_progress(show_progress)
		.show_output(false)
		.no_confirm(true)
		.build()?;

	let release = update
		.get_latest_release()
		.desc("Failed to get latest release details")?;

	if !bump_is_greater(version, &release.version)? {
		bail!("Already up to date");
	}

	update.update().desc("Failed to download update").map(drop)
}
