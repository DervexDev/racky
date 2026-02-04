use axum::{Form, extract::State, response::IntoResponse};

use crate::{
	core::{CorePtr, program::Program},
	response,
	web::program::ProgramRequest,
};

pub async fn main(State(core): State<CorePtr>, Form(request): Form<ProgramRequest>) -> impl IntoResponse {
	let (program, is_new) = if let Some(program) = core.get_program(&request.program) {
		if program.is_active() {
			return response!(BAD_REQUEST, "Program {} is already running", request.program);
		} else {
			(program, false)
		}
	} else {
		(Program::new(&request.program), true)
	};

	if !program.paths().validate() {
		return response!(NOT_FOUND, "Program {} does not exist", request.program);
	}

	if is_new {
		core.add_program(&program).expect("Failed to add program");
	}

	match core.start_program(&program) {
		Ok(()) => response!(OK, "Program {} started successfully", request.program),
		Err(err) => response!(
			INTERNAL_SERVER_ERROR,
			"Failed to start program {}: {err}",
			request.program
		),
	}
}
