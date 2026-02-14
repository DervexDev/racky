use std::{borrow::Cow, collections::HashMap, fmt::Display};

use anyhow::{Result, bail};
use reqwest::{
	StatusCode,
	blocking::{
		Client as ReqwestClient, RequestBuilder,
		multipart::{Form, Part},
	},
};

use crate::{consts::USER_AGENT, ext::ResultExt, racky_info, servers::Server};

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

	pub fn binary<K, V>(mut self, key: K, value: V) -> Self
	where
		K: Into<Cow<'static, str>>,
		V: AsRef<[u8]>,
	{
		self.fields.insert(key.into(), Field::Binary(value.as_ref().to_vec()));
		self
	}

	pub fn get(&self, path: &str) -> Result<Response> {
		let mut request = ReqwestClient::builder()
			.build()
			.desc("Failed to create HTTP GET client")?
			.get(self.format_url(path));

		for (key, value) in &self.fields {
			match value {
				Field::Text(value) => request = request.query(&[(key, value)]),
				Field::Binary(_) => bail!("Binary fields are not supported for GET requests"),
			}
		}

		self.send(request)
	}

	pub fn post(&self, path: &str) -> Result<Response> {
		let mut request = ReqwestClient::builder()
			.build()
			.desc("Failed to create HTTP POST client")?
			.post(self.format_url(path));

		if self.fields.values().any(|field| matches!(field, Field::Binary(_))) {
			let mut form = Form::new();

			for (key, value) in &self.fields {
				match value {
					Field::Text(value) => form = form.text(key.clone(), value.clone()),
					Field::Binary(value) => form = form.part(key.clone(), Part::bytes(value.clone())),
				}
			}

			request = request.multipart(form);
		} else if !self.fields.is_empty() {
			let fields = self
				.fields
				.iter()
				.map(|(key, value)| {
					(
						key.clone(),
						match value {
							Field::Text(value) => value.to_owned(),
							Field::Binary(_) => unreachable!(),
						},
					)
				})
				.collect::<HashMap<_, _>>();

			request = request.form(&fields);
		}

		self.send(request)
	}

	fn format_url(&self, path: &str) -> String {
		format!("https://{}:{}/{path}", self.address, self.port)
	}

	fn send(&self, mut request: RequestBuilder) -> Result<Response> {
		if let Some(password) = &self.password {
			request = request.bearer_auth(password);
		}

		let response = request
			.header("User-Agent", USER_AGENT)
			.send()
			.desc("Failed to connect to the server")?;

		Ok(Response(response.status(), response.text().unwrap_or_default()))
	}
}

#[derive(Debug)]
pub struct Response(pub StatusCode, pub String);

impl Response {
	pub fn with_prefix(self, prefix: &str) -> Self {
		if !self.0.is_success() {
			return self;
		}

		Self(self.0, format!("{prefix}{}", self.1))
	}

	pub fn handle(self) -> Result<()> {
		if self.0.is_success() {
			racky_info!("{}", self.1);
		} else if !self.1.is_empty() {
			bail!("{} ({})", self.1, self.0);
		} else {
			bail!("{}", self.0);
		}

		Ok(())
	}
}

#[derive(Debug)]
enum Field {
	Text(String),
	Binary(Vec<u8>),
}
