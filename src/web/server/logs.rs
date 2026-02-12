use axum::{extract::Query, response::IntoResponse};
use serde::Deserialize;

use crate::{dirs, logger, response};

#[derive(Debug, Deserialize)]
pub struct Request {
	page: Option<usize>,
}

pub async fn main(Query(request): Query<Request>) -> impl IntoResponse {
	match logger::read_file(&dirs::logs().join("racky"), request.page.unwrap_or_default()) {
		Ok(logs) => response!(OK, logs),
		Err(err) => response!(BAD_REQUEST, "{err}"),
	}
}
