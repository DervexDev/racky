use argon2::Argon2;
use axum::{
	body::Body,
	extract::State,
	http::{Request, StatusCode},
	middleware::Next,
	response::IntoResponse,
};
use password_hash::PasswordHash;

pub async fn main(
	State(password_hash): State<Option<String>>,
	request: Request<Body>,
	next: Next,
) -> impl IntoResponse {
	let auth = request
		.headers()
		.get("Authorization")
		.and_then(|v| v.to_str().ok())
		.unwrap_or_default();

	let token = auth
		.strip_prefix("Bearer ")
		.or_else(|| auth.strip_prefix("bearer "))
		.unwrap_or(auth)
		.trim();

	if let Some(ref hash) = password_hash {
		if token.is_empty() {
			return Err(StatusCode::UNAUTHORIZED);
		}
		let parsed = match PasswordHash::new(hash) {
			Ok(p) => p,
			Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
		};
		if parsed.verify_password(&[&Argon2::default()], token).is_err() {
			return Err(StatusCode::UNAUTHORIZED);
		}
	}

	Ok(next.run(request).await)
}
