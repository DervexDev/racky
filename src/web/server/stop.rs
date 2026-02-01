use std::{
	env,
	process::{self},
};

use axum::response::IntoResponse;
use log::{debug, error};

use crate::{command::Command, response, util};

pub async fn main() -> impl IntoResponse {
	debug!("Server stopping in 1 second...");

	util::delay(1, || {
		if cfg!(target_os = "linux") && env::var("INVOCATION_ID").is_ok() {
			let service = util::get_service();

			if let Err(err) = Command::new("systemctl").args(["stop", &service]).run() {
				error!("Failed to run `systemctl stop {service}` command: {err}");
			}
		} else {
			process::exit(0);
		}
	});

	response!(OK, "Server stopping in 1 second...")
}
