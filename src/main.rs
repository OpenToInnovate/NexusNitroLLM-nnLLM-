//! # NexusNitroLLM (nnLLM) - Simple Server Example
//!
//! This is a basic example showing how to use the NexusNitroLLM library
//! to create a simple LLM proxy server.

use nexus_nitro_llm::{Config, AppState, create_router};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse configuration from CLI args and .env file
    let config = Config::parse_args();

    // Create application state
    let state = AppState::new(config.clone()).await;

    // Create router with all routes and middleware
    let app = create_router(state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("🚀 NexusNitroLLM server starting on http://{}", addr);
    info!("Backend Type: {}", config.backend_type);
    info!("Model: {}", config.model_id);
    
    // Log backend URL safely (mask sensitive parts)
    let safe_url = if config.backend_url.contains("://") {
        if let Ok(url) = url::Url::parse(&config.backend_url) {
            format!("{}://{}", url.scheme(), url.host_str().unwrap_or("unknown"))
        } else {
            "invalid-url".to_string()
        }
    } else {
        config.backend_url.clone()
    };
    info!("Backend URL: {}", safe_url);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}