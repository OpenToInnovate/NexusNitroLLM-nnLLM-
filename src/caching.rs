//! # Caching Module
//!
//! Implements intelligent caching for improved performance and reduced costs.

use serde::{Deserialize, Serialize};
use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use crate::schemas::{ChatCompletionRequest, ChatCompletionResponse};
use crate::error::ProxyError;

/// # Cache Configuration
///
/// Configuration for caching behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size (number of entries)
    pub max_size: usize,
    /// Cache TTL in seconds
    pub ttl_seconds: u64,
    /// Whether caching is enabled
    pub enabled: bool,
    /// Whether to cache based on request similarity
    pub similarity_caching: bool,
    /// Minimum response size to cache (in bytes)
    pub min_response_size: usize,
    /// Cache eviction strategy
    pub eviction_strategy: EvictionStrategy,
}

/// Cache eviction strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionStrategy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// First In, First Out
    FIFO,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            ttl_seconds: 3600,
            enabled: true,
            similarity_caching: true,
            min_response_size: 100,
            eviction_strategy: EvictionStrategy::LRU,
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// Cached response
    response: ChatCompletionResponse,
    /// Timestamp when entry was created
    created_at: u64,
    /// Timestamp when entry was last accessed
    last_accessed: u64,
    /// Number of times this entry has been accessed
    access_count: u64,
    /// Entry order for FIFO eviction
    entry_order: u64,
}

impl CacheEntry {
    fn new(response: ChatCompletionResponse, entry_order: u64) -> Self {
        let now = current_timestamp();
        Self {
            response,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            entry_order,
        }
    }

    fn is_expired(&self, ttl_seconds: u64) -> bool {
        let now = current_timestamp();
        now > self.created_at + ttl_seconds
    }

    fn access(&mut self) {
        self.last_accessed = current_timestamp();
        self.access_count += 1;
    }
}

/// Get current timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
}

/// # Cache Manager
///
/// Manages caching operations with intelligent storage and eviction.
#[derive(Debug)]
pub struct CacheManager {
    /// Configuration
    config: CacheConfig,
    /// Cache storage
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Hit counter
    hit_counter: Arc<AtomicU64>,
    /// Miss counter
    miss_counter: Arc<AtomicU64>,
    /// Entry counter for FIFO ordering
    entry_counter: Arc<AtomicU64>,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            hit_counter: Arc::new(AtomicU64::new(0)),
            miss_counter: Arc::new(AtomicU64::new(0)),
            entry_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Generate cache key from request
    fn generate_cache_key(&self, request: &ChatCompletionRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash relevant request parameters for caching
        request.model.hash(&mut hasher);
        request.messages.hash(&mut hasher);

        if let Some(temp) = request.temperature {
            ((temp * 10000.0) as u64).hash(&mut hasher);
        }

        if let Some(max_tokens) = request.max_tokens {
            max_tokens.hash(&mut hasher);
        }

        if let Some(top_p) = request.top_p {
            ((top_p * 10000.0) as u64).hash(&mut hasher);
        }

        // Include stop sequences if present
        if let Some(stop) = &request.stop {
            stop.hash(&mut hasher);
        }

        format!("cache:{:x}", hasher.finish())
    }

    /// Check if response should be cached
    fn should_cache_response(&self, response: &ChatCompletionResponse) -> bool {
        if !self.config.enabled {
            return false;
        }

        // Calculate response size
        let response_size = serde_json::to_string(response)
            .map(|s| s.len())
            .unwrap_or(0);

        response_size >= self.config.min_response_size
    }

    /// Get cached response if available
    pub async fn get(&self, request: &ChatCompletionRequest) -> Option<ChatCompletionResponse> {
        if !self.config.enabled {
            return None;
        }

        let cache_key = self.generate_cache_key(request);
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(&cache_key) {
            if entry.is_expired(self.config.ttl_seconds) {
                // Remove expired entry
                cache.remove(&cache_key);
                self.miss_counter.fetch_add(1, Ordering::Relaxed);
                tracing::debug!("Cache entry expired for key: {}", cache_key);
                None
            } else {
                // Update access metadata
                entry.access();
                self.hit_counter.fetch_add(1, Ordering::Relaxed);
                tracing::debug!("Cache hit for key: {}", cache_key);
                Some(entry.response.clone())
            }
        } else {
            self.miss_counter.fetch_add(1, Ordering::Relaxed);
            tracing::debug!("Cache miss for key: {}", cache_key);
            None
        }
    }

    /// Store response in cache
    pub async fn put(&self, request: &ChatCompletionRequest, response: ChatCompletionResponse) -> Result<(), ProxyError> {
        if !self.config.enabled || !self.should_cache_response(&response) {
            return Ok(());
        }

        let cache_key = self.generate_cache_key(request);
        let entry_order = self.entry_counter.fetch_add(1, Ordering::Relaxed);
        let entry = CacheEntry::new(response, entry_order);

        let mut cache = self.cache.write().await;

        // Check if we need to evict entries
        if cache.len() >= self.config.max_size {
            self.evict_entries(&mut cache).await;
        }

        cache.insert(cache_key.clone(), entry);
        tracing::debug!("Cached response for key: {}, cache size: {}", cache_key, cache.len());

        Ok(())
    }

    /// Evict entries based on configured strategy
    async fn evict_entries(&self, cache: &mut HashMap<String, CacheEntry>) {
        if cache.is_empty() {
            return;
        }

        let entries_to_remove = (cache.len() / 4).max(1); // Remove 25% of entries
        let mut keys_to_remove = Vec::new();

        match self.config.eviction_strategy {
            EvictionStrategy::LRU => {
                // Remove least recently used entries
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.last_accessed);

                for (key, _) in entries.iter().take(entries_to_remove) {
                    keys_to_remove.push((*key).clone());
                }
            }
            EvictionStrategy::LFU => {
                // Remove least frequently used entries
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.access_count);

                for (key, _) in entries.iter().take(entries_to_remove) {
                    keys_to_remove.push((*key).clone());
                }
            }
            EvictionStrategy::FIFO => {
                // Remove oldest entries
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.entry_order);

                for (key, _) in entries.iter().take(entries_to_remove) {
                    keys_to_remove.push((*key).clone());
                }
            }
        }

        for key in keys_to_remove {
            cache.remove(&key);
        }

        tracing::debug!("Evicted {} entries using {:?} strategy", entries_to_remove, self.config.eviction_strategy);
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let initial_size = cache.len();

        cache.retain(|_, entry| !entry.is_expired(self.config.ttl_seconds));

        let removed = initial_size - cache.len();
        if removed > 0 {
            tracing::debug!("Cleaned up {} expired cache entries", removed);
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let size = cache.len();
        cache.clear();

        if size > 0 {
            tracing::info!("Cleared {} cache entries", size);
        }
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let hits = self.hit_counter.load(Ordering::Relaxed);
        let misses = self.miss_counter.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 { hits as f64 / total as f64 } else { 0.0 };

        let cache = self.cache.read().await;
        let current_size = cache.len();

        // Calculate memory usage estimate
        let memory_usage_bytes = current_size * 1024; // Rough estimate

        CacheStats {
            hits,
            misses,
            hit_rate,
            current_size,
            max_size: self.config.max_size,
            memory_usage_bytes,
            config: self.config.clone(),
        }
    }

    /// Get detailed cache information
    pub async fn get_cache_info(&self) -> serde_json::Value {
        let cache = self.cache.read().await;
        let stats = self.get_stats().await;

        let mut entries_info = Vec::new();
        for (key, entry) in cache.iter() {
            entries_info.push(serde_json::json!({
                "key": key,
                "created_at": entry.created_at,
                "last_accessed": entry.last_accessed,
                "access_count": entry.access_count,
                "is_expired": entry.is_expired(self.config.ttl_seconds)
            }));
        }

        serde_json::json!({
            "stats": stats,
            "entries": entries_info
        })
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
    /// Current number of cached entries
    pub current_size: usize,
    /// Maximum number of entries allowed
    pub max_size: usize,
    /// Estimated memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Cache configuration
    pub config: CacheConfig,
}