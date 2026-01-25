use std::{io::Result, net::TcpListener, sync::Arc};

use actix_web::{
	web::{self, Data},
	App, HttpServer, Responder,
};

use crate::core::Core;

pub struct Server {
	core: Arc<Core>,
	host: String,
	port: u16,
	password: Option<String>,
}

impl Server {
	pub fn new(core: Core, host: &str, port: u16, password: Option<String>) -> Self {
		Self {
			core: Arc::new(core),
			host: host.to_owned(),
			port,
			password,
		}
	}

	#[actix_web::main]
	pub async fn start(&self) -> Result<()> {
		let core = self.core.clone();

		HttpServer::new(move || {
			App::new()
				.app_data(Data::new(core.clone()))
				// .service(details::main)
				.default_service(web::to(Self::default_redirect))
		})
		.backlog(0)
		.disable_signals()
		.bind((self.host.clone(), self.port))?
		.run()
		.await
	}

	async fn default_redirect() -> impl Responder {
		web::Redirect::to("/")
	}

	pub fn is_port_free(&self) -> bool {
		TcpListener::bind((self.host.as_str(), self.port)).is_ok()
	}

	pub fn get_address(&self) -> String {
		format!("http://{}:{}", self.host, self.port)
	}
}
