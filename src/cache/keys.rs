pub fn task_key(id: &uuid::Uuid) -> String { format!("task:{}", id) }
pub fn user_tasks_key(user_id: &uuid::Uuid) -> String { format!("user_tasks:{}", user_id) }
pub fn all_tasks_key() -> String { "tasks:all".to_string() }


