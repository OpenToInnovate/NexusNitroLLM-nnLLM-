//! Basic server example using the NexusNitroLLM library
//!
//! This example shows how to create a basic universal LLM proxy server
//! using the library with default configuration.

use nexus_nitro_llm::{Config, AppState, create_router, Result};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create configuration with defaults
    let mut config = Config::for_test();
    config.port = 8080;
    config.backend_url = "http://localhost:8000".to_string();
    config.backend_type = "lightllm".to_string();
    config.model_id = "llama".to_string();

    info!("Starting universal LLM proxy server...");
    info!("Backend URL: {}", config.backend_url);
    info!("Backend Type: {}", config.backend_type);
    info!("Default model: {}", config.model_id);

    // Create application state
    let state = AppState::new(config).await;

    // Create router with all routes
    let app = create_router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Server listening on http://{}", addr);
    info!("API endpoint: http://{}/v1/chat/completions", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service()
    ).await?;

    Ok(())
}