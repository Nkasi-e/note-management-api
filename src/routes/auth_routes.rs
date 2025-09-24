use axum::{
    routing::post,
    Router,
};

use crate::handlers::{register, login};
use crate::services::auth_service::AuthService;

pub fn auth_routes() -> Router<AuthService> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}


