use axum::{extract::State, response::IntoResponse};
use crate::domain::Result;
use crate::services::auth_service::{AuthService, RegisterRequest, LoginRequest};
use crate::extractors::ValidatedJson;
use super::{respond_created, respond_ok};
use tracing::{info, debug};

pub async fn register(
    State(auth): State<AuthService>,
    ValidatedJson(req): ValidatedJson<RegisterRequest>,
) -> Result<impl IntoResponse> {
    info!("User registration attempt for email: {}", req.email);
    debug!("Registration request payload: {:?}", req);
    
    let user = auth.register(req).await?;
    
    info!("User registered successfully: {} ({})", user.id, user.email);
    Ok(respond_created(user))
}

pub async fn login(
    State(auth): State<AuthService>,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> Result<impl IntoResponse> {
    let email = req.email.clone();
    info!("Login attempt for email: {}", email);
    debug!("Login request payload: {:?}", req);
    
    let token = auth.login(req).await?;
    
    info!("User logged in successfully: {}", email);
    Ok(respond_ok(token))
}