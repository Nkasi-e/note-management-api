use note_task_api::{
    config::AppConfig,
    repositories::{UserRepository, TaskRepository},
    services::{UserService, TaskService, AuthService},
    routes::{api_v1_routes, health_routes},
    middleware::{logging_middleware, json_404_middleware},
    init_pg_pool,
};

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load environment variables from .env if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "note_task_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration from environment (with defaults)
    let config = AppConfig::from_env();
    
    // Initialize Postgres pool
    let pool = init_pg_pool(&config).await;
    
    // Initialize repositories (Postgres-backed)
    let user_repository = UserRepository::new(pool.clone());
    let task_repository = TaskRepository::new(pool.clone());
    
    // Initialize services
    let user_service = UserService::new(user_repository.clone());
    let task_service = TaskService::new(task_repository, user_repository.clone());
    let auth_service = AuthService::new(user_repository, config.auth.clone());

    // Build our application with modular routes
    let app = Router::new()
        .merge(health_routes())
        .merge(api_v1_routes(user_service, task_service, auth_service, config.auth.clone()))
        // Add middleware
        .layer(logging_middleware())
        .layer(axum::middleware::from_fn(json_404_middleware))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    // Run the server using config-resolved host/port
    let addr = SocketAddr::from((config.server.host.parse::<std::net::IpAddr>().unwrap(), config.server.port));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
