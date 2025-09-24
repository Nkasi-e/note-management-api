use axum::{extract::State, response::IntoResponse};
use crate::domain::Result;
use crate::services::auth_service::{AuthService, RegisterRequest, LoginRequest};
use crate::extractors::ValidatedJson;
use super::{respond_created, respond_ok};

pub async fn register(
    State(auth): State<AuthService>,
    ValidatedJson(req): ValidatedJson<RegisterRequest>,
) -> Result<impl IntoResponse> {
    let user = auth.register(req).await?;
    Ok(respond_created(user))
}

pub async fn login(
    State(auth): State<AuthService>,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> Result<impl IntoResponse> {
    let token = auth.login(req).await?;
    Ok(respond_ok(token))
}


