use std::process::Command as StdCommand;

use anyhow::{Result, bail};

#[derive(Debug)]
pub struct Command {
	program: String,
	args: Vec<String>,
}

impl Command {
	pub fn new<S>(program: S) -> Self
	where
		S: Into<String>,
	{
		Self {
			program: program.into(),
			args: Vec::new(),
		}
	}

	pub fn arg<S>(&mut self, arg: S) -> &mut Self
	where
		S: Into<String>,
	{
		self.args.push(arg.into());
		self
	}

	pub fn args<I, S>(&mut self, args: I) -> &mut Self
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		self.args.extend(args.into_iter().map(|arg| arg.into()));
		self
	}

	pub fn run(&self) -> Result<String> {
		match StdCommand::new(&self.program).args(&self.args).output() {
			Ok(output) => match output.status.success() {
				true => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
				false => bail!("{}", String::from_utf8_lossy(&output.stderr)),
			},
			Err(err) => Err(err.into()),
		}
	}
}
