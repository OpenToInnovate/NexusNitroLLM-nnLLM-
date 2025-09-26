//! # Caching Module
//!
//! Implements intelligent caching for improved performance and reduced costs.

use serde::{Deserialize, Serialize};
use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };

/// # Cache Configuration
///
/// Configuration for caching behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size
    pub max_size: usize,
    /// Cache TTL in seconds
    pub ttl_seconds: u64,
    /// Whether caching is enabled
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            ttl_seconds: 3600,
            enabled: true,
        }
    }
}

/// # Cache Manager
///
/// Manages caching operations.
#[derive(Debug)]
pub struct CacheManager {
    /// Configuration
    config: CacheConfig,
    /// Hit counter
    hit_counter: Arc<AtomicU64>,
    /// Miss counter
    miss_counter: Arc<AtomicU64>,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            hit_counter: Arc::new(AtomicU64::new(0)),
            miss_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let hits = self.hit_counter.load(Ordering::Relaxed);
        let misses = self.miss_counter.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 { hits as f64 / total as f64 } else { 0.0 };

        CacheStats {
            hits,
            misses,
            hit_rate,
            config: self.config.clone(),
        }
    }
}

/// # Cache Statistics
///
/// Statistics about cache performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Cache configuration
    pub config: CacheConfig,
}