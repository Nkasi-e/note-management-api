use axum::response::Json;

/// Ping endpoint
///
/// Simple ping endpoint to check if the server is responding.
/// This is a public endpoint that doesn't require authentication.
#[utoipa::path(
    get,
    path = "/ping",
    tag = "health",
    responses(
        (status = 200, description = "Server is running", body = inline(serde_json::Value))
    )
)]
pub async fn ping() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "message": "Server is running",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Health check endpoint
///
/// Returns the health status of the API including version information.
/// This is a public endpoint that doesn't require authentication.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = inline(serde_json::Value))
    )
)]
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "message": "All systems operational",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}
