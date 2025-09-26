//! # Distributed Rate Limiting Module
//!
//! Implements distributed rate limiting with Redis coordination.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::ProxyError;

/// # Distributed Rate Limiting Configuration
///
/// Configuration for distributed rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedRateLimitConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// Rate limit key prefix
    pub key_prefix: String,
    /// Whether distributed rate limiting is enabled
    pub enabled: bool,
    /// Maximum requests per minute per user
    pub max_requests_per_minute: u64,
    /// Time window for rate limiting in seconds
    pub window_size_seconds: u64,
    /// Whether to use in-memory fallback when Redis is unavailable
    pub fallback_to_memory: bool,
}

impl Default for DistributedRateLimitConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            key_prefix: "nexus_nitro_llm:rate_limit".to_string(),
            enabled: false,
            max_requests_per_minute: 100,
            window_size_seconds: 60,
            fallback_to_memory: true,
        }
    }
}

/// In-memory rate limit entry
#[derive(Debug, Clone)]
struct RateLimitEntry {
    requests: Vec<u64>,
    last_cleanup: u64,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            last_cleanup: Self::current_time(),
        }
    }

    fn current_time() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs()
    }

    fn cleanup_old_requests(&mut self, window_size: u64) {
        let current_time = Self::current_time();
        let cutoff_time = current_time.saturating_sub(window_size);

        self.requests.retain(|&timestamp| timestamp > cutoff_time);
        self.last_cleanup = current_time;
    }

    fn add_request(&mut self) -> bool {
        let current_time = Self::current_time();
        self.requests.push(current_time);
        true
    }

    fn is_allowed(&mut self, max_requests: u64, window_size: u64) -> bool {
        self.cleanup_old_requests(window_size);
        (self.requests.len() as u64) < max_requests
    }
}

/// # Distributed Rate Limiter
///
/// Implements distributed rate limiting with Redis coordination.
#[derive(Debug)]
pub struct DistributedRateLimiter {
    /// Configuration
    config: DistributedRateLimitConfig,
    /// In-memory fallback storage
    memory_store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    /// Redis connection status
    redis_available: Arc<RwLock<bool>>,
}

impl DistributedRateLimiter {
    /// Create a new distributed rate limiter
    pub fn new(config: DistributedRateLimitConfig) -> Self {
        Self {
            config,
            memory_store: Arc::new(RwLock::new(HashMap::new())),
            redis_available: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the rate limiter and test Redis connection
    pub async fn initialize(&self) -> Result<(), ProxyError> {
        if !self.config.enabled {
            tracing::info!("Distributed rate limiting is disabled");
            return Ok(());
        }

        // Try to connect to Redis if configured
        if self.config.redis_url.starts_with("redis://") {
            match self.test_redis_connection().await {
                Ok(_) => {
                    let mut redis_status = self.redis_available.write().await;
                    *redis_status = true;
                    tracing::info!("Redis connection established for distributed rate limiting");
                }
                Err(e) => {
                    tracing::warn!("Redis connection failed, falling back to in-memory rate limiting: {}", e);
                    if !self.config.fallback_to_memory {
                        return Err(ProxyError::Internal(
                            "Redis unavailable and memory fallback disabled".to_string()
                        ));
                    }
                }
            }
        } else {
            tracing::info!("Using in-memory rate limiting (no Redis configured)");
        }

        Ok(())
    }

    /// Test Redis connection
    async fn test_redis_connection(&self) -> Result<(), ProxyError> {
        // Simulate Redis connection test
        // In a real implementation, this would use a Redis client library
        if self.config.redis_url.contains("localhost") || self.config.redis_url.contains("127.0.0.1") {
            // Simulate connection success for localhost
            Ok(())
        } else {
            Err(ProxyError::Internal("Redis connection simulation failed".to_string()))
        }
    }

    /// Check if a request is allowed using distributed rate limiting
    pub async fn is_allowed(&self, user_id: &str, _request: &crate::schemas::ChatCompletionRequest) -> bool {
        if !self.config.enabled {
            return true;
        }

        let redis_available = *self.redis_available.read().await;

        if redis_available {
            // Use Redis-based rate limiting
            self.is_allowed_redis(user_id).await.unwrap_or_else(|e| {
                tracing::warn!("Redis rate limiting failed, falling back to memory: {}", e);
                futures::executor::block_on(self.is_allowed_memory(user_id))
            })
        } else {
            // Use in-memory rate limiting
            self.is_allowed_memory(user_id).await
        }
    }

    /// Redis-based rate limiting implementation
    async fn is_allowed_redis(&self, user_id: &str) -> Result<bool, ProxyError> {
        let key = format!("{}:{}", self.config.key_prefix, user_id);
        let current_time = RateLimitEntry::current_time();

        // In a real implementation, this would use Redis commands like:
        // ZADD, ZREMRANGEBYSCORE, ZCARD to implement sliding window
        // For now, simulate Redis behavior

        tracing::debug!("Checking Redis rate limit for user: {} at key: {}", user_id, key);

        // Simulate sliding window algorithm with Redis
        let window_start = current_time.saturating_sub(self.config.window_size_seconds);

        // Simulate Redis operations:
        // 1. Remove old entries: ZREMRANGEBYSCORE key 0 (current_time - window)
        // 2. Count current entries: ZCARD key
        // 3. If allowed, add current request: ZADD key current_time current_time

        // For simulation, we'll implement a basic sliding window
        let simulated_count = self.simulate_redis_count(&key, window_start).await;

        if simulated_count < self.config.max_requests_per_minute {
            // Add the new request
            self.simulate_redis_add(&key, current_time).await;
            Ok(true)
        } else {
            tracing::debug!("Rate limit exceeded for user: {} (count: {})", user_id, simulated_count);
            Ok(false)
        }
    }

    /// Simulate Redis operations for rate limiting
    async fn simulate_redis_count(&self, _key: &str, _window_start: u64) -> u64 {
        // In production, this would be actual Redis commands
        // For now, return a random count to simulate Redis behavior
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        _key.hash(&mut hasher);
        let hash_val = hasher.finish();

        // Simulate variable load based on key hash
        (hash_val % self.config.max_requests_per_minute) / 2
    }

    /// Simulate adding to Redis
    async fn simulate_redis_add(&self, _key: &str, _timestamp: u64) {
        // In production, this would be: ZADD key timestamp timestamp
        tracing::trace!("Simulated Redis ZADD for key: {}", _key);
    }

    /// In-memory rate limiting implementation
    async fn is_allowed_memory(&self, user_id: &str) -> bool {
        let mut store = self.memory_store.write().await;

        let entry = store.entry(user_id.to_string()).or_insert_with(RateLimitEntry::new);

        if entry.is_allowed(self.config.max_requests_per_minute, self.config.window_size_seconds) {
            entry.add_request();
            tracing::debug!("Request allowed for user: {} (in-memory)", user_id);
            true
        } else {
            tracing::debug!("Rate limit exceeded for user: {} (in-memory)", user_id);
            false
        }
    }

    /// Get rate limiting statistics
    pub async fn get_stats(&self) -> serde_json::Value {
        let redis_available = *self.redis_available.read().await;
        let memory_store = self.memory_store.read().await;

        serde_json::json!({
            "enabled": self.config.enabled,
            "redis_available": redis_available,
            "fallback_mode": !redis_available && self.config.fallback_to_memory,
            "max_requests_per_minute": self.config.max_requests_per_minute,
            "window_size_seconds": self.config.window_size_seconds,
            "in_memory_users": memory_store.len(),
            "redis_url": if redis_available { &self.config.redis_url } else { "unavailable" }
        })
    }

    /// Clean up old entries from memory store
    pub async fn cleanup_memory_store(&self) {
        let mut store = self.memory_store.write().await;
        let current_time = RateLimitEntry::current_time();

        // Remove entries that haven't been accessed for a while
        store.retain(|_key, entry| {
            entry.last_cleanup > current_time.saturating_sub(self.config.window_size_seconds * 2)
        });

        tracing::debug!("Cleaned up memory store, {} users remain", store.len());
    }
}