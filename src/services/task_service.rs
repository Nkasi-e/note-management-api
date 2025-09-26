use uuid::Uuid;

use crate::domain::{Task, CreateTaskRequest, Result, ApiError};
use crate::repositories::{TaskRepository, UserRepository};
use crate::cache::{RedisCache, task_key, user_tasks_key, all_tasks_key};

#[derive(Debug, Clone)]
pub struct TaskService {
    task_repository: TaskRepository,
    user_repository: UserRepository,
    cache: Option<RedisCache>,
}

impl TaskService {
    pub fn new(task_repository: TaskRepository, user_repository: UserRepository, cache: Option<RedisCache>) -> Self {
        Self {
            task_repository,
            user_repository,
            cache,
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
        let task = self.task_repository.create(request).await?;

        // Invalidate caches related to tasks
        if let Some(cache) = &self.cache {
            let _ = cache.del(&all_tasks_key()).await;
            let _ = cache.del(&user_tasks_key(&task.user_id)).await;
            let _ = cache.set_json(&task_key(&task.id), &task).await;
        }

        Ok(task)
    }

    pub async fn get_task(&self, id: Uuid) -> Result<Task> {
        if let Some(cache) = &self.cache {
            if let Ok(Some(task)) = cache.get_json::<Task>(&task_key(&id)).await {
                return Ok(task);
            }
        }

        let task = self.task_repository.find_by_id(id).await?;
        if let Some(cache) = &self.cache {
            let _ = cache.set_json(&task_key(&id), &task).await;
        }
        Ok(task)
    }

    pub async fn get_tasks_by_user(&self, user_id: Uuid) -> Result<Vec<Task>> {
        // Verify user exists
        if !self.user_repository.exists(user_id).await {
            return Err(ApiError::UserNotFound { id: user_id });
        }

        if let Some(cache) = &self.cache {
            if let Ok(Some(tasks)) = cache.get_json::<Vec<Task>>(&user_tasks_key(&user_id)).await {
                return Ok(tasks);
            }
        }

        let tasks = self.task_repository.find_by_user_id(user_id).await?;
        if let Some(cache) = &self.cache {
            let _ = cache.set_json(&user_tasks_key(&user_id), &tasks).await;
        }
        Ok(tasks)
    }

    pub async fn get_all_tasks(&self) -> Result<Vec<Task>> {
        if let Some(cache) = &self.cache {
            if let Ok(Some(tasks)) = cache.get_json::<Vec<Task>>(&all_tasks_key()).await {
                return Ok(tasks);
            }
        }

        let tasks = self.task_repository.find_all().await?;
        if let Some(cache) = &self.cache {
            let _ = cache.set_json(&all_tasks_key(), &tasks).await;
        }
        Ok(tasks)
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
