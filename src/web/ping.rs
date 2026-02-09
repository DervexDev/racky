use axum::response::IntoResponse;

use crate::response;

pub async fn main() -> impl IntoResponse {
	response!(OK, "pong")
}
