use axum::{
    routing::get,
    Router,
};

use crate::handlers::{ping, health};

pub fn health_routes() -> Router {
    Router::new()
        .route("/ping", get(ping))
        .route("/health", get(health))
}
