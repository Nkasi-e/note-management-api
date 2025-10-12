use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use bytes::Bytes;
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

    pub async fn get_connection(&self) -> redis::RedisResult<ConnectionManager> {
        Ok(self.manager.clone())
    }

    // ==================== Zero-Copy Binary Methods ====================
    
    /// Get raw bytes from cache (zero-copy)
    /// Useful for caching binary data like serialized messages, files, etc.
    pub async fn get_bytes(&self, key: &str) -> redis::RedisResult<Option<Bytes>> {
        let mut con = self.manager.clone();
        let value: Option<Vec<u8>> = con.get(key).await?;
        Ok(value.map(Bytes::from))
    }

    /// Set raw bytes in cache (efficient for binary data)
    pub async fn set_bytes(&self, key: &str, value: Bytes) -> redis::RedisResult<()> {
        self.set_bytes_with_ttl(key, value, self.default_ttl_secs).await
    }

    /// Set raw bytes with TTL
    pub async fn set_bytes_with_ttl(&self, key: &str, value: Bytes, ttl_secs: u64) -> redis::RedisResult<()> {
        let mut con = self.manager.clone();
        // Bytes can be used directly without conversion
        let _: () = con.set_ex(key, value.as_ref(), ttl_secs).await?;
        Ok(())
    }

    /// Cache JSON as bytes for zero-copy retrieval
    /// Useful when the same cached data is sent to multiple clients
    pub async fn set_json_bytes<T: Serialize>(&self, key: &str, value: &T) -> redis::RedisResult<Bytes> {
        self.set_json_bytes_with_ttl(key, value, self.default_ttl_secs).await
    }

    /// Cache JSON as bytes with TTL, returns the serialized Bytes
    pub async fn set_json_bytes_with_ttl<T: Serialize>(&self, key: &str, value: &T, ttl_secs: u64) -> redis::RedisResult<Bytes> {
        let mut con = self.manager.clone();
        let payload = serde_json::to_vec(value)
            .map_err(|_| redis::RedisError::from((redis::ErrorKind::TypeError, "serde encode error")))?;
        
        let bytes = Bytes::from(payload);
        let _: () = con.set_ex(key, bytes.as_ref(), ttl_secs).await?;
        Ok(bytes)  // Return Bytes for potential reuse (zero-copy)
    }
}


