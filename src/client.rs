use std::{borrow::Cow, collections::HashMap, fmt::Display};

use anyhow::{Result, bail};
use reqwest::{
	StatusCode,
	blocking::{
		Client as ReqwestClient,
		multipart::{Form, Part},
	},
};

use crate::{constants::USER_AGENT, ext::ResultExt, racky_info, servers::Server};

#[derive(Debug)]
pub struct Client {
	address: String,
	port: u16,
	password: Option<String>,
	fields: HashMap<Cow<'static, str>, Field>,
}

impl Client {
	pub fn new(server: &Server) -> Self {
		Self {
			address: server.address.clone(),
			port: server.port,
			password: if server.password.is_empty() {
				None
			} else {
				Some(server.password.clone())
			},
			fields: HashMap::new(),
		}
	}

	pub fn text<K, V>(mut self, key: K, value: V) -> Self
	where
		K: Into<Cow<'static, str>>,
		V: Display,
	{
		self.fields.insert(key.into(), Field::Text(value.to_string()));
		self
	}

	pub fn file<K, V>(mut self, key: K, value: V) -> Self
	where
		K: Into<Cow<'static, str>>,
		V: AsRef<[u8]>,
	{
		self.fields.insert(key.into(), Field::File(value.as_ref().to_vec()));
		self
	}

	pub fn post(self, path: &str) -> Result<Response> {
		let mut request = ReqwestClient::builder()
			.build()
			.desc("Failed to create HTTP client")?
			.post(format!("http://{}:{}/{path}", self.address, self.port))
			.header("User-Agent", USER_AGENT);

		if !self.fields.is_empty() {
			if self.fields.values().any(|field| matches!(field, Field::File(_))) {
				let mut form = Form::new();

				for (key, value) in self.fields {
					match value {
						Field::Text(value) => form = form.text(key, value),
						Field::File(value) => form = form.part(key, Part::bytes(value)),
					}
				}

				request = request.multipart(form);
			} else {
				let fields = self
					.fields
					.iter()
					.map(|(key, value)| {
						(
							key.clone(),
							match value {
								Field::Text(value) => value.to_owned(),
								Field::File(_) => unreachable!(),
							},
						)
					})
					.collect::<HashMap<_, _>>();

				request = request.form(&fields);
			}
		}

		if let Some(password) = &self.password {
			request = request.header("Authorization", password);
		}

		let response = request.send().desc("Failed to connect to the server")?;

		Ok(Response(response.status(), response.text().unwrap_or_default()))
	}
}

#[derive(Debug)]
pub struct Response(pub StatusCode, pub String);

impl Response {
	pub fn handle(self) -> Result<()> {
		if self.0.is_success() {
			racky_info!("{}", self.1);
		} else {
			bail!("{} ({})", self.1, self.0);
		}

		Ok(())
	}
}

#[derive(Debug)]
enum Field {
	Text(String),
	File(Vec<u8>),
}
