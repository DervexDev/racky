use axum::{
	body::Body,
	extract::State,
	http::{Request, StatusCode},
	middleware::Next,
	response::IntoResponse,
};

pub async fn main(State(password): State<Option<String>>, request: Request<Body>, next: Next) -> impl IntoResponse {
	let auth = request
		.headers()
		.get("Authorization")
		.and_then(|v| v.to_str().ok())
		.unwrap_or_default();

	if let Some(password) = password
		&& auth != password
	{
		return Err(StatusCode::UNAUTHORIZED);
	}

	Ok(next.run(request).await)
}
