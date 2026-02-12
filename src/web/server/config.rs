use axum::{Form, response::IntoResponse};
use serde::Deserialize;

use crate::{config::Config, response};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Request {
	data: String,
	default: bool,
	list: bool,
}

pub async fn main(Form(request): Form<Request>) -> impl IntoResponse {
	let result = Config::new_mut().apply_user_data(
		request.data.split(',').map(|s| s.to_string()).collect(),
		request.default,
		request.list,
	);

	match result {
		Ok(message) => response!(OK, message),
		Err(err) => response!(INTERNAL_SERVER_ERROR, "{err}"),
	}
}
