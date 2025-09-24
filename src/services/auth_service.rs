use crate::config::settings::AuthConfig;
use crate::domain::{ApiError, Result, User};
use crate::repositories::UserRepository;
use crate::validation::Validator;
use argon2::{password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user id
    pub email: String,
    pub role: String,       // user role
    pub iss: String,
    pub aud: String,
    pub exp: usize,
}

#[derive(Clone)]
pub struct AuthService {
    user_repository: UserRepository,
    jwt_key: EncodingKey,
    cfg: AuthConfig,
}

impl AuthService {
    pub fn new(user_repository: UserRepository, cfg: AuthConfig) -> Self {
        let jwt_key = EncodingKey::from_secret(cfg.jwt_secret.as_bytes());
        Self { user_repository, jwt_key, cfg }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<User> {
        // Validate all input fields
        Validator::validate_register_request(&req)?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(req.password.as_bytes(), &salt)
            .map_err(|e| ApiError::internal_error(format!("Password hashing failed: {}", e)))?
            .to_string();

        let user = self.user_repository
            .create_with_password_hash(req.name.trim().to_string(), req.email.trim().to_lowercase(), password_hash)
            .await?;
        Ok(user)
    }

    pub async fn login(&self, req: LoginRequest) -> Result<TokenResponse> {
        // Validate input fields
        Validator::validate_login_request(&req)?;

        let auth = self.user_repository
            .find_auth_by_email(&req.email.trim().to_lowercase())
            .await
            .map_err(|e| ApiError::internal_error(format!("Login lookup failed: {}", e)))?;

        let (user, stored_hash) = auth.ok_or_else(|| ApiError::validation_error("Invalid email or password"))?;

        let parsed = PasswordHash::new(&stored_hash)
            .map_err(|_| ApiError::internal_error("Corrupt password hash"))?;
        let ok = Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed)
            .is_ok();
        if !ok {
            return Err(ApiError::validation_error("Invalid email or password"));
        }

        let exp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::minutes(self.cfg.expiry_minutes as i64))
            .ok_or_else(|| ApiError::internal_error("Failed to compute token expiry"))?
            .timestamp() as usize;

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.to_string(),
            iss: self.cfg.issuer.clone(),
            aud: self.cfg.audience.clone(),
            exp,
        };

        let token = encode(&Header::default(), &claims, &self.jwt_key)
            .map_err(|e| ApiError::internal_error(format!("JWT encoding failed: {}", e)))?;

        Ok(TokenResponse { token })
    }
}


