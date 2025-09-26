//! # Server Module
//!
//! This module consolidates all server-related functionality including
//! routes, handlers, and middleware. It replaces the separate routes.rs
//! and routes_enhanced.rs files with a unified server implementation.

pub mod routes;
pub mod handlers;
pub mod state;

// Re-export commonly used server types
pub use handlers::{chat_completions, ui_proxy, login_proxy};
pub use state::AppState;

use axum::{
    routing::{any, get, post},
    Router
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{self, TraceLayer},
    compression::CompressionLayer,
};
use tracing::Level;

/// Create router with all routes and middleware
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Main API endpoint for chat completions
        .route("/v1/chat/completions", post(chat_completions))

        // Health check endpoints for production monitoring
        .route("/health", get(handlers::health_check))

        // UI proxy routes - these forward requests to the backend LightLLM server
        .route("/v1/ui", any(ui_proxy))
        .route("/v1/ui/{*path}", any(ui_proxy))
        .route("/ui", any(ui_proxy))
        .route("/ui/{*path}", any(ui_proxy))

        // Authentication and SSO routes
        .route("/sso/{*path}", any(ui_proxy))
        .route("/login", any(login_proxy))

        // Static asset routes
        .route("/litellm-asset-prefix/{*path}", any(ui_proxy))
        .route("/.well-known/{*path}", any(ui_proxy))
        .route("/litellm/{*path}", any(ui_proxy))
        .route("/favicon.ico", any(ui_proxy))

        // Add middleware stack
        .layer(
            ServiceBuilder::new()
                // Compression middleware - automatically compresses responses
                .layer(CompressionLayer::new())

                // Tracing middleware - logs HTTP requests and responses
                .layer(TraceLayer::new_for_http()
                    .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(trace::DefaultOnResponse::new().level(Level::INFO)))

                // CORS middleware - allows cross-origin requests
                .layer(CorsLayer::permissive()),
        )
        // Inject application state into all handlers
        .with_state(state)
}