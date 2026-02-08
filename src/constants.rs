use axum::extract::DefaultBodyLimit;

pub const USER_AGENT: &str = "Racky CLI";
pub const BODY_SIZE_LIMIT: DefaultBodyLimit = DefaultBodyLimit::max(100 * 1024 * 1024);

pub const GIGABYTE: f64 = 1024.0 * 1024.0 * 1024.0;
