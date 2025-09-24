use axum::Router;

use crate::services::{UserService, TaskService, AuthService};
use crate::middleware::auth_middleware;
use crate::config::settings::AuthConfig;

use super::{user_routes, task_routes, auth_routes};

pub fn api_v1_routes(
    user_service: UserService,
    task_service: TaskService,
    auth_service: AuthService,
    auth_config: AuthConfig,
) -> Router {
    Router::new()
        .nest("/api/v1", Router::new()
            .nest("/auth", auth_routes().with_state(auth_service))
            .nest("/users", 
                user_routes()
                    .with_state(user_service)
                    .layer(axum::middleware::from_fn_with_state(auth_config.clone(), auth_middleware))
            )
            .nest("/tasks", 
                task_routes()
                    .with_state(task_service)
                    .layer(axum::middleware::from_fn_with_state(auth_config, auth_middleware))
            )
        )
}
