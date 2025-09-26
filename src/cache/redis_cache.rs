use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;

#[derive(Clone)]
pub struct RedisCache {
    manager: ConnectionManager,
    default_ttl_secs: u64,
}

impl fmt::Debug for RedisCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RedisCache")
            .field("default_ttl_secs", &self.default_ttl_secs)
            .finish()
    }
}

impl RedisCache {
    pub fn new(manager: ConnectionManager, default_ttl_secs: u64) -> Self {
        Self { manager, default_ttl_secs }
    }

    pub fn with_ttl(&self, ttl_secs: u64) -> Self {
        Self { manager: self.manager.clone(), default_ttl_secs: ttl_secs }
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> redis::RedisResult<Option<T>> {
        let mut con = self.manager.clone();
        let value: Option<String> = con.get(key).await?;
        if let Some(s) = value {
            match serde_json::from_str::<T>(&s) {
                Ok(v) => Ok(Some(v)),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn set_json<T: Serialize>(&self, key: &str, value: &T) -> redis::RedisResult<()> {
        self.set_json_with_ttl(key, value, self.default_ttl_secs).await
    }

    pub async fn set_json_with_ttl<T: Serialize>(&self, key: &str, value: &T, ttl_secs: u64) -> redis::RedisResult<()> {
        let mut con = self.manager.clone();
        let payload = serde_json::to_string(value)
            .map_err(|_| redis::RedisError::from((redis::ErrorKind::TypeError, "serde encode error")))?;
        let _: () = con.set_ex(key, payload, ttl_secs).await?;
        Ok(())
    }

    pub async fn del(&self, key: &str) -> redis::RedisResult<()> {
        let mut con = self.manager.clone();
        let _: () = con.del(key).await?;
        Ok(())
    }
}


