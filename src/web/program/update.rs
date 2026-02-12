use std::fs;

use axum::{extract::Multipart, response::IntoResponse};
use log::trace;

use crate::{dirs, response, zip};

pub async fn main(mut multipart: Multipart) -> impl IntoResponse {
	let mut zip: Option<Vec<u8>> = None;

	while let Ok(Some(field)) = multipart.next_field().await {
		if let Some(name) = field.name()
			&& name == "file"
			&& let Ok(data) = field.bytes().await
		{
			zip = Some(data.to_vec());
		}
	}

	let Some(zip) = zip else {
		return response!(BAD_REQUEST, "missing multipart field `file` (program)");
	};

	let path = dirs::bin();
	let name = match zip::get_root_name(&zip) {
		Ok(name) => name,
		Err(err) => return response!(INTERNAL_SERVER_ERROR, "Failed to get name of zipped program: {err}"),
	};

	let mut current_path = path.join(&name);

	if !current_path.exists() {
		current_path = path.join(format!("{name}.sh"));

		if !current_path.exists() {
			return response!(NOT_FOUND, "Program {name} does not exist");
		}
	}

	match if current_path.is_dir() {
		fs::remove_dir_all(&current_path)
	} else {
		fs::remove_file(&current_path)
	} {
		Ok(()) => trace!("Removed {name} program binary"),
		Err(err) => return response!(INTERNAL_SERVER_ERROR, "Failed to remove {name} program binary: {err}"),
	}

	match zip::decompress(&zip, &path) {
		Ok(()) => trace!("Decompressed {} bytes", zip.len()),
		Err(err) => return response!(INTERNAL_SERVER_ERROR, "Failed to decompress zipped program: {err}"),
	};

	response!(
		OK,
		"Program {name} updated successfully. Restart it for the changes to take effect"
	)
}
