use std::{io::Result, net::TcpListener, sync::Arc};

use axum::{
	Router,
	middleware::from_fn_with_state,
	routing::{get, post},
};
use tokio::net;

use crate::{constants::BODY_SIZE_LIMIT, core::Core};

mod middleware;
mod program;
mod root;

pub struct Server {
	router: Router,
	address: String,
	port: u16,
}

impl Server {
	pub fn new(core: Core, address: &str, port: u16, password: Option<String>) -> Self {
		let router = Router::new()
			.route("/", get(root::main))
			.route("/program/add", post(program::add::main).layer(BODY_SIZE_LIMIT))
			.layer(from_fn_with_state(password, middleware::auth::main))
			.with_state(Arc::new(core));

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
