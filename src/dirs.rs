use std::path::PathBuf;

use directories::UserDirs;
use lazy_static::lazy_static;

lazy_static! {
	static ref HOME_DIR: PathBuf = UserDirs::new()
		.expect("Failed to get home directory which is required for Racky to work")
		.home_dir()
		.to_owned();
}

#[inline]
pub fn home() -> PathBuf {
	HOME_DIR.clone()
}

#[inline]
pub fn racky() -> PathBuf {
	HOME_DIR.join(".racky")
}

#[inline]
pub fn bin() -> PathBuf {
	racky().join("bin")
}

#[inline]
pub fn config() -> PathBuf {
	racky().join("config")
}

#[inline]
pub fn logs() -> PathBuf {
	racky().join("logs")
}
