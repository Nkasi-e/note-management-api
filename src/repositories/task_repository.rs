use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{Task, CreateTaskRequest, Result, ApiError};

#[derive(Debug, Clone)]
pub struct TaskRepository {
    tasks: Arc<RwLock<HashMap<Uuid, Task>>>,
}

impl TaskRepository {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create(&self, request: CreateTaskRequest) -> Result<Task> {
        let task = Task::new(request.title, request.description, request.user_id);
        let task_id = task.id;

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id, task.clone());
        }

        Ok(task)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Task> {
        let tasks = self.tasks.read().await;
        tasks
            .get(&id)
            .cloned()
            .ok_or(ApiError::TaskNotFound { id })
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().await;
        let user_tasks: Vec<Task> = tasks
            .values()
            .filter(|task| task.user_id == user_id)
            .cloned()
            .collect();

        Ok(user_tasks)
    }

    pub async fn find_all(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().await;
        let all_tasks: Vec<Task> = tasks.values().cloned().collect();
        Ok(all_tasks)
    }

    pub async fn count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }
}
