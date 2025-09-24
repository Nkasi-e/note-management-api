use uuid::Uuid;

use crate::domain::{Task, CreateTaskRequest, Result, ApiError};
use crate::repositories::{TaskRepository, UserRepository};

#[derive(Debug, Clone)]
pub struct TaskService {
    task_repository: TaskRepository,
    user_repository: UserRepository,
}

impl TaskService {
    pub fn new(task_repository: TaskRepository, user_repository: UserRepository) -> Self {
        Self { 
            task_repository,
            user_repository,
        }
    }

    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<Task> {
        // Business logic validation
        self.validate_task_request(&request)?;
        
        // Verify user exists
        if !self.user_repository.exists(request.user_id).await {
            return Err(ApiError::UserNotFound {
                id: request.user_id,
            });
        }

        // Delegate to repository
        self.task_repository.create(request).await
    }

    pub async fn get_task(&self, id: Uuid) -> Result<Task> {
        self.task_repository.find_by_id(id).await
    }

    pub async fn get_tasks_by_user(&self, user_id: Uuid) -> Result<Vec<Task>> {
        // Verify user exists
        if !self.user_repository.exists(user_id).await {
            return Err(ApiError::UserNotFound { id: user_id });
        }

        self.task_repository.find_by_user_id(user_id).await
    }

    pub async fn get_all_tasks(&self) -> Result<Vec<Task>> {
        self.task_repository.find_all().await
    }

    pub async fn get_task_count(&self) -> usize {
        self.task_repository.count().await
    }

    fn validate_task_request(&self, request: &CreateTaskRequest) -> Result<()> {
        if request.title.trim().is_empty() {
            return Err(ApiError::ValidationError("Title cannot be empty".to_string()));
        }
        
        if request.title.len() > 200 {
            return Err(ApiError::ValidationError("Title cannot exceed 200 characters".to_string()));
        }
        
        if let Some(desc) = &request.description {
            if desc.len() > 1000 {
                return Err(ApiError::ValidationError("Description cannot exceed 1000 characters".to_string()));
            }
        }

        Ok(())
    }
}
