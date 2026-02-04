use serde::Deserialize;

pub mod add;
pub mod remove;
pub mod restart;
pub mod start;
pub mod stop;

#[derive(Debug, Deserialize)]
pub struct ProgramRequest {
	program: String,
}
