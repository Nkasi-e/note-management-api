use std::sync::Arc;
use note_task_api::{
    config::AppConfig,
    repositories::{UserRepository, TaskRepository, FileRepository},
    services::{UserService, TaskService, AuthService, EmailService, FileService},
    routes::{api_v1_routes, health_routes, ws_routes},
    middleware::{logging_middleware, request_logging_middleware, json_404_middleware},
    workers::{WorkerService, JobProcessor, JobQueueConfig},
    websocket::WebSocketManager,
    init_pg_pool,
};

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use redis::Client as RedisClient;
use redis::aio::ConnectionManager as RedisConnectionManager;
use note_task_api::cache::RedisCache;

#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "note_task_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // App configuration
    let config = AppConfig::from_env();
    
    // Postgres pool
    let pool = init_pg_pool(&config).await;
    
    // Repositories
    let user_repository = UserRepository::new(pool.clone());
    let task_repository = TaskRepository::new(pool.clone());
    let file_repository = FileRepository::new(pool.clone());
    
    // Redis cache
    let redis_client = RedisClient::open(config.redis.url.clone()).expect("Invalid REDIS_URL");
    let redis_manager = RedisConnectionManager::new(redis_client).await.expect("Failed to connect to Redis");
    let cache = RedisCache::new(redis_manager, config.redis.ttl_secs);

    // Services
    let user_service = UserService::new(user_repository.clone());
    let task_service = TaskService::new(task_repository.clone(), user_repository.clone(), file_repository.clone(), Some(cache.clone()));
    let auth_service = AuthService::new(user_repository.clone(), config.auth.clone());
    let email_service = EmailService::new(config.email.clone());
    let file_service = FileService::new(file_repository, config.storage.clone())
        .await
        .expect("Failed to initialize file service");
    
    // Initialize file storage directory (for local storage)
    file_service.init_storage().await.expect("Failed to initialize file storage");

    // Background worker
    let job_processor = JobProcessor::new(
        std::sync::Arc::new(user_repository.clone()),
        std::sync::Arc::new(task_repository),
        std::sync::Arc::new(task_service.clone()),
        std::sync::Arc::new(email_service.clone()),
    );
    
    let worker_config = JobQueueConfig {
        max_retries: 3,
        retry_delay_secs: 60,
        job_timeout_secs: 300,
        worker_pool_size: 4,
        queue_name: "default".to_string(),
    };
    
    // WebSocket manager
    let ws_manager = WebSocketManager::new();
    
    let worker_service = WorkerService::new(cache.clone(), job_processor, worker_config)
        .with_websocket(ws_manager.clone());

    // Router
    let app = Router::new()
        .merge(health_routes())
        .merge(ws_routes(ws_manager.clone(), config.auth.clone()))
        .merge(api_v1_routes(user_service, task_service, auth_service, file_service, config.auth.clone(), Arc::new(worker_service.clone()), Arc::new(email_service)))
        // Middleware
        .layer(axum::middleware::from_fn(request_logging_middleware))
        .layer(logging_middleware())
        .layer(axum::middleware::from_fn(json_404_middleware))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    // Start worker
    let worker_service_clone = worker_service.clone();
    tokio::spawn(async move {
        if let Err(e) = worker_service_clone.start().await {
            tracing::error!("Failed to start worker service: {}", e);
        }
    });

    // Start HTTP server
    let addr = SocketAddr::from((config.server.host.parse::<std::net::IpAddr>().unwrap(), config.server.port));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
