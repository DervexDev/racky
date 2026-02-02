#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::{
	env,
	process::ExitStatus,
	thread,
	time::{Duration, SystemTime},
};

use anyhow::Result;
use env_logger::WriteStyle;
use jiff::Timestamp;
use log::LevelFilter;

use crate::ext::ResultExt;

/// Returns the `RUST_VERBOSE` environment variable
pub fn env_verbosity() -> LevelFilter {
	let verbosity = env::var("RUST_VERBOSE").unwrap_or("ERROR".into());

	match verbosity.as_str() {
		"OFF" => LevelFilter::Off,
		"ERROR" => LevelFilter::Error,
		"WARN" => LevelFilter::Warn,
		"INFO" => LevelFilter::Info,
		"DEBUG" => LevelFilter::Debug,
		"TRACE" => LevelFilter::Trace,
		_ => LevelFilter::Error,
	}
}

/// Returns the `RUST_LOG_STYLE` environment variable
pub fn env_log_style() -> WriteStyle {
	let log_style = env::var("RUST_LOG_STYLE").unwrap_or("auto".into());

	match log_style.as_str() {
		"always" => WriteStyle::Always,
		"never" => WriteStyle::Never,
		_ => WriteStyle::Auto,
	}
}

/// Returns the `RUST_BACKTRACE` environment variable
pub fn env_backtrace() -> bool {
	let backtrace = env::var("RUST_BACKTRACE").unwrap_or("0".into());
	backtrace == "1"
}

/// Returns the `RUST_YES` environment variable
pub fn env_yes() -> bool {
	let yes = env::var("RUST_YES").unwrap_or("0".into());
	yes == "1"
}

/// Returns the current timestamp in the `YYYY-MM-DD HH:MM:SS` format
pub fn timestamp() -> String {
	format!("{:.0}", Timestamp::try_from(SystemTime::now()).unwrap_or_default())
}

/// Returns the current user name
pub fn get_user() -> Result<String> {
	#[cfg(unix)]
	let result = env::var("SUDO_USER").or_else(|_| env::var("USER"));
	#[cfg(windows)]
	let result = env::var("USERNAME");

	result.desc("Failed to get current user")
}

pub fn get_exit_code(status: &ExitStatus) -> i32 {
	#[cfg(unix)]
	let code = status
		.code()
		.or_else(|| status.signal())
		.or_else(|| status.stopped_signal());

	#[cfg(windows)]
	let code = status.code();

	code.unwrap_or(-1)
}

/// Returns the service name for the current user
pub fn get_service() -> String {
	get_user()
		.map(|user| format!("racky-{user}"))
		.unwrap_or_else(|_| String::from("racky"))
}

/// Delays the execution of a function for a given number of seconds
pub fn delay<F: FnOnce() + Send + 'static>(seconds: u64, f: F) {
	thread::spawn(move || {
		thread::sleep(Duration::from_secs(seconds));
		f();
	});
}
