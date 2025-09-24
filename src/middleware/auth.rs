use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::config::settings::AuthConfig;
use crate::domain::user::UserRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user id
    pub email: String,
    pub role: String,       // user role
    pub iss: String,
    pub aud: String,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
}

fn create_error_response(status: StatusCode, message: &str) -> Response {
    let body = Json(json!({
        "success": false,
        "error": message,
        "status": status.as_u16()
    }));
    (status, body).into_response()
}

pub async fn auth_middleware(
    State(auth_config): State<AuthConfig>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| create_error_response(StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header"))?;

    // Check if it's a Bearer token
    if !auth_header.starts_with("Bearer ") {
        return Err(create_error_response(StatusCode::UNAUTHORIZED, "Invalid token format. Expected 'Bearer <token>'"));
    }

    // Extract the token
    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Decode and validate the JWT
    let decoding_key = DecodingKey::from_secret(auth_config.jwt_secret.as_bytes());
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_issuer(&[&auth_config.issuer]);
    validation.set_audience(&[&auth_config.audience]);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|_| create_error_response(StatusCode::UNAUTHORIZED, "Invalid or expired token"))?;

    // Parse user ID
    let user_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| create_error_response(StatusCode::UNAUTHORIZED, "Invalid user ID in token"))?;

    // Parse user role
    let user_role = token_data.claims.role.parse::<UserRole>()
        .map_err(|_| create_error_response(StatusCode::UNAUTHORIZED, "Invalid user role in token"))?;

    // Create CurrentUser and attach to request
    let current_user = CurrentUser {
        id: user_id,
        email: token_data.claims.email,
        role: user_role,
    };

    // Insert CurrentUser into request extensions
    request.extensions_mut().insert(current_user);

    Ok(next.run(request).await)
}

pub async fn admin_only_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Get CurrentUser from request extensions (set by auth_middleware)
    let current_user = request
        .extensions()
        .get::<CurrentUser>()
        .ok_or_else(|| create_error_response(StatusCode::UNAUTHORIZED, "Authentication required"))?;

    // Check if user has admin role
    if current_user.role != UserRole::Admin {
        return Err(create_error_response(StatusCode::FORBIDDEN, "Admin access required"));
    }

    Ok(next.run(request).await)
}
