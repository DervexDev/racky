use axum::{body::Body, http::Request, middleware::Next};

use crate::constants::USER_AGENT;

pub async fn main(mut request: Request<Body>, next: Next) -> axum::response::Response {
	let is_racky = request
		.headers()
		.get("User-Agent")
		.is_some_and(|v| v.as_bytes() == USER_AGENT.as_bytes());

	request.extensions_mut().insert(is_racky);
	next.run(request).await
}
