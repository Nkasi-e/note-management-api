use uuid::Uuid;

use crate::domain::{User, CreateUserRequest, Result, ApiError};
use crate::repositories::UserRepository;

#[derive(Debug, Clone)]
pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        // Business logic validation
        self.validate_user_request(&request)?;
        
        // Delegate to repository
        self.user_repository.create(request).await
    }

    pub async fn get_user(&self, id: Uuid) -> Result<User> {
        self.user_repository.find_by_id(id).await
    }

    pub async fn user_exists(&self, id: Uuid) -> bool {
        self.user_repository.exists(id).await
    }

    pub async fn get_user_count(&self) -> usize {
        self.user_repository.count().await
    }

    fn validate_user_request(&self, request: &CreateUserRequest) -> Result<()> {
        if request.name.trim().is_empty() {
            return Err(ApiError::ValidationError("Name cannot be empty".to_string()));
        }
        
        if request.email.trim().is_empty() {
            return Err(ApiError::ValidationError("Email cannot be empty".to_string()));
        }
        
        // Basic email validation
        if !request.email.contains('@') {
            return Err(ApiError::ValidationError("Invalid email format".to_string()));
        }

        Ok(())
    }
}
