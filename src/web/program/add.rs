use std::collections::HashMap;

use axum::{
	Extension,
	extract::{Multipart, State},
	response::IntoResponse,
};
use colored::Colorize;
use log::{info, trace, warn};

use crate::{
	core::{CorePtr, program::Program},
	dirs, response, zip,
};

pub async fn main(
	State(core): State<CorePtr>,
	Extension(is_racky): Extension<bool>,
	mut multipart: Multipart,
) -> impl IntoResponse {
	let mut zip: Option<Vec<u8>> = None;
	let mut settings = HashMap::new();

	while let Ok(Some(field)) = multipart.next_field().await {
		let name = field.name().unwrap_or_default().to_owned();

		match name.as_str() {
			"file" => {
				if let Ok(data) = field.bytes().await {
					zip = Some(data.to_vec());
				}
			}
			_ => {
				if let Ok(value) = field.text().await {
					settings.insert(name, value);
				}
			}
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
	let name_pretty = if is_racky { name.bold() } else { name.normal() };

	if path.join(&name).exists() || path.join(format!("{name}.sh")).exists() {
		return response!(CONFLICT, "Program {name_pretty} already exists");
	}

	match zip::decompress(&zip, &path) {
		Ok(()) => trace!("Decompressed {} bytes", zip.len()),
		Err(err) => return response!(INTERNAL_SERVER_ERROR, "Failed to decompress zipped program: {err}"),
	};

	let program = Program::new(&name);
	let total = settings.len();
	let mut failed = 0;

	for (key, value) in settings {
		match program.update_config(&key, &value) {
			Ok(()) => trace!("Set `{key}` to `{value}` in {name} config"),
			Err(err) => {
				warn!("Failed to update {name} config: {err}");
				failed += 1;
			}
		}
	}

	let saved = match program.save_config() {
		Ok(()) => {
			info!("Created {name} config file");
			true
		}
		Err(err) => {
			warn!("Failed to create {name} config file: {err}");
			false
		}
	};

	let mut message = if !saved {
		String::from(" but failed to create config file")
	} else if failed != 0 {
		format!(" but failed to load {} of {} settings", failed, total)
	} else {
		String::new()
	};

	if !program.config().auto_start {
		return response!(OK, "Program {name_pretty} added successfully{message}");
	}

	let started = core.start_program(&program);

	if !started || !message.is_empty() {
		message += ". See server logs for more details";
	}

	if started {
		response!(OK, "Program {name_pretty} added and started successfully{message}")
	} else {
		response!(
			OK,
			"Program {name_pretty} added successfully but failed to start{}",
			message.replace("but", "and")
		)
	}
}
