use std::{
	env,
	fmt::Display,
	path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use path_clean::PathClean;

/// Collection of extension methods for `Path`
pub trait PathExt {
	fn resolve(&self) -> Result<PathBuf>;
	fn to_string(&self) -> String;
	fn get_name(&self) -> &str;
	fn get_stem(&self) -> &str;
	fn get_ext(&self) -> &str;
	fn get_parent(&self) -> &Path;
	fn len(&self) -> usize;
	fn is_empty(&self) -> bool;
}

impl PathExt for Path {
	fn resolve(&self) -> Result<PathBuf> {
		if self.is_absolute() {
			return Ok(self.to_owned());
		}

		let current_dir = env::current_dir()?;
		let absolute = current_dir.join(self);

		Ok(absolute.clean())
	}

	fn to_string(&self) -> String {
		self.to_str().unwrap_or_default().to_owned()
	}

	fn get_name(&self) -> &str {
		self.file_name().unwrap_or_default().to_str().unwrap_or_default()
	}

	fn get_stem(&self) -> &str {
		if !self.is_dir() {
			self.file_stem().unwrap_or_default().to_str().unwrap_or_default()
		} else {
			self.get_name()
		}
	}

	fn get_ext(&self) -> &str {
		if !self.is_dir() {
			self.extension().unwrap_or_default().to_str().unwrap_or_default()
		} else {
			""
		}
	}

	fn get_parent(&self) -> &Path {
		self.parent().unwrap_or(self)
	}

	fn len(&self) -> usize {
		self.components().count()
	}

	fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

/// Additional methods for `anyhow::Error`, similar to `context` and `with_context`
pub trait ResultExt<T, E> {
	fn desc<D>(self, desc: D) -> Result<T, anyhow::Error>
	where
		D: Display + Send + Sync + 'static;

	fn with_desc<C, F>(self, f: F) -> Result<T, anyhow::Error>
	where
		C: Display + Send + Sync + 'static,
		F: FnOnce() -> C;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
	E: Display + Send + Sync + 'static,
{
	fn desc<D>(self, desc: D) -> Result<T, anyhow::Error>
	where
		D: Display + Send + Sync + 'static,
	{
		match self {
			Ok(ok) => Ok(ok),
			Err(err) => {
				bail!("{desc}: {err}");
			}
		}
	}

	fn with_desc<C, F>(self, desc: F) -> Result<T, anyhow::Error>
	where
		C: Display + Send + Sync + 'static,
		F: FnOnce() -> C,
	{
		match self {
			Ok(ok) => Ok(ok),
			Err(err) => {
				bail!("{}: {err}", desc());
			}
		}
	}
}
