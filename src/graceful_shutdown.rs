//! # Graceful Shutdown Module
//! 
//! This module provides graceful shutdown handling for the NexusNitroLLM server.
//! It ensures that the server shuts down cleanly without dropping active connections
//! or losing data in progress.
//! 
//! ## Key Concepts for C++ Developers:
//! 
//! - **Signal Handling**: Similar to signal handlers in C++ (SIGTERM, SIGINT)
//! - **Graceful Shutdown**: Allows ongoing requests to complete before shutdown
//! - **Connection Draining**: Stops accepting new connections while finishing existing ones
//! - **Resource Cleanup**: Ensures proper cleanup of resources like HTTP clients
//! 
//! ## Shutdown Process:
//! 
//! 1. **Signal Reception**: Receives SIGTERM, SIGINT, or other shutdown signals
//! 2. **Stop Accepting**: Stops accepting new connections
//! 3. **Drain Connections**: Waits for existing connections to complete (with timeout)
//! 4. **Cleanup Resources**: Closes HTTP clients, clears caches, etc.
//! 5. **Exit**: Exits the process cleanly

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "server")]
use tokio::signal;
#[cfg(feature = "server")]
use tokio::time::timeout;
use tracing::{info, warn, error};

/// # Graceful Shutdown Manager
/// 
/// Manages the graceful shutdown process for the server.
/// This is similar to a shutdown manager in C++ applications.
#[derive(Clone)]
pub struct GracefulShutdown {
    /// Flag indicating if shutdown has been initiated
    pub shutdown_initiated: Arc<AtomicBool>,
    /// Flag indicating if shutdown is complete
    shutdown_complete: Arc<AtomicBool>,
}

impl GracefulShutdown {
    /// # Create a new graceful shutdown manager
    /// 
    /// Creates a new graceful shutdown manager with default settings.
    /// 
    /// ## Returns:
    /// - `Self`: New graceful shutdown manager
    pub fn new() -> Self {
        Self {
            shutdown_initiated: Arc::new(AtomicBool::new(false)),
            shutdown_complete: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// # Check if shutdown has been initiated
    /// 
    /// Returns true if a shutdown signal has been received.
    /// 
    /// ## Returns:
    /// - `bool`: True if shutdown is in progress
    pub fn is_shutdown_initiated(&self) -> bool {
        self.shutdown_initiated.load(Ordering::Relaxed)
    }
    
    /// # Check if shutdown is complete
    /// 
    /// Returns true if shutdown has been completed.
    /// 
    /// ## Returns:
    /// - `bool`: True if shutdown is complete
    pub fn is_shutdown_complete(&self) -> bool {
        self.shutdown_complete.load(Ordering::Relaxed)
    }
    
    /// # Initiate shutdown
    /// 
    /// Initiates the graceful shutdown process.
    /// This should be called when a shutdown signal is received.
    pub fn initiate_shutdown(&self) {
        info!("🛑 Graceful shutdown initiated");
        self.shutdown_initiated.store(true, Ordering::Relaxed);
    }
    
    /// # Complete shutdown
    /// 
    /// Marks the shutdown process as complete.
    /// This should be called after all cleanup is finished.
    pub fn complete_shutdown(&self) {
        info!("✅ Graceful shutdown completed");
        self.shutdown_complete.store(true, Ordering::Relaxed);
    }
    
    /// # Wait for shutdown signal
    /// 
    /// Waits for a shutdown signal (SIGTERM, SIGINT, etc.) and initiates shutdown.
    /// This function blocks until a signal is received.
    /// 
    /// ## Returns:
    /// - `Result<(), Box<dyn std::error::Error>>`: Result of waiting for signal
    pub async fn wait_for_shutdown_signal(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔍 Waiting for shutdown signal...");
        
        // Create signal handlers with proper lifetimes
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;
        let mut sigquit = signal::unix::signal(signal::unix::SignalKind::quit())?;
        
        // Wait for either SIGTERM or SIGINT
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("📡 Received SIGINT (Ctrl+C)");
                self.initiate_shutdown();
            }
            _ = sigterm.recv() => {
                info!("📡 Received SIGTERM");
                self.initiate_shutdown();
            }
            _ = sigquit.recv() => {
                info!("📡 Received SIGQUIT");
                self.initiate_shutdown();
            }
        }
        
        Ok(())
    }
    
    /// # Graceful shutdown with timeout
    /// 
    /// Performs a graceful shutdown with a specified timeout.
    /// If the timeout is exceeded, the shutdown will be forced.
    /// 
    /// ## Parameters:
    /// - `shutdown_timeout`: Maximum time to wait for graceful shutdown
    /// - `cleanup_fn`: Function to call for cleanup operations
    /// 
    /// ## Returns:
    /// - `Result<(), Box<dyn std::error::Error>>`: Result of shutdown process
    pub async fn graceful_shutdown<F, Fut>(
        &self,
        shutdown_timeout: Duration,
        cleanup_fn: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
    {
        info!("🛑 Starting graceful shutdown (timeout: {:?})", shutdown_timeout);
        
        // Perform cleanup with timeout
        match timeout(shutdown_timeout, cleanup_fn()).await {
            Ok(Ok(())) => {
                info!("✅ Cleanup completed successfully");
            }
            Ok(Err(e)) => {
                error!("❌ Cleanup failed: {}", e);
                return Err(e);
            }
            Err(_) => {
                warn!("⏰ Cleanup timeout exceeded, forcing shutdown");
            }
        }
        
        self.complete_shutdown();
        Ok(())
    }
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new()
    }
}

/// # Shutdown Signal Handler
/// 
/// A convenience function that sets up signal handling and returns a shutdown receiver.
/// This is similar to setting up signal handlers in C++ applications.
/// 
/// ## Returns:
/// - `Result<GracefulShutdown, Box<dyn std::error::Error>>`: Shutdown manager
pub async fn setup_shutdown_handler() -> Result<GracefulShutdown, Box<dyn std::error::Error>> {
    let shutdown = GracefulShutdown::new();
    
    // Spawn a task to wait for shutdown signals
    let shutdown_clone = shutdown.clone();
    tokio::spawn(async move {
        if let Err(e) = shutdown_clone.wait_for_shutdown_signal().await {
            error!("❌ Error waiting for shutdown signal: {}", e);
        }
    });
    
    Ok(shutdown)
}

/// # Server Shutdown Configuration
/// 
/// Configuration for graceful shutdown behavior.
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    /// Maximum time to wait for graceful shutdown
    pub shutdown_timeout: Duration,
    /// Maximum time to wait for connections to drain
    pub drain_timeout: Duration,
    /// Whether to force shutdown if timeout is exceeded
    pub force_shutdown: bool,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            shutdown_timeout: Duration::from_secs(30),
            drain_timeout: Duration::from_secs(10),
            force_shutdown: true,
        }
    }
}

/// # Server Lifecycle Manager
/// 
/// Manages the complete lifecycle of the server including startup and shutdown.
/// This is similar to a service manager in C++ applications.
pub struct ServerLifecycle {
    shutdown: GracefulShutdown,
    config: ShutdownConfig,
}

impl ServerLifecycle {
    /// # Create a new server lifecycle manager
    /// 
    /// ## Parameters:
    /// - `config`: Shutdown configuration
    /// 
    /// ## Returns:
    /// - `Self`: New server lifecycle manager
    pub fn new(config: ShutdownConfig) -> Self {
        Self {
            shutdown: GracefulShutdown::new(),
            config,
        }
    }
    
    /// # Get the shutdown manager
    /// 
    /// ## Returns:
    /// - `&GracefulShutdown`: Reference to the shutdown manager
    pub fn shutdown(&self) -> &GracefulShutdown {
        &self.shutdown
    }
    
    /// # Start the server lifecycle
    /// 
    /// Starts the server lifecycle management including signal handling.
    /// 
    /// ## Returns:
    /// - `Result<(), Box<dyn std::error::Error>>`: Result of starting lifecycle
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Starting server lifecycle management");
        
        // Spawn signal handler
        let shutdown_clone = self.shutdown.clone();
        tokio::spawn(async move {
            if let Err(e) = shutdown_clone.wait_for_shutdown_signal().await {
                error!("❌ Error in signal handler: {}", e);
            }
        });
        
        Ok(())
    }
    
    /// # Wait for shutdown
    /// 
    /// Waits for the shutdown signal and performs graceful shutdown.
    /// 
    /// ## Parameters:
    /// - `cleanup_fn`: Function to call for cleanup
    /// 
    /// ## Returns:
    /// - `Result<(), Box<dyn std::error::Error>>`: Result of shutdown process
    pub async fn wait_for_shutdown<F, Fut>(
        &self,
        cleanup_fn: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
    {
        // Wait for shutdown signal
        while !self.shutdown.is_shutdown_initiated() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Perform graceful shutdown
        self.shutdown
            .graceful_shutdown(self.config.shutdown_timeout, cleanup_fn)
            .await
    }
}

impl Default for ServerLifecycle {
    fn default() -> Self {
        Self::new(ShutdownConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_graceful_shutdown_creation() {
        let shutdown = GracefulShutdown::new();
        assert!(!shutdown.is_shutdown_initiated());
        assert!(!shutdown.is_shutdown_complete());
    }
    
    #[tokio::test]
    async fn test_shutdown_initiation() {
        let shutdown = GracefulShutdown::new();
        shutdown.initiate_shutdown();
        assert!(shutdown.is_shutdown_initiated());
        assert!(!shutdown.is_shutdown_complete());
    }
    
    #[tokio::test]
    async fn test_shutdown_completion() {
        let shutdown = GracefulShutdown::new();
        shutdown.initiate_shutdown();
        shutdown.complete_shutdown();
        assert!(shutdown.is_shutdown_initiated());
        assert!(shutdown.is_shutdown_complete());
    }
    
    #[tokio::test]
    async fn test_graceful_shutdown_with_cleanup() {
        let shutdown = GracefulShutdown::new();
        shutdown.initiate_shutdown();
        
        let result = shutdown
            .graceful_shutdown(Duration::from_secs(1), || async {
                // Simulate cleanup
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(())
            })
            .await;
        
        assert!(result.is_ok());
        assert!(shutdown.is_shutdown_complete());
    }
    
    #[tokio::test]
    async fn test_graceful_shutdown_timeout() {
        let shutdown = GracefulShutdown::new();
        shutdown.initiate_shutdown();
        
        let result = shutdown
            .graceful_shutdown(Duration::from_millis(100), || async {
                // Simulate long cleanup that exceeds timeout
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(())
            })
            .await;
        
        // Should still succeed even with timeout
        assert!(result.is_ok());
        assert!(shutdown.is_shutdown_complete());
    }
}
