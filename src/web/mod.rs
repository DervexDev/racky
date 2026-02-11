use std::{io::Result, net::TcpListener};

use axum::{
	Router,
	middleware::from_fn_with_state,
	response::Redirect,
	routing::{get, post},
};
use tokio::net;

use crate::{consts::BODY_SIZE_LIMIT, core::CorePtr};

mod middleware;
mod ping;
mod program;
mod server;

#[macro_export]
macro_rules! response {
	($status:ident) => {
		axum::http::StatusCode::$status.into_response()
	};
	($status:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {
		(axum::http::StatusCode::$status, format!($fmt $(, $arg)*)).into_response()
	};
	($status:ident, $body:expr) => {
		(axum::http::StatusCode::$status, $body).into_response()
	};
}

pub struct Web {
	router: Router,
	address: String,
	port: u16,
}

impl Web {
	pub fn new(core: CorePtr, address: &str, port: u16, password: Option<String>) -> Self {
		let router = Router::new()
			.route("/", get(|| async { Redirect::to("/server/status") }))
			.route("/ping", get(ping::main))
			// Program routes
			.route("/program/add", post(program::add::main).layer(BODY_SIZE_LIMIT))
			.route("/program/config", post(program::config::main))
			.route("/program/logs", get(program::logs::main))
			.route("/program/remove", post(program::remove::main))
			.route("/program/restart", post(program::restart::main))
			.route("/program/start", post(program::start::main))
			.route("/program/status", get(program::status::main))
			.route("/program/stop", post(program::stop::main))
			.route("/program/update", post(program::update::main).layer(BODY_SIZE_LIMIT))
			// Server routes
			.route("/server/config", post(server::config::main))
			.route("/server/logs", get(server::logs::main))
			.route("/server/reboot", post(server::reboot::main))
			.route("/server/restart", post(server::restart::main))
			.route("/server/shutdown", post(server::shutdown::main))
			.route("/server/status", get(server::status::main))
			.route("/server/stop", post(server::stop::main))
			// Middleware
			.layer(from_fn_with_state(password, middleware::auth::main))
			.with_state(core);

		Self {
			router,
			address: address.to_owned(),
			port,
		}
	}

	#[tokio::main]
	pub async fn start(self) -> Result<()> {
		axum::serve(
			net::TcpListener::bind((self.address.as_str(), self.port)).await?,
			self.router,
		)
		.await
	}

	pub fn is_port_free(&self) -> bool {
		TcpListener::bind((self.address.as_str(), self.port)).is_ok()
	}
}
