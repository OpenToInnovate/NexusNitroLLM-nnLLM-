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
    Router,
    extract::{Request, State},
    middleware::{self, Next},
    response::Response as AxumResponse,
    http::{StatusCode, HeaderMap},
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{self, TraceLayer},
    compression::CompressionLayer,
};
use tracing::Level;

/// API key validation middleware
async fn api_key_validation(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<AxumResponse, StatusCode> {
    // Check if API key validation is enabled
    if !state.config.api_key_validation_enabled {
        return Ok(next.run(request).await);
    }

    // Skip validation for health check and UI routes
    let path = request.uri().path();
    if path.starts_with("/health") ||
       path.starts_with("/ui") ||
       path.starts_with("/v1/ui") ||
       path.starts_with("/sso") ||
       path.starts_with("/login") ||
       path.starts_with("/litellm") ||
       path.starts_with("/.well-known") ||
       path == "/favicon.ico" {
        return Ok(next.run(request).await);
    }

    // Get the API key from the configured header
    let api_key_header = &state.config.api_key_header;
    let api_key = headers.get(api_key_header)
        .and_then(|h| h.to_str().ok())
        .or_else(|| {
            // Also check Authorization header with Bearer prefix
            headers.get("authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|auth| {
                    if auth.starts_with("Bearer ") {
                        Some(&auth[7..])
                    } else {
                        None
                    }
                })
        });

    // Check if API key is provided
    let api_key = match api_key {
        Some(key) if !key.is_empty() => key,
        _ => {
            tracing::warn!("API key validation failed: missing or empty API key");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Validate the API key
    if !is_valid_api_key(&state, api_key).await {
        tracing::warn!("API key validation failed: invalid key");
        return Err(StatusCode::UNAUTHORIZED);
    }

    tracing::debug!("API key validation successful");
    Ok(next.run(request).await)
}

/// Check if the provided API key is valid
async fn is_valid_api_key(state: &AppState, api_key: &str) -> bool {
    // In a production system, this would check against a database or key store
    // For now, we'll implement a simple validation scheme:

    // 1. Check if it matches the backend token (if configured)
    if let Some(ref backend_token) = state.config.backend_token {
        if api_key == backend_token {
            return true;
        }
    }

    // 2. Check against environment variables for valid API keys
    if let Ok(valid_keys) = std::env::var("VALID_API_KEYS") {
        for valid_key in valid_keys.split(',') {
            if api_key == valid_key.trim() {
                return true;
            }
        }
    }

    // 3. Check for common development keys (only in development mode)
    if state.config.environment == "development" {
        let dev_keys = ["dev-key", "test-key", "local-key"];
        if dev_keys.contains(&api_key) {
            return true;
        }
    }

    // 4. For demonstration, accept any key that looks like an OpenAI key format
    if api_key.starts_with("sk-") && api_key.len() > 20 {
        return true;
    }

    false
}

/// Create router with all routes and middleware
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Main API endpoint for chat completions
        .route("/v1/chat/completions", post(chat_completions))
        
        // Anthropic API compatibility endpoint
        .route("/v1/messages", post(handlers::anthropic_messages))

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

        // Add API key validation middleware (applied first, before other middleware)
        .layer(middleware::from_fn_with_state(state.clone(), api_key_validation))

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