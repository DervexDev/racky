use std::{
	collections::HashMap,
	fs,
	sync::{Arc, RwLock},
};

use anyhow::Result;
use log::error;

use crate::{
	core::program::{Program, ProgramPtr},
	dirs,
	ext::{PathExt, ResultExt},
	racky_error, racky_warn, wlock,
};

pub mod program;

pub type CorePtr = Arc<Core>;

#[derive(Debug, Default)]
pub struct Core {
	programs: RwLock<HashMap<String, ProgramPtr>>,
}

impl Core {
	pub fn new() -> CorePtr {
		Arc::new(Self::default())
	}

	pub fn start(self: &CorePtr) -> Result<(usize, usize)> {
		let mut total = 0;
		let mut successful = 0;

		let mut programs = wlock!(self.programs);

		for entry in fs::read_dir(dirs::config()).desc("Failed to read config directory")? {
			if let Err(err) = entry {
				error!("Failed to read entry: {err}");
				continue;
			}

			let path = entry.unwrap().path();
			let stem = path.get_stem();

			if stem == "racky" {
				continue;
			}

			let program = Program::new(stem);
			program.load_config();

			if !program.config().auto_start {
				continue;
			}

			total += 1;

			if !program.is_valid() {
				racky_warn!("Program {stem} has a config file but no executable");
				continue;
			}

			if program.start() {
				successful += 1;
			}

			programs.insert(stem.to_owned(), program);
		}

		Ok((successful, total))
	}

	pub fn start_program(self: &CorePtr, program: &ProgramPtr) -> bool {
		if !program.is_valid() {
			racky_error!("Program {} does not exist", program.name());
			return false;
		}

		let result = program.start();
		wlock!(self.programs).insert(program.name().to_owned(), program.to_owned());

		result
	}

	pub fn stop_program(self: &CorePtr, program: &ProgramPtr) -> bool {
		wlock!(self.programs).remove(program.name());
		program.stop()
	}
}
