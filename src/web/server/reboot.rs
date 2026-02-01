use axum::response::IntoResponse;
use log::{debug, error};

use crate::{command::Command, ext::ResultExt, response};

pub async fn main() -> impl IntoResponse {
	if cfg!(windows) {
		return response!(
			BAD_REQUEST,
			"Rebooting the server is currently only supported on Unix systems!"
		);
	}

	match Command::new("reboot").run().desc("Failed to run `reboot` command") {
		Ok(output) => {
			debug!("Server will reboot shortly: {}", output);
			response!(OK, "Server will reboot shortly")
		}
		Err(err) => {
			error!("{err}");
			response!(INTERNAL_SERVER_ERROR, "{err}")
		}
	}
}
