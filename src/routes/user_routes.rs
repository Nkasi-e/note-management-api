use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{create_user, get_user};
use crate::services::UserService;

pub fn user_routes() -> Router<UserService> {
    Router::new()
        .route("/users/", post(create_user))
        .route("/users/:id", get(get_user))
}
