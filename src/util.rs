use std::{env, time::SystemTime};

use env_logger::WriteStyle;
use jiff::Timestamp;
use log::LevelFilter;

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
