use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::{User, CreateUserRequest, Result, ApiError};
use crate::services::UserService;

#[derive(Debug, Deserialize)]
pub struct UserIdPath {
    pub id: String,
}

pub async fn create_user(
    State(user_service): State<UserService>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<User>> {
    let user = user_service.create_user(request).await?;
    Ok(Json(user))
}

pub async fn get_user(
    State(user_service): State<UserService>,
    Path(params): Path<UserIdPath>,
) -> Result<Json<User>> {
    let user_id = params
        .id
        .parse::<Uuid>()
        .map_err(|_| ApiError::InvalidUuid(params.id))?;

    let user = user_service.get_user(user_id).await?;
    Ok(Json(user))
}
