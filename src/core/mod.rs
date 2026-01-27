use std::{collections::HashMap, fs};

use anyhow::Result;
use log::error;

use crate::{core::program::Program, dirs, ext::PathExt};

mod program;

pub struct Core {
	programs: HashMap<String, Program>,
}

impl Core {
	pub fn new() -> Self {
		Self {
			programs: HashMap::new(),
		}
	}

	pub fn start(&mut self) -> Result<(usize, usize)> {
		let programs = self.get_autostart_programs()?;

		let total = programs.len();
		let mut successful = 0;

		for program in programs {
			if program.start() {
				successful += 1;
			}

			self.programs.insert(program.name.clone(), program);
		}

		Ok((successful, total))
	}

	fn get_autostart_programs(&self) -> Result<Vec<Program>> {
		let mut programs = Vec::new();

		for entry in fs::read_dir(dirs::config())? {
			if let Err(err) = entry {
				error!("Failed to read entry: {err}");
				continue;
			}

			let path = entry.unwrap().path();
			let stem = path.get_stem();

			if stem == "racky" {
				continue;
			}

			if let Some(program) = Program::new(stem)
				&& program.config.auto_start
			{
				programs.push(program);
			}
		}

		Ok(programs)
	}
}
