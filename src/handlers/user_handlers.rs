use axum::{
    extract::{Path, State, Extension},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::{CreateUserRequest, Result, ApiError};
use crate::domain::user::UserRole;
use crate::services::UserService;
use crate::middleware::CurrentUser;
use super::{respond_created, respond_ok};

#[derive(Debug, Deserialize)]
pub struct UserIdPath {
    pub id: String,
}

pub async fn create_user(
    State(user_service): State<UserService>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse> {
    // Only admins can create users via this endpoint
    if current_user.role != UserRole::Admin {
        return Err(ApiError::forbidden("Only administrators can create user accounts"));
    }

    let user = user_service.create_user(request).await?;
    Ok(respond_created(user))
}

pub async fn get_user(
    State(user_service): State<UserService>,
    Extension(current_user): Extension<CurrentUser>,
    Path(params): Path<UserIdPath>,
) -> Result<impl IntoResponse> {
    let user_id = params
        .id
        .parse::<Uuid>()
        .map_err(|_| ApiError::bad_request(format!("Invalid user ID format: {}", params.id)))?;

    // Users can only view their own profile, admins can view any profile
    if current_user.role != UserRole::Admin && current_user.id != user_id {
        return Err(ApiError::forbidden("You can only view your own profile"));
    }

    let user = user_service.get_user(user_id).await?;
    Ok(respond_ok(user))
}
