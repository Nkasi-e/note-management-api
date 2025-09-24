use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("APP_HOST")
            .expect("APP_HOST must be set (e.g. 127.0.0.1)");
        let port: u16 = std::env::var("APP_PORT")
            .expect("APP_PORT must be set (e.g. 3001)")
            .parse()
            .expect("APP_PORT must be a valid u16");
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let max_connections: u32 = std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);
        let connection_timeout: u64 = std::env::var("DB_CONNECTION_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        let format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string());

        AppConfig {
            server: ServerConfig {
                host,
                port,
                cors_origins: vec!["*".to_string()],
            },
            database: DatabaseConfig {
                database_url,
                max_connections,
                connection_timeout,
            },
            logging: LoggingConfig {
                level,
                format
            },
        }
    }
}
