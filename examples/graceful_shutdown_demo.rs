//! # Graceful Shutdown Demo
//! 
//! This example demonstrates the graceful shutdown functionality of NexusNitroLLM.
//! It shows how the server handles shutdown signals and performs cleanup operations.

use nexus_nitro_llm::{
    graceful_shutdown::{ServerLifecycle, ShutdownConfig},
    config::Config,
    routes::AppState,
};
use axum::{
    routing::get,
    Router,
    response::Json,
};
use serde_json::json;
use std::time::Duration;
use tracing::{info, warn};

/// # Simple health check endpoint
/// 
/// Returns a simple health status for testing.
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "graceful-shutdown-demo",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// # Long running endpoint
/// 
/// Simulates a long-running operation that takes time to complete.
async fn long_operation() -> Json<serde_json::Value> {
    info!("🔄 Starting long operation...");
    
    // Simulate work that takes 5 seconds
    for i in 1..=5 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        info!("⏳ Long operation progress: {}/5", i);
    }
    
    info!("✅ Long operation completed");
    Json(json!({
        "status": "completed",
        "message": "Long operation finished successfully",
        "duration": "5 seconds"
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 Starting Graceful Shutdown Demo");
    info!("==================================");

    // Create a minimal configuration for the demo
    let config = Config::for_test();
    let state = AppState::new(config).await;

    // Create a simple router with demo endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/long-operation", get(long_operation))
        .with_state(state);

    // Set up graceful shutdown with custom configuration
    let shutdown_config = ShutdownConfig {
        shutdown_timeout: Duration::from_secs(15), // 15 second timeout
        drain_timeout: Duration::from_secs(5),     // 5 second drain timeout
        force_shutdown: true,                      // Force shutdown if timeout exceeded
    };
    
    let lifecycle = ServerLifecycle::new(shutdown_config);
    
    // Start the lifecycle management
    lifecycle.start().await?;
    
    info!("🌐 Server will start on http://localhost:3000");
    info!("📡 Send SIGTERM or SIGINT (Ctrl+C) to test graceful shutdown");
    info!("🔗 Try: curl http://localhost:3000/health");
    info!("🔗 Try: curl http://localhost:3000/long-operation");
    info!("==================================");

    // Bind to localhost:3000 for the demo
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    
    // Create a shutdown signal for the server
    let shutdown_initiated = lifecycle.shutdown().shutdown_initiated.clone();
    let shutdown_signal = async move {
        // Wait for shutdown signal
        while !shutdown_initiated.load(std::sync::atomic::Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        info!("🛑 Shutdown signal received, stopping server...");
    };
    
    // Start the server with graceful shutdown
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await
    });
    
    // Wait for shutdown and perform cleanup
    lifecycle.wait_for_shutdown(|| async {
        info!("🧹 Performing cleanup operations...");
        
        // Simulate cleanup operations
        info!("  📝 Closing database connections...");
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        info!("  🗄️  Clearing caches...");
        tokio::time::sleep(Duration::from_millis(300)).await;
        
        info!("  🔌 Closing HTTP clients...");
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        info!("  📊 Saving metrics...");
        tokio::time::sleep(Duration::from_millis(400)).await;
        
        info!("✅ All cleanup operations completed");
        Ok(())
    }).await?;
    
    // Wait for the server to finish
    match server_handle.await {
        Ok(Ok(())) => {
            info!("✅ Server stopped gracefully");
        }
        Ok(Err(e)) => {
            warn!("⚠️  Server stopped with error: {}", e);
        }
        Err(e) => {
            warn!("⚠️  Server task failed: {}", e);
        }
    }
    
    info!("👋 Graceful Shutdown Demo completed");
    Ok(())
}
