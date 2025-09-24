use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{User, CreateUserRequest, Result, ApiError};

#[derive(Debug, Clone)]
pub struct UserRepository {
    users: Arc<RwLock<HashMap<Uuid, User>>>,
    user_emails: Arc<RwLock<HashMap<String, Uuid>>>, // email -> user_id mapping
}

impl UserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            user_emails: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create(&self, request: CreateUserRequest) -> Result<User> {
        // Validate data length constraints
        self.validate_user_data(&request.name, &request.email)?;
        
        // Check if email already exists
        {
            let emails = self.user_emails.read().await;
            if emails.contains_key(&request.email) {
                return Err(ApiError::EmailAlreadyExists {
                    email: request.email,
                });
            }
        }

        let user = User::new(request.name, request.email.clone());
        let user_id = user.id;

        // Store user and email mapping
        {
            let mut users = self.users.write().await;
            let mut emails = self.user_emails.write().await;
            
            users.insert(user_id, user.clone());
            emails.insert(request.email, user_id);
        }

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<User> {
        let users = self.users.read().await;
        users
            .get(&id)
            .cloned()
            .ok_or(ApiError::UserNotFound { id })
    }

    pub async fn exists(&self, id: Uuid) -> bool {
        let users = self.users.read().await;
        users.contains_key(&id)
    }

    pub async fn count(&self) -> usize {
        let users = self.users.read().await;
        users.len()
    }

    // Helper function that could potentially fail
    fn validate_user_data(&self, name: &str, email: &str) -> Result<()> {
        if name.len() > 100 {
            return Err(ApiError::InternalError("User name too long for storage".to_string()));
        }
        if email.len() > 255 {
            return Err(ApiError::InternalError("Email too long for storage".to_string()));
        }
        Ok(())
    }
}
