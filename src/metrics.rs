//! # Metrics Collection Module
//!
//! Collects and aggregates performance metrics for monitoring and optimization.
//! Provides real-time insights into system performance and usage patterns.

use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::{Duration, Instant},
};
use tokio::time::interval;
use tracing::info;

/// # LLM Metrics
///
/// Comprehensive metrics for LLM operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMMetrics {
    /// Total number of requests
    pub total_requests: u64,
    /// Total number of successful requests
    pub successful_requests: u64,
    /// Total number of failed requests
    pub failed_requests: u64,
    /// Total tokens processed
    pub total_tokens: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Requests per second
    pub requests_per_second: f64,
    /// Tokens per second
    pub tokens_per_second: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
}

impl Default for LLMMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_tokens: 0,
            avg_response_time_ms: 0.0,
            requests_per_second: 0.0,
            tokens_per_second: 0.0,
            error_rate: 0.0,
        }
    }
}

/// # Metrics Collector
///
/// Collects and aggregates metrics from various sources.
#[derive(Debug)]
pub struct MetricsCollector {
    /// Current metrics
    metrics: Arc<RwLock<LLMMetrics>>,
    /// Request counter
    request_counter: Arc<AtomicU64>,
    /// Success counter
    success_counter: Arc<AtomicU64>,
    /// Failure counter
    failure_counter: Arc<AtomicU64>,
    /// Token counter
    token_counter: Arc<AtomicU64>,
    /// Response time accumulator
    response_time_accumulator: Arc<AtomicU64>,
    /// Response time count
    response_time_count: Arc<AtomicUsize>,
    /// Start time for rate calculations
    start_time: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(LLMMetrics::default())),
            request_counter: Arc::new(AtomicU64::new(0)),
            success_counter: Arc::new(AtomicU64::new(0)),
            failure_counter: Arc::new(AtomicU64::new(0)),
            token_counter: Arc::new(AtomicU64::new(0)),
            response_time_accumulator: Arc::new(AtomicU64::new(0)),
            response_time_count: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
        }
    }

    /// Record a request
    pub fn record_request(&self) {
        self.request_counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful request
    pub fn record_success(&self, tokens: u64, response_time_ms: u64) {
        self.success_counter.fetch_add(1, Ordering::Relaxed);
        self.token_counter.fetch_add(tokens, Ordering::Relaxed);
        self.response_time_accumulator.fetch_add(response_time_ms, Ordering::Relaxed);
        self.response_time_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed request
    pub fn record_failure(&self) {
        self.failure_counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> LLMMetrics {
        let total_requests = self.request_counter.load(Ordering::Relaxed);
        let successful_requests = self.success_counter.load(Ordering::Relaxed);
        let failed_requests = self.failure_counter.load(Ordering::Relaxed);
        let total_tokens = self.token_counter.load(Ordering::Relaxed);
        
        let response_time_sum = self.response_time_accumulator.load(Ordering::Relaxed);
        let response_time_count = self.response_time_count.load(Ordering::Relaxed);
        
        let avg_response_time_ms = if response_time_count > 0 {
            response_time_sum as f64 / response_time_count as f64
        } else {
            0.0
        };

        let elapsed_seconds = self.start_time.elapsed().as_secs_f64();
        let requests_per_second = if elapsed_seconds > 0.0 {
            total_requests as f64 / elapsed_seconds
        } else {
            0.0
        };

        let tokens_per_second = if elapsed_seconds > 0.0 {
            total_tokens as f64 / elapsed_seconds
        } else {
            0.0
        };

        let error_rate = if total_requests > 0 {
            failed_requests as f64 / total_requests as f64
        } else {
            0.0
        };

        LLMMetrics {
            total_requests,
            successful_requests,
            failed_requests,
            total_tokens,
            avg_response_time_ms,
            requests_per_second,
            tokens_per_second,
            error_rate,
        }
    }

    /// Start periodic metrics reporting
    pub fn start_reporting(&self, interval_seconds: u64) {
        let metrics = self.metrics.clone();
        let request_counter = self.request_counter.clone();
        let success_counter = self.success_counter.clone();
        let failure_counter = self.failure_counter.clone();
        let token_counter = self.token_counter.clone();
        let response_time_accumulator = self.response_time_accumulator.clone();
        let response_time_count = self.response_time_count.clone();
        let start_time = self.start_time;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_seconds));
            
            loop {
                interval.tick().await;
                
                let total_requests = request_counter.load(Ordering::Relaxed);
                let successful_requests = success_counter.load(Ordering::Relaxed);
                let failed_requests = failure_counter.load(Ordering::Relaxed);
                let total_tokens = token_counter.load(Ordering::Relaxed);
                
                let response_time_sum = response_time_accumulator.load(Ordering::Relaxed);
                let response_time_count_val = response_time_count.load(Ordering::Relaxed);
                
                let avg_response_time_ms = if response_time_count_val > 0 {
                    response_time_sum as f64 / response_time_count_val as f64
                } else {
                    0.0
                };

                let elapsed_seconds = start_time.elapsed().as_secs_f64();
                let requests_per_second = if elapsed_seconds > 0.0 {
                    total_requests as f64 / elapsed_seconds
                } else {
                    0.0
                };

                let tokens_per_second = if elapsed_seconds > 0.0 {
                    total_tokens as f64 / elapsed_seconds
                } else {
                    0.0
                };

                let error_rate = if total_requests > 0 {
                    failed_requests as f64 / total_requests as f64
                } else {
                    0.0
                };

                let current_metrics = LLMMetrics {
                    total_requests,
                    successful_requests,
                    failed_requests,
                    total_tokens,
                    avg_response_time_ms,
                    requests_per_second,
                    tokens_per_second,
                    error_rate,
                };

                {
                    let mut metrics_guard = metrics.write().unwrap();
                    *metrics_guard = current_metrics.clone();
                }

                info!(
                    "Metrics: requests={}, success={}, failed={}, tokens={}, avg_time={:.2}ms, rps={:.2}, tps={:.2}, error_rate={:.2}%",
                    total_requests,
                    successful_requests,
                    failed_requests,
                    total_tokens,
                    avg_response_time_ms,
                    requests_per_second,
                    tokens_per_second,
                    error_rate * 100.0
                );
            }
        });
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}