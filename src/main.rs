use note_task_api::{
    config::AppConfig,
    repositories::{UserRepository, TaskRepository},
    services::{UserService, TaskService},
    routes::{user_routes, task_routes, health_routes},
    middleware::logging_middleware,
};

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "note_task_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::default();
    
    // Initialize repositories
    let user_repository = UserRepository::new();
    let task_repository = TaskRepository::new();
    
    // Initialize services
    let user_service = UserService::new(user_repository.clone());
    let task_service = TaskService::new(task_repository, user_repository);

    // Build our application with modular routes
    let app = Router::new()
        .merge(health_routes())
        .merge(user_routes().with_state(user_service))
        .merge(task_routes().with_state(task_service))
        // Add middleware
        .layer(logging_middleware())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    // Run the server
    let addr = SocketAddr::from((config.server.host.parse::<std::net::IpAddr>().unwrap(), config.server.port));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
