use std::fs;

use axum::{extract::State, response::IntoResponse};
use jiff::SignedDuration;
use log::error;

use crate::{
	core::{CorePtr, program::Program},
	dirs,
	ext::PathExt,
	logger::Table,
	response, util,
};

pub async fn main(State(core): State<CorePtr>) -> impl IntoResponse {
	let mut programs = core.programs().clone();

	let entries = match fs::read_dir(dirs::bin()) {
		Ok(entries) => entries,
		Err(err) => return response!(INTERNAL_SERVER_ERROR, "Failed to read bin directory: {err}"),
	};

	for entry in entries {
		if let Err(err) = entry {
			error!("Failed to check program binary: {err}");
			continue;
		}

		let path = entry.unwrap().path();
		let stem = path.get_stem();

		if stem == "racky" {
			continue;
		}

		let program = Program::new(stem);

		if programs.contains_key(program.name()) || !program.paths().validate() {
			continue;
		}

		programs.insert(program.name().to_owned(), program);
	}

	if programs.is_empty() {
		return response!(NOT_FOUND, "There are no installed programs on the server");
	}

	let mut table = Table::new();
	table.set_header(vec!["Name", "Status", "Executions", "Runtime", "Start Time"]);

	for (name, program) in programs.iter() {
		let state = program.state();
		let runtime = state.get_runtime();

		table.add_row(vec![
			name.to_owned(),
			state.status.to_string(),
			state.executions.to_string(),
			format!("{:#}", SignedDuration::from_secs(runtime.current.as_secs() as i64)),
			state
				.start_time
				.current
				.map(|time| util::timestamp(Some(time)))
				.unwrap_or_else(|| String::from("N/A")),
		]);
	}

	response!(OK, table.to_string())
}
