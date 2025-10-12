use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub auth: AuthConfig,
    pub redis: RedisConfig,
    pub email: EmailConfig,
    pub storage: StorageConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub issuer: String,
    pub audience: String,
    pub expiry_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub ttl_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub upload_dir: String,
    pub max_file_size: usize,        // in bytes
    pub allowed_extensions: Vec<String>,
    pub serve_files: bool,
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
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let issuer = std::env::var("JWT_ISSUER").unwrap_or_else(|_| "note-task-api".to_string());
        let audience = std::env::var("JWT_AUDIENCE").unwrap_or_else(|_| "note-clients".to_string());
        let expiry_minutes: u64 = std::env::var("JWT_EXP_MINUTES").ok().and_then(|v| v.parse().ok()).unwrap_or(60);

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
            auth: AuthConfig { jwt_secret, issuer, audience, expiry_minutes },
            redis: RedisConfig {
                url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
                ttl_secs: std::env::var("REDIS_TTL_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(300),
            },
            email: EmailConfig {
                smtp_host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
                smtp_port: std::env::var("SMTP_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(587),
                username: std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set (your Gmail address)"),
                password: std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set (Gmail App Password)"),
                from_email: std::env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@yourdomain.com".to_string()),
            },
            storage: StorageConfig {
                upload_dir: std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()),
                max_file_size: std::env::var("MAX_FILE_SIZE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10 * 1024 * 1024), // 10MB default
                allowed_extensions: std::env::var("ALLOWED_EXTENSIONS")
                    .unwrap_or_else(|_| "jpg,jpeg,png,gif,pdf,doc,docx,txt,zip".to_string())
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .collect(),
                serve_files: std::env::var("SERVE_FILES")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
            },
        }
    }
}
