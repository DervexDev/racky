use std::{borrow::Cow, collections::HashMap, fmt::Display};

use anyhow::Result;
use reqwest::{
	StatusCode,
	blocking::{
		Client as ReqwestClient,
		multipart::{Form, Part},
	},
};

use crate::{constants::USER_AGENT, ext::ResultExt, servers::Server};

#[derive(Debug)]
pub enum Field {
	Text(String),
	File(Vec<u8>),
}

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

	pub fn post(self, path: &str) -> Result<(StatusCode, String)> {
		let mut request = ReqwestClient::builder()
			.build()
			.desc("Failed to create HTTP client")?
			.post(format!("http://{}:{}/{path}", self.address, self.port))
			.header("User-Agent", USER_AGENT);

		if !self.fields.is_empty() {
			let mut form = Form::new();

			for (key, value) in self.fields {
				match value {
					Field::Text(value) => form = form.text(key, value),
					Field::File(value) => form = form.part(key, Part::bytes(value)),
				}
			}

			request = request.multipart(form);
		}

		if let Some(password) = &self.password {
			request = request.header("Authorization", password);
		}

		let response = request.send().desc("Failed to connect to the server")?;

		Ok((response.status(), response.text().unwrap_or_default()))
	}
}
