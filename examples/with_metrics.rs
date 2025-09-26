//! Example showing metrics collection and monitoring
//!
//! This example demonstrates how to use the metrics system
//! to monitor proxy performance.

use nexus_nitro_llm::{Config, AppState, create_router, LLMMetrics, Result};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::time::interval;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create configuration with metrics enabled
    let mut config = Config::for_test();
    config.port = 8080;
    config.backend_url = "http://localhost:8000".to_string();
    config.model_id = "llama".to_string();
    config.enable_metrics = true;

    // Create application state
    let state = AppState::new(config).await;

    // Clone metrics for monitoring task
    let metrics = Arc::new(LLMMetrics::default());
    let metrics_monitor = Arc::clone(&metrics);

    // Start metrics monitoring task
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            let stats = &*metrics_monitor;
            info!("=== Metrics Report ===");
            info!("Total requests: {}", stats.total_requests);
            info!("Successful requests: {}", stats.successful_requests);
            info!("Failed requests: {}", stats.failed_requests);
            info!("Average response time: {:.2}ms", stats.avg_response_time_ms);
            info!("Total tokens processed: {}", stats.total_tokens);
            info!("Requests per second: {:.2}", stats.requests_per_second);
            info!("=====================");
        }
    });

    // Create router
    let app = create_router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Server with metrics listening on http://{}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service()
    ).await?;

    Ok(())
}