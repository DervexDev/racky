use axum::{Form, extract::State, response::IntoResponse};

use crate::{core::CorePtr, response, web::program::ProgramRequest};

pub async fn main(State(core): State<CorePtr>, Form(request): Form<ProgramRequest>) -> impl IntoResponse {
	let program = if let Some(program) = core.get_program(&request.program)
		&& program.is_active()
	{
		program
	} else {
		return response!(BAD_REQUEST, "Program {} is not running", request.program);
	};

	match core.stop_program(&program) {
		Ok(()) => response!(OK, "Program {} stopped successfully", request.program),
		Err(err) => response!(
			INTERNAL_SERVER_ERROR,
			"Failed to stop program {}: {err}",
			request.program
		),
	}
}
