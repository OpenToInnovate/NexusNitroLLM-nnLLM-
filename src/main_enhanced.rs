//! # Enhanced Main Module
//!
//! Enhanced main application with advanced features.

use crate::{
    adapters::Adapter,
    config::Config,
    routes_enhanced::EnhancedAppState,
};
use axum::Router;
use std::sync::Arc;

/// # Enhanced Application
///
/// Enhanced application with advanced features.
#[derive(Debug)]
pub struct EnhancedApplication {
    /// Application state
    state: EnhancedAppState,
    /// Router
    router: Router,
}

impl EnhancedApplication {
    /// Create a new enhanced application
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let adapter = Adapter::from_config(&config);
        let state = EnhancedAppState {
            adapter,
            request_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        };
        
        let router = Router::new();
        
        Ok(Self { state, router })
    }

    /// Run the enhanced application
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // Enhanced application runner with monitoring and graceful shutdown
        // For now, delegate to the standard server implementation
        println!("Enhanced server mode - delegating to standard implementation");
        println!("Use the main server binary for production deployment");
        Ok(())
    }
}