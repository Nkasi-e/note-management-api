use axum::{
    routing::get,
    Router,
};

use crate::websocket::ws_handler::{ws_handler, ws_health};
use crate::websocket::WebSocketManager;
use crate::config::settings::AuthConfig;

/// WebSocket routes
pub fn ws_routes(ws_manager: WebSocketManager, auth_config: AuthConfig) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .route("/ws/health", get(ws_health))
        .with_state(auth_config)
        .layer(axum::Extension(ws_manager))
}

