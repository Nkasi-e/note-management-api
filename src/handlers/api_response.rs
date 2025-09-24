use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), message: None }
    }

    pub fn created(data: T) -> Self {
        Self { success: true, data: Some(data), message: None }
    }

    pub fn msg(message: impl Into<String>) -> Self {
        Self { success: true, data: None, message: Some(message.into()) }
    }
}

pub fn respond_ok<T: Serialize>(data: T) -> impl IntoResponse {
    Json(ApiResponse::ok(data))
}

pub fn respond_created<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::CREATED, Json(ApiResponse::created(data)))
}


