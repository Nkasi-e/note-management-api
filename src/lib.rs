// Library root - exports all public modules
pub mod config;
pub mod domain;
pub mod services;
pub mod repositories;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod db;

// Re-export commonly used types for convenience
pub use domain::error::{ApiError, Result};
pub use domain::user::{User, CreateUserRequest};
pub use domain::task::{Task, CreateTaskRequest};
pub use config::settings::AppConfig;
pub use db::init_pg_pool;
