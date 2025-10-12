use axum::{extract::State, response::IntoResponse, Extension};
use std::sync::Arc;
use crate::domain::Result;
use crate::services::auth_service::{AuthService, RegisterRequest, LoginRequest};
use crate::extractors::ValidatedJson;
use super::{respond_created, respond_ok};
use tracing::{info, debug};
use crate::workers::{WorkerService, JobType, JobPriority, EmailJobPayload};
use crate::services::EmailService;

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