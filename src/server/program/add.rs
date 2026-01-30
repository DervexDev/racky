use axum::{extract::Multipart, http::StatusCode, response::IntoResponse};

use crate::{dirs, ext::ResultExt, zip};

pub async fn main(mut multipart: Multipart) -> impl IntoResponse {
	let mut zip: Option<Vec<u8>> = None;

	while let Ok(Some(field)) = multipart.next_field().await {
		if field.name().unwrap_or_default() == "file"
			&& let Ok(data) = field.bytes().await
		{
			zip = Some(data.to_vec());
			break;
		}
	}

	let Some(zip) = zip else {
		return (StatusCode::BAD_REQUEST, "missing multipart field 'file'").into_response();
	};

	match zip::decompress(&zip, &dirs::bin()).desc("Failed to decompress zip file") {
		Ok(_) => (StatusCode::OK, format!("extracted {} bytes", zip.len())).into_response(),
		Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
	}
}
