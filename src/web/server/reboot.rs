use axum::response::IntoResponse;
use log::{debug, error};

use crate::{command::Command, ext::ResultExt, response};

pub async fn main() -> impl IntoResponse {
	#[cfg(unix)]
	let result = Command::new("shutdown")
		.args(["-r", "now"])
		.run()
		.desc("Failed to run `shutdown -r now` command");

	#[cfg(windows)]
	let result = Command::new("shutdown")
		.args(["/r", "/t", "0"])
		.run()
		.desc("Failed to run `shutdown /r /t 0` command");

	match result {
		Ok(_) => {
			debug!("Server rebooting...");
			response!(OK, "Server rebooting...")
		}
		Err(err) => {
			error!("Server failed to reboot: {err}");
			response!(INTERNAL_SERVER_ERROR, "{err}")
		}
	}
}
