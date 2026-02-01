#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
	fs::{self, File},
	io::{self, Cursor},
	path::Path,
};

use anyhow::{Context, Result};
use walkdir::WalkDir;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::ext::{PathExt, ResultExt};

pub fn compress(target: &Path) -> Result<Vec<u8>> {
	let options = SimpleFileOptions::default().unix_permissions(0o755);
	let mut writer = ZipWriter::new(Cursor::new(Vec::new()));

	if target.is_dir() {
		let root = target.get_name();

		for entry in WalkDir::new(target) {
			let entry = entry.with_desc(|| format!("Error while traversing directory {target:?}"))?;
			let path = entry.path();

			let name = Path::new(root).join(path.strip_prefix(target)?);
			let name = name.to_str().with_context(|| format!("{path:?} is a non UTF-8 path"))?;

			if path.is_file() {
				writer.start_file(name, options)?;
				io::copy(&mut File::open(path)?, &mut writer)?;
			} else {
				writer.add_directory(name, options)?;
			}
		}
	} else {
		writer.start_file(target.get_name(), options)?;
		io::copy(&mut File::open(target)?, &mut writer)?;
	}

	Ok(writer.finish()?.into_inner())
}

pub fn decompress(data: &[u8], target: &Path) -> Result<()> {
	let mut archive = ZipArchive::new(Cursor::new(data)).desc("Unable to open archive")?;

	for i in 0..archive.len() {
		let mut file = archive
			.by_index(i)
			.with_desc(|| format!("Unable to open file {i} in archive"))?;
		let path = target.join(
			file.enclosed_name()
				.with_context(|| format!("Unable to extract file {i} because it has an invalid path"))?,
		);

		if file.is_dir() {
			fs::create_dir_all(&path).with_desc(|| format!("Unable to extract directory {i} to {path:?}"))?;
		} else {
			if let Some(parent) = path.parent()
				&& !parent.exists()
			{
				fs::create_dir_all(parent)
					.with_desc(|| format!("Unable to create parent directory {parent:?} of file {path:?}"))?;
			}

			File::create(&path)
				.and_then(|mut out| io::copy(&mut file, &mut out))
				.with_desc(|| format!("Unable to extract file {i} to {path:?}"))?;
		}

		#[cfg(unix)]
		if let Some(mode) = file.unix_mode() {
			fs::set_permissions(&path, fs::Permissions::from_mode(mode))
				.with_desc(|| format!("Unable to change permissions of file {i} ({path:?}) to {mode}"))?;
		}
	}

	Ok(())
}

pub fn get_root_name(data: &[u8]) -> Result<String> {
	ZipArchive::new(Cursor::new(data))
		.desc("Unable to open archive")?
		.by_index(0)
		.desc("Unable to open file root in archive")?
		.enclosed_name()
		.context("Unable to extract root file because it has an invalid path")
		.map(|p| p.get_stem().to_owned())
}
