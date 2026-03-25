use axum::{extract::State, response::IntoResponse, Extension};
use std::sync::Arc;
use crate::domain::Result;
use crate::services::auth_service::{AuthService, RegisterRequest, LoginRequest};
use crate::extractors::ValidatedJson;
use super::{respond_created, respond_ok};
use tracing::{info, debug};
use crate::workers::{WorkerService, JobType, JobPriority, EmailJobPayload};
use crate::services::EmailService;

/// Register a new user
///
/// Creates a new user account with the provided credentials. The password is
/// securely hashed using Argon2 before storage. Upon successful registration,
/// a welcome email is sent asynchronously via a background job.
///
/// Returns the created user object (without password) and automatically logs
/// them in by generating a JWT token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = User),
        (status = 400, description = "Invalid input - validation failed", body = ApiErrorResponse),
        (status = 409, description = "Conflict - email already exists", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn register(
    State(auth): State<AuthService>,
    Extension(worker_service): Extension<Arc<WorkerService>>,
    Extension(email_service): Extension<Arc<EmailService>>,
    ValidatedJson(req): ValidatedJson<RegisterRequest>,
) -> Result<impl IntoResponse> {
    info!("User registration attempt for email: {}", req.email);
    debug!("Registration request payload: {:?}", req);
    
    let user = auth.register(req).await?;
    
    // Generate welcome email content using EmailService
    let (subject, body) = email_service.generate_welcome_email_content(&user.name);

    let payload = EmailJobPayload {
        to: user.email.clone(),
        subject,
        body,
        user_id: user.id,
    };

    // Fire and forget: enqueue background job
    if let Err(e) = worker_service.enqueue_job(JobType::SendEmail(payload), JobPriority::Normal).await {
        tracing::error!("Failed to enqueue welcome email job for user {}: {}", user.id, e);
    }

    info!("User registered successfully: {} ({})", user.id, user.email);
    Ok(respond_created(user))
}

/// Login user
///
/// Authenticates a user with their email and password. If credentials are valid,
/// returns a JWT token that can be used for subsequent authenticated requests.
///
/// The token should be included in the Authorization header as: `Bearer <token>`
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = crate::services::auth_service::TokenResponse),
        (status = 400, description = "Invalid input", body = ApiErrorResponse),
        (status = 401, description = "Invalid credentials", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
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