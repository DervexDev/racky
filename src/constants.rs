use axum::extract::DefaultBodyLimit;

pub const BODY_SIZE_LIMIT: DefaultBodyLimit = DefaultBodyLimit::max(100 * 1024 * 1024);
