use axum::{
	extract::{Query, State},
	response::IntoResponse,
};
use jiff::SignedDuration;

use crate::{core::CorePtr, response, util, web::program::ProgramRequest};

pub async fn main(State(core): State<CorePtr>, Query(request): Query<ProgramRequest>) -> impl IntoResponse {
	let program = if let Some(program) = core.get_program(&request.program) {
		program
	} else {
		return response!(
			NOT_FOUND,
			"Program {} has not been run since the server was started",
			request.program
		);
	};

	let state = program.state();
	let config = program.config();
	let runtime = state.get_runtime();

	let mut response = format!("Name: {}\n", request.program);
	response.push_str(&format!("Status: {}\n", state.status));
	response.push_str(&format!("Executions: {}\n", state.executions));
	response.push('\n');

	response.push_str("Current:\n");
	response.push_str(&format!(
		"  Restart Attempts: {}/{}\n",
		state.attempts.current, config.restart_attempts
	));
	response.push_str(&format!(
		"  Runtime: {:#}\n",
		SignedDuration::from_secs(runtime.current.as_secs() as i64)
	));
	response.push_str(&format!(
		"  Start Time: {}\n",
		state
			.start_time
			.current
			.map(|time| util::timestamp(Some(time)))
			.unwrap_or_else(|| String::from("N/A"))
	));
	response.push('\n');

	response.push_str("Total:\n");
	response.push_str(&format!("  Restart Attempts: {}\n", state.attempts.total));
	response.push_str(&format!(
		"  Runtime: {:#}\n",
		SignedDuration::from_secs(runtime.total.as_secs() as i64)
	));
	response.push_str(&format!(
		"  Start Time: {}\n",
		state
			.start_time
			.total
			.map(|time| util::timestamp(Some(time)))
			.unwrap_or_else(|| String::from("N/A"))
	));

	response!(OK, response)
}
