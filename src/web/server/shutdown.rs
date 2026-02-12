use axum::response::IntoResponse;
use log::{debug, error};

use crate::{command::Command, ext::ResultExt, response};

pub async fn main() -> impl IntoResponse {
	#[cfg(unix)]
	let result = Command::new("shutdown")
		.arg("now")
		.run()
		.desc("Failed to run `shutdown now` command");

	#[cfg(windows)]
	let result = Command::new("shutdown")
		.args(["/s", "/t", "0"])
		.run()
		.desc("Failed to run `shutdown /s /t 0` command");

	match result {
		Ok(_) => {
			debug!("Server shutting down...");
			response!(OK, "Server shutting down...")
		}
		Err(err) => {
			error!("Server failed to shutdown: {err}");
			response!(INTERNAL_SERVER_ERROR, "{err}")
		}
	}
}
