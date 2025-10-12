// Routes module - route definitions
pub mod api;
pub mod user_routes;
pub mod task_routes;
pub mod health_routes;
pub mod auth_routes;
pub mod worker_routes;
pub mod ws_routes;

pub use api::api_v1_routes;
pub use user_routes::user_routes;
pub use task_routes::task_routes;
pub use health_routes::health_routes;
pub use auth_routes::auth_routes;
pub use worker_routes::worker_routes;
pub use ws_routes::ws_routes;
