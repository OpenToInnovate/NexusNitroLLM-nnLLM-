//! # Server Handlers
//!
//! This module contains HTTP route handlers for the server.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{Response, IntoResponse, Json as JsonResponse},
    Json,
};
use crate::{
    error::ProxyError,
    schemas::ChatCompletionRequest,
};
#[cfg(feature = "streaming")]
use crate::streaming::create_streaming_response;
use super::AppState;

/// Chat completions handler
pub async fn chat_completions(
    State(state): State<AppState>,
    Json(req): Json<ChatCompletionRequest>,
) -> Result<Response, ProxyError> {
    // Check if streaming is requested
    if req.stream.unwrap_or(false) {
        // Check if the adapter supports streaming
        if state.adapter().supports_streaming() {
            #[cfg(feature = "streaming")]
            {
                let sse_response = create_streaming_response(state.adapter(), req).await?;
                Ok(sse_response.into_response())
            }
            #[cfg(not(feature = "streaming"))]
            {
                Err(ProxyError::BadRequest(
                    "Streaming not compiled in this build".to_string()
                ))
            }
        } else {
            Err(ProxyError::BadRequest(
                "stream=true unsupported for this adapter".to_string()
            ))
        }
    } else {
        // Return regular JSON response
        state.adapter().chat_completions(req).await
    }
}

/// Health check handler
pub async fn health_check() -> impl IntoResponse {
    let health_status = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "nexus-nitro-llm",
        "version": env!("CARGO_PKG_VERSION")
    });

    (StatusCode::OK, JsonResponse(health_status))
}

/// UI proxy handler
pub async fn ui_proxy(
    State(state): State<AppState>,
    method: Method,
    Path(path): Path<String>,
    headers: HeaderMap,
    body: axum::body::Body,
) -> Result<Response, ProxyError> {
    // Extract base URL without the /v1 suffix for UI routes
    let base_url = state.config().backend_url.trim_end_matches("/v1").trim_end_matches("/");

    // Determine the target URL based on the request path
    let target_url = if path.starts_with("_next/") || path.starts_with("litellm-asset-prefix/") || path.starts_with("litellm-ui-config") || path.starts_with(".well-known/") {
        format!("{}/{}", base_url, path)
    } else if path.is_empty() {
        format!("{}/favicon.ico", base_url)
    } else if path.starts_with("key/generate") {
        format!("{}/sso/{}", base_url, path)
    } else if path == "login" || path.starts_with("login") {
        format!("{}/{}", base_url, path)
    } else {
        format!("{}/ui/{}", base_url, path)
    };

    // Build the HTTP request using the shared client
    let mut request_builder = state.http_client().request(method.clone(), &target_url);

    // Forward relevant headers (excluding host to avoid conflicts)
    for (name, value) in headers.iter() {
        if name != "host" {
            request_builder = request_builder.header(name, value);
        }
    }

    // Convert axum body to reqwest body
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await
        .map_err(|e| ProxyError::BadRequest(format!("Failed to read request body: {}", e)))?;

    // Only add body if it's not empty
    if !body_bytes.is_empty() {
        request_builder = request_builder.body(body_bytes);
    }

    // Send the request and await the response
    let response = request_builder.send().await
        .map_err(|e| ProxyError::Upstream(format!("UI proxy request failed: {}", e)))?;

    let status = response.status();
    let mut response_builder = axum::http::Response::builder().status(status);

    // Forward response headers
    for (name, value) in response.headers().iter() {
        response_builder = response_builder.header(name, value);
    }

    // Read the response body
    let body = response.bytes().await
        .map_err(|e| ProxyError::Upstream(format!("Failed to read response body: {}", e)))?;

    // Build and return the response
    response_builder.body(axum::body::Body::from(body))
        .map_err(|e| ProxyError::Upstream(format!("Failed to build response: {}", e)))
}

/// Login proxy handler
pub async fn login_proxy(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    body: axum::body::Body,
) -> Result<Response, ProxyError> {
    let base_url = state.config().backend_url.trim_end_matches("/v1").trim_end_matches("/");
    let target_url = format!("{}/login", base_url);

    let mut request_builder = state.http_client().request(method.clone(), &target_url);

    // Forward relevant headers (excluding host)
    for (name, value) in headers.iter() {
        if name != "host" {
            request_builder = request_builder.header(name, value);
        }
    }

    // Convert axum body to reqwest body
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await
        .map_err(|e| ProxyError::BadRequest(format!("Failed to read request body: {}", e)))?;

    if !body_bytes.is_empty() {
        request_builder = request_builder.body(body_bytes);
    }

    let response = request_builder.send().await
        .map_err(|e| ProxyError::Upstream(format!("UI proxy request failed: {}", e)))?;

    let status = response.status();
    let mut response_builder = axum::http::Response::builder().status(status);

    // Forward response headers
    for (name, value) in response.headers().iter() {
        response_builder = response_builder.header(name, value);
    }

    let body = response.bytes().await
        .map_err(|e| ProxyError::Upstream(format!("Failed to read response body: {}", e)))?;

    response_builder.body(axum::body::Body::from(body))
        .map_err(|e| ProxyError::Upstream(format!("Failed to build response: {}", e)))
}