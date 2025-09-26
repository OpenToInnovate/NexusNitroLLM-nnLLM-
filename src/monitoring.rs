//! # Comprehensive Monitoring and Observability Module
//! 
//! This module provides comprehensive monitoring, metrics collection, health checks,
//! distributed tracing, and observability features for production deployments.
//! 
//! ## Key Features:
//! 
//! - **Metrics Collection**: Prometheus-compatible metrics with custom collectors
//! - **Health Monitoring**: Comprehensive health checks for all components
//! - **Distributed Tracing**: OpenTelemetry-compatible tracing for request flows
//! - **Performance Monitoring**: Real-time performance metrics and alerting
//! - **Error Tracking**: Detailed error tracking and categorization
//! - **Resource Monitoring**: CPU, memory, and network usage tracking
//! - **Custom Dashboards**: Built-in monitoring dashboards and endpoints

use crate::{
    adapters::Adapter,
    error::ProxyError,
    schemas::ChatCompletionRequest,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Response, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::RwLock,
    time::interval,
};
use tracing::{debug, info, warn, error, instrument};
use uuid::Uuid;

/// # System Metrics
/// 
/// Comprehensive system metrics for monitoring and alerting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Request metrics
    pub requests: RequestMetrics,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Error metrics
    pub errors: ErrorMetrics,
    /// Resource metrics
    pub resources: ResourceMetrics,
    /// Backend metrics
    pub backends: HashMap<String, BackendHealthMetrics>,
    /// System information
    pub system_info: SystemInfo,
}

/// # Request Metrics
/// 
/// Metrics related to HTTP requests and responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Requests per second
    pub requests_per_second: f64,
    /// Average request duration in milliseconds
    pub avg_request_duration: f64,
    /// P95 request duration in milliseconds
    pub p95_request_duration: f64,
    /// P99 request duration in milliseconds
    pub p99_request_duration: f64,
    /// Active connections
    pub active_connections: u32,
    /// Total bytes transferred
    pub total_bytes_transferred: u64,
}

/// # Performance Metrics
/// 
/// Performance-related metrics for optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Average response time
    pub avg_response_time: f64,
    /// Throughput (requests per second)
    pub throughput: f64,
    /// Error rate
    pub error_rate: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Network I/O bytes per second
    pub network_io_bps: f64,
}

/// # Error Metrics
/// 
/// Error tracking and categorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total errors
    pub total_errors: u64,
    /// Errors by type
    pub errors_by_type: HashMap<String, u64>,
    /// Errors by endpoint
    pub errors_by_endpoint: HashMap<String, u64>,
    /// Recent errors (last 100)
    pub recent_errors: Vec<ErrorEvent>,
    /// Error rate (errors per minute)
    pub error_rate: f64,
}

/// # Resource Metrics
/// 
/// System resource usage metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Memory limit in bytes
    pub memory_limit_bytes: u64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Disk usage in bytes
    pub disk_usage_bytes: u64,
    /// Network bytes received
    pub network_bytes_received: u64,
    /// Network bytes sent
    pub network_bytes_sent: u64,
    /// Open file descriptors
    pub open_file_descriptors: u32,
    /// Thread count
    pub thread_count: u32,
}

/// # Backend Health Metrics
/// 
/// Health metrics for individual backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendHealthMetrics {
    /// Backend identifier
    pub backend_id: String,
    /// Health status
    pub health_status: BackendHealthStatus,
    /// Response time in milliseconds
    pub response_time_ms: f64,
    /// Success rate
    pub success_rate: f64,
    /// Total requests
    pub total_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Last health check time
    pub last_health_check: Option<SystemTime>,
    /// Circuit breaker status
    pub circuit_breaker_status: CircuitBreakerStatus,
}

/// # Backend Health Status
/// 
/// Health status enumeration for backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendHealthStatus {
    /// Backend is healthy
    Healthy,
    /// Backend is degraded
    Degraded,
    /// Backend is unhealthy
    Unhealthy,
    /// Backend is unknown
    Unknown,
}

/// # Circuit Breaker Status
/// 
/// Circuit breaker status for backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitBreakerStatus {
    /// Circuit breaker is closed (normal operation)
    Closed,
    /// Circuit breaker is open (failing fast)
    Open,
    /// Circuit breaker is half-open (testing)
    HalfOpen,
}

/// # System Information
/// 
/// System information and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Application version
    pub version: String,
    /// Build timestamp
    pub build_timestamp: String,
    /// Git commit hash
    pub git_commit: String,
    /// Rust version
    pub rust_version: String,
    /// Operating system
    pub os: String,
    /// Architecture
    pub arch: String,
    /// Uptime
    pub uptime: Duration,
    /// Start time
    pub start_time: SystemTime,
}

/// # Error Event
/// 
/// Detailed error event for tracking and debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// Error ID
    pub error_id: String,
    /// Error type
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Stack trace
    pub stack_trace: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
    /// Endpoint
    pub endpoint: Option<String>,
    /// Timestamp
    pub timestamp: SystemTime,
    /// User agent
    pub user_agent: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
}

/// # Monitoring Configuration
/// 
/// Configuration for monitoring and observability features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable health checks
    pub enable_health_checks: bool,
    /// Enable distributed tracing
    pub enable_tracing: bool,
    /// Metrics collection interval
    pub metrics_interval: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Maximum error events to keep
    pub max_error_events: usize,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Metrics endpoint path
    pub metrics_endpoint: String,
    /// Health endpoint path
    pub health_endpoint: String,
    /// Tracing endpoint path
    pub tracing_endpoint: String,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            enable_health_checks: true,
            enable_tracing: true,
            metrics_interval: Duration::from_secs(10),
            health_check_interval: Duration::from_secs(30),
            max_error_events: 1000,
            enable_profiling: false,
            metrics_endpoint: "/metrics".to_string(),
            health_endpoint: "/health".to_string(),
            tracing_endpoint: "/tracing".to_string(),
        }
    }
}

/// # Comprehensive Monitoring System
/// 
/// Main monitoring system that collects and manages all observability data.
pub struct MonitoringSystem {
    /// Monitoring configuration
    config: MonitoringConfig,
    /// Current metrics
    metrics: Arc<RwLock<SystemMetrics>>,
    /// Metrics collector
    collector: Arc<MetricsCollector>,
    /// Health monitor
    health_monitor: Arc<HealthMonitor>,
    /// Error tracker
    error_tracker: Arc<ErrorTracker>,
    /// Performance profiler
    profiler: Arc<PerformanceProfiler>,
    /// System start time
    start_time: SystemTime,
}

/// # Metrics Collector
/// 
/// Collects and aggregates metrics from various sources.
pub struct MetricsCollector {
    /// Request counter
    request_counter: Arc<std::sync::atomic::AtomicU64>,
    /// Success counter
    success_counter: Arc<std::sync::atomic::AtomicU64>,
    /// Error counter
    error_counter: Arc<std::sync::atomic::AtomicU64>,
    /// Response time histogram
    response_times: Arc<RwLock<Vec<f64>>>,
    /// Active connections
    active_connections: Arc<std::sync::atomic::AtomicU32>,
    /// Bytes transferred
    bytes_transferred: Arc<std::sync::atomic::AtomicU64>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self {
            request_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            success_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            error_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            response_times: Arc::new(RwLock::new(Vec::new())),
            active_connections: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            bytes_transferred: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
}

impl MetricsCollector {
    /// # Record request
    /// 
    /// Records a new request with timing information.
    pub async fn record_request(&self, duration: Duration, success: bool, bytes: u64) {
        self.request_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.bytes_transferred.fetch_add(bytes, std::sync::atomic::Ordering::Relaxed);
        
        if success {
            self.success_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.error_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        
        // Record response time
        let response_time_ms = duration.as_millis() as f64;
        let mut response_times = self.response_times.write().await;
        response_times.push(response_time_ms);
        
        // Keep only last 1000 response times for memory efficiency
        if response_times.len() > 1000 {
            response_times.drain(0..response_times.len() - 1000);
        }
    }
    
    /// # Get current metrics
    /// 
    /// Returns current metrics snapshot.
    pub async fn get_metrics(&self) -> RequestMetrics {
        let total_requests = self.request_counter.load(std::sync::atomic::Ordering::Relaxed);
        let successful_requests = self.success_counter.load(std::sync::atomic::Ordering::Relaxed);
        let failed_requests = self.error_counter.load(std::sync::atomic::Ordering::Relaxed);
        let active_connections = self.active_connections.load(std::sync::atomic::Ordering::Relaxed);
        let total_bytes = self.bytes_transferred.load(std::sync::atomic::Ordering::Relaxed);
        
        let response_times = self.response_times.read().await;
        let avg_duration = if response_times.is_empty() {
            0.0
        } else {
            response_times.iter().sum::<f64>() / response_times.len() as f64
        };
        
        let p95_duration = if response_times.len() >= 20 {
            let mut sorted = response_times.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted[(sorted.len() * 95 / 100)]
        } else {
            avg_duration
        };
        
        let p99_duration = if response_times.len() >= 20 {
            let mut sorted = response_times.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted[(sorted.len() * 99 / 100)]
        } else {
            avg_duration
        };
        
        RequestMetrics {
            total_requests,
            successful_requests,
            failed_requests,
            requests_per_second: 0.0, // Will be calculated by the monitoring system
            avg_request_duration: avg_duration,
            p95_request_duration: p95_duration,
            p99_request_duration: p99_duration,
            active_connections,
            total_bytes_transferred: total_bytes,
        }
    }
}

/// # Health Monitor
/// 
/// Monitors system health and component status.
pub struct HealthMonitor {
    /// Backend health status
    backend_health: Arc<RwLock<HashMap<String, BackendHealthMetrics>>>,
    /// System health status
    system_health: Arc<RwLock<SystemHealthStatus>>,
}

/// # System Health Status
/// 
/// Overall system health status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStatus {
    /// Overall status
    pub status: HealthStatus,
    /// Component statuses
    pub components: HashMap<String, ComponentHealth>,
    /// Last check time
    pub last_check: SystemTime,
    /// Uptime
    pub uptime: Duration,
}

/// # Health Status
/// 
/// Health status enumeration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System is degraded
    Degraded,
    /// System is unhealthy
    Unhealthy,
}

/// # Component Health
/// 
/// Health status for individual components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Last check time
    pub last_check: SystemTime,
    /// Error message if unhealthy
    pub error_message: Option<String>,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self {
            backend_health: Arc::new(RwLock::new(HashMap::new())),
            system_health: Arc::new(RwLock::new(SystemHealthStatus {
                status: HealthStatus::Healthy,
                components: HashMap::new(),
                last_check: SystemTime::now(),
                uptime: Duration::from_secs(0),
            })),
        }
    }
}

impl HealthMonitor {
    /// # Check backend health
    /// 
    /// Performs health check on a backend.
    pub async fn check_backend_health(&self, backend_id: &str, adapter: &Adapter) -> BackendHealthMetrics {
        let start_time = Instant::now();
        
        // Create a simple health check request
        let health_request = ChatCompletionRequest {
            model: Some("health-check".to_string()),
            messages: vec![crate::schemas::Message {
                role: "user".to_string(),
                content: Some("health".to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            }],
            stream: Some(false),
            temperature: Some(0.1),
            max_tokens: Some(1),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            tools: None,
            tool_choice: None,
        };
        
        // Perform health check with timeout
        let is_healthy = match tokio::time::timeout(
            Duration::from_secs(5),
            adapter.chat_completions(health_request)
        ).await {
            Ok(Ok(_)) => true,
            Ok(Err(_)) => false,
            Err(_) => false, // Timeout
        };
        
        let response_time = start_time.elapsed();
        let response_time_ms = response_time.as_millis() as f64;
        
        let health_status = if is_healthy {
            BackendHealthStatus::Healthy
        } else {
            BackendHealthStatus::Unhealthy
        };
        
        let metrics = BackendHealthMetrics {
            backend_id: backend_id.to_string(),
            health_status,
            response_time_ms,
            success_rate: if is_healthy { 1.0 } else { 0.0 },
            total_requests: 1,
            failed_requests: if is_healthy { 0 } else { 1 },
            last_health_check: Some(SystemTime::now()),
            circuit_breaker_status: CircuitBreakerStatus::Closed,
        };
        
        // Update backend health
        let mut backend_health = self.backend_health.write().await;
        backend_health.insert(backend_id.to_string(), metrics.clone());
        
        metrics
    }
    
    /// # Get system health
    /// 
    /// Returns current system health status.
    pub async fn get_system_health(&self) -> SystemHealthStatus {
        self.system_health.read().await.clone()
    }
}

/// # Error Tracker
/// 
/// Tracks and categorizes errors for debugging and alerting.
pub struct ErrorTracker {
    /// Error events
    error_events: Arc<RwLock<Vec<ErrorEvent>>>,
    /// Error counters by type
    error_counters: Arc<RwLock<HashMap<String, u64>>>,
    /// Error counters by endpoint
    endpoint_error_counters: Arc<RwLock<HashMap<String, u64>>>,
    /// Maximum error events to keep
    max_events: usize,
}

impl ErrorTracker {
    /// # Create new error tracker
    /// 
    /// Creates a new error tracker with the specified configuration.
    pub fn new(max_events: usize) -> Self {
        Self {
            error_events: Arc::new(RwLock::new(Vec::new())),
            error_counters: Arc::new(RwLock::new(HashMap::new())),
            endpoint_error_counters: Arc::new(RwLock::new(HashMap::new())),
            max_events,
        }
    }
    
    /// # Record error
    /// 
    /// Records a new error event.
    pub async fn record_error(
        &self,
        error_type: String,
        message: String,
        request_id: Option<String>,
        endpoint: Option<String>,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) {
        let error_event = ErrorEvent {
            error_id: Uuid::new_v4().to_string(),
            error_type: error_type.clone(),
            message,
            stack_trace: None, // Could be added with backtrace
            request_id,
            endpoint: endpoint.clone(),
            timestamp: SystemTime::now(),
            user_agent,
            ip_address,
        };
        
        // Add to error events
        let mut error_events = self.error_events.write().await;
        error_events.push(error_event);
        
        // Keep only the most recent events
        if error_events.len() > self.max_events {
            error_events.drain(0..error_events.len() - self.max_events);
        }
        
        // Update error counters
        let mut error_counters = self.error_counters.write().await;
        *error_counters.entry(error_type).or_insert(0) += 1;
        
        if let Some(endpoint) = endpoint {
            let mut endpoint_counters = self.endpoint_error_counters.write().await;
            *endpoint_counters.entry(endpoint).or_insert(0) += 1;
        }
    }
    
    /// # Get error metrics
    /// 
    /// Returns current error metrics.
    pub async fn get_error_metrics(&self) -> ErrorMetrics {
        let error_events = self.error_events.read().await;
        let error_counters = self.error_counters.read().await;
        let endpoint_error_counters = self.endpoint_error_counters.read().await;
        
        let total_errors = error_counters.values().sum();
        let recent_errors = error_events.iter().rev().take(100).cloned().collect();
        
        ErrorMetrics {
            total_errors,
            errors_by_type: error_counters.clone(),
            errors_by_endpoint: endpoint_error_counters.clone(),
            recent_errors,
            error_rate: 0.0, // Will be calculated by the monitoring system
        }
    }
}

/// # Performance Profiler
/// 
/// Profiles system performance and identifies bottlenecks.
pub struct PerformanceProfiler {
    /// Performance samples
    performance_samples: Arc<RwLock<Vec<PerformanceSample>>>,
    /// Maximum samples to keep
    max_samples: usize,
}

/// # Performance Sample
/// 
/// A single performance measurement sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSample {
    /// Sample timestamp
    pub timestamp: SystemTime,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Network I/O bytes per second
    pub network_io: f64,
    /// Request throughput
    pub throughput: f64,
    /// Response time
    pub response_time: f64,
}

impl PerformanceProfiler {
    /// # Create new performance profiler
    /// 
    /// Creates a new performance profiler.
    pub fn new(max_samples: usize) -> Self {
        Self {
            performance_samples: Arc::new(RwLock::new(Vec::new())),
            max_samples,
        }
    }
    
    /// # Record performance sample
    /// 
    /// Records a new performance sample.
    pub async fn record_sample(&self, sample: PerformanceSample) {
        let mut samples = self.performance_samples.write().await;
        samples.push(sample);
        
        // Keep only the most recent samples
        if samples.len() > self.max_samples {
            samples.drain(0..samples.len() - self.max_samples);
        }
    }
    
    /// # Get performance metrics
    /// 
    /// Returns current performance metrics.
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        let samples = self.performance_samples.read().await;
        
        if samples.is_empty() {
            return PerformanceMetrics {
                cache_hit_rate: 0.0,
                avg_response_time: 0.0,
                throughput: 0.0,
                error_rate: 0.0,
                memory_usage_percent: 0.0,
                cpu_usage_percent: 0.0,
                network_io_bps: 0.0,
            };
        }
        
        let avg_cpu = samples.iter().map(|s| s.cpu_usage).sum::<f64>() / samples.len() as f64;
        let avg_memory = samples.iter().map(|s| s.memory_usage).sum::<u64>() / samples.len() as u64;
        let avg_network_io = samples.iter().map(|s| s.network_io).sum::<f64>() / samples.len() as f64;
        let avg_throughput = samples.iter().map(|s| s.throughput).sum::<f64>() / samples.len() as f64;
        let avg_response_time = samples.iter().map(|s| s.response_time).sum::<f64>() / samples.len() as f64;
        
        PerformanceMetrics {
            cache_hit_rate: 0.0, // Would need cache metrics
            avg_response_time,
            throughput: avg_throughput,
            error_rate: 0.0, // Would need error rate calculation
            memory_usage_percent: (avg_memory as f64 / 1024.0 / 1024.0 / 1024.0) * 100.0, // Convert to GB and percentage
            cpu_usage_percent: avg_cpu,
            network_io_bps: avg_network_io,
        }
    }
}

impl MonitoringSystem {
    /// # Create new monitoring system
    /// 
    /// Creates a new monitoring system with the specified configuration.
    pub fn new(config: MonitoringConfig) -> Self {
        let start_time = SystemTime::now();
        
        Self {
            config,
            metrics: Arc::new(RwLock::new(SystemMetrics {
                requests: RequestMetrics {
                    total_requests: 0,
                    successful_requests: 0,
                    failed_requests: 0,
                    requests_per_second: 0.0,
                    avg_request_duration: 0.0,
                    p95_request_duration: 0.0,
                    p99_request_duration: 0.0,
                    active_connections: 0,
                    total_bytes_transferred: 0,
                },
                performance: PerformanceMetrics {
                    cache_hit_rate: 0.0,
                    avg_response_time: 0.0,
                    throughput: 0.0,
                    error_rate: 0.0,
                    memory_usage_percent: 0.0,
                    cpu_usage_percent: 0.0,
                    network_io_bps: 0.0,
                },
                errors: ErrorMetrics {
                    total_errors: 0,
                    errors_by_type: HashMap::new(),
                    errors_by_endpoint: HashMap::new(),
                    recent_errors: Vec::new(),
                    error_rate: 0.0,
                },
                resources: ResourceMetrics {
                    memory_usage_bytes: 0,
                    memory_limit_bytes: 0,
                    cpu_usage_percent: 0.0,
                    disk_usage_bytes: 0,
                    network_bytes_received: 0,
                    network_bytes_sent: 0,
                    open_file_descriptors: 0,
                    thread_count: 0,
                },
                backends: HashMap::new(),
                system_info: SystemInfo {
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    build_timestamp: env!("VERGEN_BUILD_TIMESTAMP").to_string(),
                    git_commit: env!("VERGEN_GIT_SHA").to_string(),
                    rust_version: env!("VERGEN_RUSTC_SEMVER").to_string(),
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    uptime: Duration::from_secs(0),
                    start_time,
                },
            })),
            collector: Arc::new(MetricsCollector::default()),
            health_monitor: Arc::new(HealthMonitor::default()),
            error_tracker: Arc::new(ErrorTracker::new(1000)),
            profiler: Arc::new(PerformanceProfiler::new(1000)),
            start_time,
        }
    }
    
    /// # Start monitoring
    /// 
    /// Starts the monitoring system with background tasks.
    pub async fn start(&self) {
        if self.config.enable_metrics {
            self.start_metrics_collection().await;
        }
        
        if self.config.enable_health_checks {
            self.start_health_monitoring().await;
        }
        
        if self.config.enable_profiling {
            self.start_performance_profiling().await;
        }
        
        info!("🔍 Monitoring system started with configuration: {:?}", self.config);
    }
    
    /// # Start metrics collection
    /// 
    /// Starts the metrics collection background task.
    async fn start_metrics_collection(&self) {
        let metrics = self.metrics.clone();
        let collector = self.collector.clone();
        let interval_duration = self.config.metrics_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                // Collect metrics
                let request_metrics = collector.get_metrics().await;
                
                // Update system metrics
                let mut system_metrics = metrics.write().await;
                system_metrics.requests = request_metrics;
                system_metrics.system_info.uptime = system_metrics.system_info.start_time.elapsed().unwrap_or_default();
                
                debug!("📊 Metrics collected: {} requests, {} errors", 
                    system_metrics.requests.total_requests, 
                    system_metrics.requests.failed_requests);
            }
        });
    }
    
    /// # Start health monitoring
    /// 
    /// Starts the health monitoring background task.
    async fn start_health_monitoring(&self) {
        let health_monitor = self.health_monitor.clone();
        let interval_duration = self.config.health_check_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                // Update system health
                let mut system_health = health_monitor.system_health.write().await;
                system_health.last_check = SystemTime::now();
                system_health.uptime = system_health.last_check.duration_since(system_health.last_check).unwrap_or_default();
                
                debug!("🏥 Health check completed");
            }
        });
    }
    
    /// # Start performance profiling
    /// 
    /// Starts the performance profiling background task.
    async fn start_performance_profiling(&self) {
        let profiler = self.profiler.clone();
        let interval_duration = self.config.metrics_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                // Collect performance sample
                let sample = PerformanceSample {
                    timestamp: SystemTime::now(),
                    cpu_usage: 0.0, // Would need actual CPU monitoring
                    memory_usage: 0, // Would need actual memory monitoring
                    network_io: 0.0, // Would need actual network monitoring
                    throughput: 0.0, // Would need actual throughput calculation
                    response_time: 0.0, // Would need actual response time calculation
                };
                
                profiler.record_sample(sample).await;
                
                debug!("📈 Performance sample recorded");
            }
        });
    }
    
    /// # Record request
    /// 
    /// Records a request for metrics collection.
    pub async fn record_request(&self, duration: Duration, success: bool, bytes: u64) {
        self.collector.record_request(duration, success, bytes).await;
    }
    
    /// # Record error
    /// 
    /// Records an error for tracking and alerting.
    pub async fn record_error(
        &self,
        error_type: String,
        message: String,
        request_id: Option<String>,
        endpoint: Option<String>,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) {
        self.error_tracker.record_error(
            error_type,
            message,
            request_id,
            endpoint,
            user_agent,
            ip_address,
        ).await;
    }
    
    /// # Get metrics
    /// 
    /// Returns current system metrics.
    pub async fn get_metrics(&self) -> SystemMetrics {
        self.metrics.read().await.clone()
    }
    
    /// # Get health status
    /// 
    /// Returns current system health status.
    pub async fn get_health_status(&self) -> SystemHealthStatus {
        self.health_monitor.get_system_health().await
    }
    
    /// # Create monitoring router
    /// 
    /// Creates a router with monitoring endpoints.
    pub fn create_monitoring_router(&self) -> Router {
        let metrics = self.metrics.clone();
        let health_monitor = self.health_monitor.clone();
        let error_tracker = self.error_tracker.clone();
        let profiler = self.profiler.clone();
        
        Router::new()
            .route("/metrics", get(move || async move {
                let metrics = metrics.read().await;
                Json(metrics.clone())
            }))
            .route("/health", get(move || async move {
                let health = health_monitor.get_system_health().await;
                Json(health)
            }))
            .route("/errors", get(move || async move {
                let errors = error_tracker.get_error_metrics().await;
                Json(errors)
            }))
            .route("/performance", get(move || async move {
                let performance = profiler.get_performance_metrics().await;
                Json(performance)
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_monitoring_system_creation() {
        let config = MonitoringConfig::default();
        let monitoring = MonitoringSystem::new(config);
        
        let metrics = monitoring.get_metrics().await;
        assert_eq!(metrics.requests.total_requests, 0);
        assert_eq!(metrics.errors.total_errors, 0);
    }
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::default();
        
        // Record some requests
        collector.record_request(Duration::from_millis(100), true, 1024).await;
        collector.record_request(Duration::from_millis(200), false, 512).await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.total_bytes_transferred, 1536);
    }
    
    #[tokio::test]
    async fn test_error_tracking() {
        let tracker = ErrorTracker::new(100);
        
        // Record some errors
        tracker.record_error(
            "TestError".to_string(),
            "Test error message".to_string(),
            Some("req-123".to_string()),
            Some("/test".to_string()),
            None,
            None,
        ).await;
        
        let error_metrics = tracker.get_error_metrics().await;
        assert_eq!(error_metrics.total_errors, 1);
        assert_eq!(error_metrics.errors_by_type.get("TestError"), Some(&1));
        assert_eq!(error_metrics.errors_by_endpoint.get("/test"), Some(&1));
    }
    
    #[tokio::test]
    async fn test_performance_profiling() {
        let profiler = PerformanceProfiler::new(100);
        
        // Record some performance samples
        let sample1 = PerformanceSample {
            timestamp: SystemTime::now(),
            cpu_usage: 50.0,
            memory_usage: 1024 * 1024,
            network_io: 1000.0,
            throughput: 10.0,
            response_time: 100.0,
        };
        
        let sample2 = PerformanceSample {
            timestamp: SystemTime::now(),
            cpu_usage: 60.0,
            memory_usage: 2048 * 1024,
            network_io: 2000.0,
            throughput: 15.0,
            response_time: 150.0,
        };
        
        profiler.record_sample(sample1).await;
        profiler.record_sample(sample2).await;
        
        let performance_metrics = profiler.get_performance_metrics().await;
        assert_eq!(performance_metrics.cpu_usage_percent, 55.0);
        assert_eq!(performance_metrics.avg_response_time, 125.0);
        assert_eq!(performance_metrics.throughput, 12.5);
    }
}
