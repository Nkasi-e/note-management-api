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

/// Create a new user (Admin only)
///
/// Creates a new user account. This endpoint is restricted to administrators only.
/// Regular users should use the `/api/v1/auth/register` endpoint instead.
#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 400, description = "Invalid request body", body = ApiErrorResponse),
        (status = 403, description = "Forbidden - admin only", body = ApiErrorResponse),
        (status = 409, description = "User already exists", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Get user by ID
///
/// Retrieves a user's profile information. Users can only view their own profile,
/// while administrators can view any user's profile.
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID (UUID format)")
    ),
    responses(
        (status = 200, description = "User found successfully", body = User),
        (status = 400, description = "Invalid user ID format", body = ApiErrorResponse),
        (status = 404, description = "User not found", body = ApiErrorResponse),
        (status = 403, description = "Forbidden - not your profile", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
