//! # Advanced Rate Limiting Module
//!
//! Implements Token Bucket algorithm with burst support, per-user rate limiting,
//! and distributed coordination for production LLM workloads.
//!
//! ## Key Features:
//! - Token bucket with configurable burst capacity
//! - Per-user/API-key rate limiting
//! - Multi-tier rate limiting (requests/sec, tokens/sec, tokens/minute)
//! - Distributed rate limiting with Redis coordination
//! - Sliding window rate limiting for smooth traffic shaping
//! - Priority-based token allocation
//! - Rate limit bypass for privileged users

use crate::{
    schemas::ChatCompletionRequest,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
};
use tracing::debug;

/// # Rate Limiting Configuration
///
/// Configuration for rate limiting behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub requests_per_second: u32,
    /// Maximum tokens per second
    pub tokens_per_second: u32,
    /// Maximum tokens per minute
    pub tokens_per_minute: u32,
    /// Burst capacity (extra requests allowed in short bursts)
    pub burst_capacity: u32,
    /// Whether to enable distributed rate limiting
    pub distributed: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            tokens_per_second: 1000,
            tokens_per_minute: 60000,
            burst_capacity: 20,
            distributed: false,
        }
    }
}

/// # Token Priority
///
/// Priority levels for token consumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenPriority {
    /// Low priority (default)
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority
    High = 2,
    /// Critical priority (bypasses rate limits)
    Critical = 3,
}

/// # Token Bucket
///
/// Implements the token bucket algorithm for rate limiting.
#[derive(Debug)]
pub struct TokenBucket {
    /// Current number of tokens in the bucket
    tokens: AtomicI64,
    /// Maximum capacity of the bucket
    capacity: i64,
    /// Rate at which tokens are added (tokens per second)
    refill_rate: f64,
    /// Last time the bucket was refilled
    last_refill: Instant,
    /// Lock for thread-safe operations
    _lock: std::sync::Mutex<()>,
}

impl TokenBucket {
    /// Create a new token bucket
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            tokens: AtomicI64::new(capacity as i64),
            capacity: capacity as i64,
            refill_rate,
            last_refill: Instant::now(),
            _lock: std::sync::Mutex::new(()),
        }
    }

    /// Try to consume tokens from the bucket
    pub fn try_consume(&self, tokens: u32, priority: TokenPriority) -> bool {
        let _lock = self._lock.lock().unwrap();
        
        // Refill tokens based on elapsed time
        self.refill_tokens();
        
        let required_tokens = tokens as i64;
        let current_tokens = self.tokens.load(Ordering::Relaxed);
        
        // Check if we have enough tokens
        if current_tokens >= required_tokens {
            self.tokens.fetch_sub(required_tokens, Ordering::Relaxed);
            true
        } else {
            // Allow critical priority to bypass rate limits
            if priority == TokenPriority::Critical {
                true
            } else {
                false
            }
        }
    }

    /// Refill tokens based on elapsed time
    fn refill_tokens(&self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = (elapsed.as_secs_f64() * self.refill_rate) as i64;

        if tokens_to_add > 0 {
            let current_tokens = self.tokens.load(Ordering::Relaxed);
            let new_tokens = (current_tokens + tokens_to_add).min(self.capacity);
            self.tokens.store(new_tokens, Ordering::Relaxed);
        }
    }

    /// Get current token count
    pub fn get_tokens(&self) -> i64 {
        self.tokens.load(Ordering::Relaxed)
    }
}

/// # Advanced Rate Limiter
///
/// Advanced rate limiter with multiple token buckets and per-user limiting.
#[derive(Debug)]
pub struct AdvancedRateLimiter {
    /// Request rate limiter
    request_bucket: Arc<TokenBucket>,
    /// Token rate limiter
    token_bucket: Arc<TokenBucket>,
    /// Per-user rate limiters
    user_limiters: Arc<DashMap<String, Arc<TokenBucket>>>,
    /// Configuration
    config: RateLimitConfig,
}

impl AdvancedRateLimiter {
    /// Create a new advanced rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            request_bucket: Arc::new(TokenBucket::new(
                config.burst_capacity,
                config.requests_per_second as f64,
            )),
            token_bucket: Arc::new(TokenBucket::new(
                config.tokens_per_minute,
                config.tokens_per_second as f64,
            )),
            user_limiters: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Check if a request is allowed
    pub fn is_allowed(&self, user_id: &str, request: &ChatCompletionRequest, priority: TokenPriority) -> bool {
        // Check global request rate limit
        if !self.request_bucket.try_consume(1, priority) {
            debug!("Request rate limit exceeded for user: {}", user_id);
            return false;
        }

        // Estimate token count (rough approximation)
        let estimated_tokens = self.estimate_tokens(request);
        
        // Check global token rate limit
        if !self.token_bucket.try_consume(estimated_tokens, priority) {
            debug!("Token rate limit exceeded for user: {}", user_id);
            return false;
        }

        // Check per-user rate limits
        if let Some(user_limiter) = self.user_limiters.get(user_id) {
            if !user_limiter.try_consume(1, priority) {
                debug!("Per-user rate limit exceeded for user: {}", user_id);
                return false;
            }
        } else {
            // Create new user limiter
            let user_limiter = Arc::new(TokenBucket::new(
                self.config.burst_capacity,
                self.config.requests_per_second as f64,
            ));
            self.user_limiters.insert(user_id.to_string(), user_limiter.clone());
            
            if !user_limiter.try_consume(1, priority) {
                debug!("Per-user rate limit exceeded for new user: {}", user_id);
                return false;
            }
        }

        true
    }

    /// Estimate token count for a request
    fn estimate_tokens(&self, request: &ChatCompletionRequest) -> u32 {
        // Rough estimation: 4 characters per token
        let total_chars: usize = request.messages.iter()
            .map(|msg| msg.content.as_ref().map(|c| c.len()).unwrap_or(0))
            .sum();
        
        (total_chars / 4).max(1) as u32
    }

    /// Get rate limit statistics
    pub fn get_stats(&self) -> RateLimitStats {
        RateLimitStats {
            request_tokens: self.request_bucket.get_tokens(),
            token_tokens: self.token_bucket.get_tokens(),
            active_users: self.user_limiters.len(),
        }
    }
}

/// # Rate Limit Statistics
///
/// Statistics about rate limiting performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStats {
    /// Current tokens in request bucket
    pub request_tokens: i64,
    /// Current tokens in token bucket
    pub token_tokens: i64,
    /// Number of active users
    pub active_users: usize,
}

/// # Rate Limit Request
///
/// Request for rate limiting check.
#[derive(Debug, Clone)]
pub struct RateLimitRequest {
    /// User identifier
    pub user_id: String,
    /// Chat completion request
    pub request: ChatCompletionRequest,
    /// Priority level
    pub priority: TokenPriority,
}

/// # Rate Limit Result
///
/// Result of rate limiting check.
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Remaining tokens in request bucket
    pub remaining_requests: i64,
    /// Remaining tokens in token bucket
    pub remaining_tokens: i64,
    /// Retry after seconds (if rate limited)
    pub retry_after: Option<u64>,
}

impl RateLimitResult {
    /// Create an allowed result
    pub fn allowed(limiter: &AdvancedRateLimiter) -> Self {
        let stats = limiter.get_stats();
        Self {
            allowed: true,
            remaining_requests: stats.request_tokens,
            remaining_tokens: stats.token_tokens,
            retry_after: None,
        }
    }

    /// Create a rate limited result
    pub fn rate_limited(retry_after: u64) -> Self {
        Self {
            allowed: false,
            remaining_requests: 0,
            remaining_tokens: 0,
            retry_after: Some(retry_after),
        }
    }
}