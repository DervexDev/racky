use std::{
	env,
	io::{self, IsTerminal},
	process::ExitCode,
};

use env_logger::WriteStyle;
use log::{error, info};
use racky::{cli::Cli, config::Config, logger, racky_error};

fn main() -> ExitCode {
	let cli = Cli::new();

	let verbosity = cli.verbosity();
	let log_style = cli.log_style();

	unsafe {
		if log_style == WriteStyle::Auto && io::stdin().is_terminal() {
			env::set_var("RUST_LOG_STYLE", "always");
		} else {
			env::set_var(
				"RUST_LOG_STYLE",
				match log_style {
					WriteStyle::Always => "always",
					_ => "never",
				},
			)
		}

		env::set_var("RUST_VERBOSE", verbosity.as_str());
		env::set_var("RUST_YES", if cli.yes() { "1" } else { "0" });
		env::set_var("RUST_BACKTRACE", if cli.backtrace() { "1" } else { "0" });
	}

	logger::init(verbosity, log_style);

	match Config::load() {
		Ok(()) => info!("Racky config loaded successfully"),
		Err(err) => error!("Failed to load config: {err}"),
	}

	match cli.main() {
		Ok(()) => ExitCode::SUCCESS,
		Err(err) => {
			racky_error!("{err}");
			ExitCode::FAILURE
		}
	}
}
