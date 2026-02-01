#![allow(clippy::new_without_default)]

pub mod cli;
pub mod client;
pub mod command;
pub mod config;
pub mod constants;
pub mod core;
pub mod dirs;
pub mod ext;
pub mod logger;
pub mod servers;
pub mod util;
pub mod web;
pub mod zip;

/// `RwLock::read` shortcut
#[macro_export]
macro_rules! rlock {
	($lock:expr) => {
		$lock.read().expect("Tried to read RwLock that panicked!")
	};
}

/// `RwLock::write` shortcut
#[macro_export]
macro_rules! wlock {
	($lock:expr) => {
		$lock.write().expect("Tried to write RwLock that panicked!")
	};
}
