//! # NexusNitroLLM (nnLLM) - Simple Server Example
//!
//! This is a basic example showing how to use the NexusNitroLLM library
//! to create a simple LLM proxy server with HTTP/2 support.

use nexus_nitro_llm::{Config, AppState, create_router};
use std::net::SocketAddr;
use tracing::info;
use hyper::server::conn::http2;
use hyper_util::rt::{TokioIo, TokioExecutor};
use tower::Service;

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
    info!("✨ HTTP/2 enabled with prior knowledge (h2c)");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let app = app.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            
            // Create a service for this connection
            let service = hyper::service::service_fn(move |req| {
                let mut app = app.clone();
                async move {
                    app.call(req).await.map_err(|e| {
                        tracing::error!("Service error: {:?}", e);
                        std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e))
                    })
                }
            });
            
            if let Err(err) = http2::Builder::new(TokioExecutor::new())
                .serve_connection(io, service)
                .await
            {
                tracing::error!("HTTP/2 connection error: {:?}", err);
            }
        });
    }
}