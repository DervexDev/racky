use axum::{Form, response::IntoResponse};
use documented::DocumentedFields;
use serde::Deserialize;

use crate::{
	core::program::{Config, Program},
	logger::Table,
	response,
};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Request {
	program: String,
	data: String,
	default: bool,
	list: bool,
}

pub async fn main(Form(request): Form<Request>) -> impl IntoResponse {
	let program = Program::new(&request.program);

	if !program.paths().validate() {
		return response!(NOT_FOUND, "Program {} does not exist", request.program);
	}

	program.load_config();

	let defaults = Config::default();
	let config = program.config();

	if request.list {
		let mut table = Table::new();
		let vars = &program.state().vars;
		let defaults_only = config == defaults && vars.is_empty();

		if defaults_only {
			table.set_header(vec!["Setting", "Default", "Description"]);
		} else {
			table.set_header(vec!["Setting", "Default", "Current", "Description"]);
		}

		for (setting, default) in &defaults {
			let doc = Config::get_field_docs(setting).unwrap_or_default().trim().to_owned();

			if defaults_only {
				table.add_row(vec![setting.to_owned(), default.to_string(), doc]);
				continue;
			}

			let default = default.to_string();
			let mut current = config.get(setting).map(|v| v.to_string()).unwrap();

			if current == default {
				current = String::new();
			}

			table.add_row(vec![setting.to_owned(), default, current, doc]);
		}

		for (key, value) in vars {
			table.add_row(vec![
				key.to_owned(),
				String::new(),
				value.to_owned(),
				String::from("User-defined program environment variable"),
			]);
		}

		return response!(OK, format!("Program configuration:\n{}", table));
	} else if request.default {
		let mut state = program.state_mut();
		state.config = Config::default();
		state.vars.clear();

		drop(state);

		return match program.save_config() {
			Ok(()) => response!(
				OK,
				"Configuration of {} restored to defaults successfully",
				request.program
			),
			Err(err) => response!(
				INTERNAL_SERVER_ERROR,
				"Failed to save {} configuration: {err}",
				request.program
			),
		};
	}

	if request.data.is_empty() {
		return response!(BAD_REQUEST, "No key=value pairs provided");
	}

	let mut changed = 0;

	for pair in request.data.split(',') {
		let (key, value) = if let Some(pair) = pair.split_once('=') {
			pair
		} else {
			return response!(BAD_REQUEST, "Invalid key=value or key= pair: {}", pair);
		};

		let original = if let Some(original) = config.get(key) {
			original
		} else {
			let original = program.state().vars.get(key).cloned();

			program.update_config(key, value).unwrap();

			if original.is_none() || original.unwrap() != value {
				changed += 1;
			}

			continue;
		};

		if value.is_empty() {
			program
				.update_config(key, &defaults.get(key).unwrap().to_string())
				.unwrap();
		} else if let Err(err) = program.update_config(key, value) {
			return response!(
				INTERNAL_SERVER_ERROR,
				"Failed to update {} configuration: {err}",
				request.program
			);
		}

		if program.config().get(key).unwrap() != original {
			changed += 1;
		}
	}

	match program.save_config() {
		Ok(()) => response!(
			OK,
			format!(
				"Configuration of {} updated successfully ({changed} changed)",
				request.program
			)
		),
		Err(err) => response!(
			INTERNAL_SERVER_ERROR,
			"Failed to save {} configuration: {err}",
			request.program
		),
	}
}
