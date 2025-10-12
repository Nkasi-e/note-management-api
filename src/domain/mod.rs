// Domain module - contains business models and error types
pub mod user;
pub mod task;
pub mod error;
pub mod pagination;
pub mod file;

pub use user::{User, CreateUserRequest};
pub use task::{Task, CreateTaskRequest, TaskWithAttachments};
pub use error::{ApiError, Result};
pub use pagination::{
    PaginationParams, TaskFilters, TaskQueryParams, 
    PaginationMeta, PaginatedResponse
};
pub use file::{FileMetadata, UploadFileRequest, UploadFileResponse, FileStats};
