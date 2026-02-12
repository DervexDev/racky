use axum::response::IntoResponse;
use log::{debug, error};

use crate::{command::Command, response, util};

pub async fn main() -> impl IntoResponse {
	if !util::is_service() {
		return response!(
			BAD_REQUEST,
			"Restarting the server is currently only supported on Linux systems running Racky as a service!"
		);
	}

	debug!("Server restarting in 1 second...");

	util::delay(1, || {
		let service = util::get_service();

		if let Err(err) = Command::new("systemctl").args(["restart", &service]).run() {
			error!("Failed to run `systemctl restart {service}` command: {err}")
		}
	});

	response!(OK, "Server restarting in 1 second...")
}
