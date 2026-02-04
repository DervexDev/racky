use std::fs;

use axum::{Form, extract::State, response::IntoResponse};

use crate::{
	core::{CorePtr, program::Paths},
	response,
	web::program::ProgramRequest,
};

pub async fn main(State(core): State<CorePtr>, Form(request): Form<ProgramRequest>) -> impl IntoResponse {
	let paths = Paths::from_name(&request.program);
	let executable = paths.get_program_root();

	let results = [
		if executable.is_dir() {
			(&executable, "program directory")
		} else {
			(&executable, "program file")
		},
		(&paths.config, "config file"),
		(&paths.logs, "logs directory"),
	]
	.into_iter()
	.filter_map(|(path, description)| {
		if !path.exists() {
			return None;
		}

		if path.is_dir() {
			Some((fs::remove_dir_all(path), description))
		} else {
			Some((fs::remove_file(path), description))
		}
	})
	.collect::<Vec<_>>();

	let errors = results
		.iter()
		.filter_map(|(result, description)| match result {
			Ok(_) => None,
			Err(_) => Some(*description),
		})
		.collect::<Vec<_>>();

	let mut message = if !errors.is_empty() {
		format!("Failed to remove {}", errors.join(" and "))
	} else {
		String::new()
	};

	if let Some(program) = core.get_program(&request.program) {
		core.remove_program(&program).expect("Failed to remove program");

		if program.is_active() && core.stop_program(&program).is_err() {
			message.push_str(" and failed to stop the process");
		}
	}

	if !message.is_empty() {
		message.push_str(". See server logs for more details!");
	}

	if results.is_empty() {
		response!(NOT_FOUND, "Program {} does not exist{message}", request.program)
	} else if !errors.is_empty() {
		response!(INTERNAL_SERVER_ERROR, format!("Failed to remove program: {message}"))
	} else {
		response!(
			OK,
			format!(
				"Program {} removed successfully{}",
				request.program,
				message.replace("and", "but")
			)
		)
	}
}
