use axum::{extract::Query, response::IntoResponse};
use serde::Deserialize;

use crate::{dirs, logger, response};

#[derive(Debug, Deserialize)]
pub struct Request {
	program: String,
	page: Option<usize>,
}

pub async fn main(Query(request): Query<Request>) -> impl IntoResponse {
	let path = dirs::logs().join(&request.program);

	if !path.exists() {
		return response!(NOT_FOUND, "Program {} does not exist", request.program);
	}

	match logger::read_file(&path, request.page.unwrap_or_default()) {
		Ok(logs) => response!(OK, logs),
		Err(error) => response!(BAD_REQUEST, format!("Failed to get {} logs: {error}", request.program)),
	}
}
