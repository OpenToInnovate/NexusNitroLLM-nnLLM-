//! # Request Batching Module
//!
//! Implements intelligent request batching for improved throughput and efficiency.
//! Groups multiple requests together to reduce overhead and improve performance.

use crate::{
    adapters::Adapter,
    schemas::ChatCompletionRequest,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{mpsc, oneshot, RwLock},
};
use tracing::{debug, info, warn, error};

/// # Batch Configuration
///
/// Configuration for request batching behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Maximum wait time for batching (milliseconds)
    pub max_wait_time_ms: u64,
    /// Whether to enable batching
    pub enabled: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 10,
            max_wait_time_ms: 100,
            enabled: true,
        }
    }
}

/// # Batch Request
///
/// Individual request within a batch.
pub struct BatchRequest {
    /// The chat completion request
    pub request: ChatCompletionRequest,
    /// Response channel
    pub response_tx: oneshot::Sender<Result<axum::response::Response, crate::error::ProxyError>>,
}

/// # Batch
///
/// A batch of requests to be processed together.
pub struct Batch {
    /// Requests in the batch
    pub requests: Vec<BatchRequest>,
    /// Batch creation time
    pub created_at: std::time::Instant,
}

impl Batch {
    /// Create a new batch
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            created_at: std::time::Instant::now(),
        }
    }

    /// Add a request to the batch
    pub fn add_request(&mut self, request: BatchRequest) {
        self.requests.push(request);
    }

    /// Check if the batch is ready to be processed
    pub fn is_ready(&self, config: &BatchConfig) -> bool {
        self.requests.len() >= config.max_batch_size ||
        self.created_at.elapsed().as_millis() >= config.max_wait_time_ms as u128
    }

    /// Get the number of requests in the batch
    pub fn len(&self) -> usize {
        self.requests.len()
    }
}

/// # Batch Processor
///
/// Processes batches of requests efficiently.
pub struct BatchProcessor {
    /// Configuration
    config: BatchConfig,
    /// Adapter for processing requests
    adapter: Adapter,
    /// Request counter
    request_counter: Arc<AtomicU64>,
    /// Current batch
    current_batch: Arc<RwLock<Option<Batch>>>,
    /// Batch processing channel
    batch_tx: mpsc::UnboundedSender<Batch>,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(config: BatchConfig, adapter: Adapter) -> Self {
        let (batch_tx, mut batch_rx) = mpsc::unbounded_channel();
        
        let processor = Self {
            config,
            adapter,
            request_counter: Arc::new(AtomicU64::new(0)),
            current_batch: Arc::new(RwLock::new(None)),
            batch_tx,
        };

        // Start batch processing task
        let adapter_clone = processor.adapter.clone();
        let config_clone = processor.config.clone();
        tokio::spawn(async move {
            while let Some(batch) = batch_rx.recv().await {
                if let Err(e) = Self::process_batch(batch, &adapter_clone).await {
                    error!("Failed to process batch: {}", e);
                }
            }
        });

        processor
    }

    /// Add a request to the current batch
    pub async fn add_request(&self, request: ChatCompletionRequest) -> Result<axum::response::Response, crate::error::ProxyError> {
        let (response_tx, response_rx) = oneshot::channel();
        let batch_request = BatchRequest {
            request,
            response_tx,
        };

        let mut current_batch = self.current_batch.write().await;
        
        if current_batch.is_none() {
            *current_batch = Some(Batch::new());
        }

        if let Some(ref mut batch) = *current_batch {
            batch.add_request(batch_request);
            
            if batch.is_ready(&self.config) {
                let batch_to_process = current_batch.take().unwrap();
                if let Err(e) = self.batch_tx.send(batch_to_process) {
                    error!("Failed to send batch for processing: {}", e);
                }
            }
        }

        // Wait for response
        response_rx.await.map_err(|_| crate::error::ProxyError::Internal("Batch processing failed".to_string()))?
    }

    /// Process a batch of requests
    async fn process_batch(batch: Batch, adapter: &Adapter) -> Result<(), crate::error::ProxyError> {
        info!("Processing batch with {} requests", batch.len());
        
        for batch_request in batch.requests {
            let result = adapter.chat_completions(batch_request.request).await;
            if let Err(e) = batch_request.response_tx.send(result) {
                error!("Failed to send batch response: {:?}", e);
            }
        }

        Ok(())
    }

    /// Get batch statistics
    pub fn get_stats(&self) -> BatchStats {
        BatchStats {
            total_requests: self.request_counter.load(Ordering::Relaxed),
            current_batch_size: 0, // Would need to check current batch
            config: self.config.clone(),
        }
    }
}

/// # Batch Statistics
///
/// Statistics about batch processing performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStats {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Current batch size
    pub current_batch_size: usize,
    /// Batch configuration
    pub config: BatchConfig,
}