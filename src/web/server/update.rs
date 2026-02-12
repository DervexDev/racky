use axum::response::IntoResponse;
use log::{debug, error};
use tokio::task;

use crate::{installer, response};

pub async fn main() -> impl IntoResponse {
	match task::spawn_blocking(|| installer::update(false)).await {
		Ok(Ok(_)) => {
			debug!("Server has been updated");
			response!(
				OK,
				"Server updated successfully. Restart it for the changes to take effect"
			)
		}
		Ok(Err(err)) => {
			error!("Server failed to update: {err}");
			response!(INTERNAL_SERVER_ERROR, "{err}")
		}
		Err(err) => response!(INTERNAL_SERVER_ERROR, "{err}"),
	}
}
