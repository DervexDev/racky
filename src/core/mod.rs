use std::{
	collections::HashMap,
	fs,
	sync::{Arc, RwLock, RwLockReadGuard},
	time::SystemTime,
};

use anyhow::{Result, bail};
use log::{error, trace, warn};

use crate::{
	core::program::{Program, ProgramPtr, Status},
	dirs,
	ext::{PathExt, ResultExt},
	rlock, wlock,
};

pub mod program;

pub type CorePtr = Arc<Core>;

#[derive(Debug)]
pub struct Core {
	programs: RwLock<HashMap<String, ProgramPtr>>,
	start_time: SystemTime,
}

impl Core {
	pub fn new() -> CorePtr {
		Arc::new(Self {
			programs: RwLock::new(HashMap::new()),
			start_time: SystemTime::now(),
		})
	}

	pub fn start(self: &CorePtr) -> Result<(usize, usize)> {
		let mut total = 0;
		let mut successful = 0;

		for entry in fs::read_dir(dirs::config()).desc("Failed to read config directory")? {
			if let Err(err) = entry {
				error!("Failed to check program config: {err}");
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

			if self.add_program(&program).and_then(|_| program.start()).is_ok() {
				successful += 1;
			}
		}

		Ok((successful, total))
	}

	pub fn start_program(self: &CorePtr, program: &ProgramPtr) -> Result<()> {
		let message = match program.status() {
			Status::Running(_) => Some("Program is already running"),
			Status::Restarting => Some("Program is now restarting"),
			_ => None,
		};

		if let Some(message) = message {
			warn!("Program {} could not be started: {message}", program.name());
			bail!("{message}");
		}

		program.state_mut().attempts.set_current(0);
		program.load_config();
		program.start()
	}

	pub fn stop_program(self: &CorePtr, program: &ProgramPtr) -> Result<()> {
		if !program.is_active() {
			warn!(
				"Program {} could not be stopped: Program is not running",
				program.name()
			);
			bail!("Program is not running");
		}

		program.stop()
	}

	pub fn programs<'a>(self: &'a CorePtr) -> RwLockReadGuard<'a, HashMap<String, ProgramPtr>> {
		rlock!(self.programs)
	}

	pub fn get_program(self: &CorePtr, name: &str) -> Option<ProgramPtr> {
		rlock!(self.programs).get(name).cloned()
	}

	pub fn add_program(self: &CorePtr, program: &ProgramPtr) -> Result<()> {
		let mut programs = wlock!(self.programs);
		let name = program.name();

		if programs.contains_key(name) {
			warn!("Program {name} already exists in core");
			bail!("Program already exists");
		}

		programs.insert(name.to_owned(), program.to_owned());
		trace!("Program {name} added");

		Ok(())
	}

	pub fn remove_program(self: &CorePtr, program: &ProgramPtr) -> Result<()> {
		let name = program.name();

		match wlock!(self.programs).remove(name) {
			Some(_) => {
				trace!("Program {name} removed");
				Ok(())
			}
			None => {
				warn!("Program {name} does not exist in core");
				bail!("Program does not exist");
			}
		}
	}

	#[allow(clippy::needless_lifetimes)]
	pub fn start_time<'a>(self: &'a CorePtr) -> &'a SystemTime {
		&self.start_time
	}
}
