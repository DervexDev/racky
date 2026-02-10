use serde::Deserialize;

pub mod add;
pub mod config;
pub mod logs;
pub mod remove;
pub mod restart;
pub mod start;
pub mod status;
pub mod stop;
pub mod update;

#[derive(Debug, Deserialize)]
pub struct ProgramRequest {
	program: String,
}
