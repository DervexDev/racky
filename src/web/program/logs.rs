use axum::{extract::Query, response::IntoResponse};
use serde::Deserialize;

use crate::{logger, response};

#[derive(Debug, Deserialize)]
pub struct Request {
	program: String,
	page: Option<usize>,
}

pub async fn main(Query(request): Query<Request>) -> impl IntoResponse {
	match logger::read_file(&request.program, request.page.unwrap_or_default()) {
		Ok(logs) => response!(OK, logs),
		Err(error) => response!(
			INTERNAL_SERVER_ERROR,
			format!("Failed to get {} logs: {error}", request.program)
		),
	}
}
