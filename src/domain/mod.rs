// Domain module - contains business models and error types
pub mod user;
pub mod task;
pub mod error;

pub use user::{User, CreateUserRequest};
pub use task::{Task, CreateTaskRequest};
pub use error::{ApiError, Result};
