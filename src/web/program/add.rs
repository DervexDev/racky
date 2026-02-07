use std::{collections::HashMap, fs};

use axum::{
	extract::{Multipart, State},
	response::IntoResponse,
};
use log::{trace, warn};

use crate::{
	core::{CorePtr, program::Program},
	dirs, response, zip,
};

pub async fn main(State(core): State<CorePtr>, mut multipart: Multipart) -> impl IntoResponse {
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

	if path.join(&name).exists() || path.join(format!("{name}.sh")).exists() {
		return response!(BAD_REQUEST, "Program {name} already exists");
	}

	match zip::decompress(&zip, &path) {
		Ok(()) => trace!("Decompressed {} bytes", zip.len()),
		Err(err) => return response!(INTERNAL_SERVER_ERROR, "Failed to decompress zipped program: {err}"),
	};

	let program = Program::new(&name);
	let total = settings.len();
	let mut failed = 0;

	for (key, value) in settings {
		if program.update_config(&key, &value).is_err() {
			failed += 1;
		}
	}

	let saved = program.save_config().is_ok();

	let mut message = if !saved {
		String::from(" but failed to create config file")
	} else if failed != 0 {
		format!(" but failed to load {failed} of {total} settings")
	} else {
		String::new()
	};

	match fs::create_dir_all(&program.paths().logs) {
		Ok(()) => trace!("Created logs directory for {name}"),
		Err(err) => {
			warn!("Failed to create logs directory for {name}: {err}");

			message.push_str(&format!(
				" {} failed to create logs directory",
				if message.is_empty() { "but" } else { "and" }
			));
		}
	}

	if !program.config().auto_start {
		return response!(OK, "Program {name} added successfully{message}");
	}

	let started = core
		.add_program(&program)
		.and_then(|_| core.start_program(&program))
		.is_ok();

	if !started || !message.is_empty() {
		message.push_str(". See server logs for more details!");
	}

	if started {
		response!(OK, "Program {name} added and started successfully{message}")
	} else {
		response!(
			OK,
			"Program {name} added successfully but failed to start{}",
			message.replace("but", "and")
		)
	}
}
