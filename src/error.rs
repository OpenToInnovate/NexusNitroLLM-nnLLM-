#[cfg(feature = "server")]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum ProxyError {
    BadRequest(String),
    Upstream(String),
    Internal(String),
    Serialization(String),
}

#[cfg(feature = "server")]
impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ProxyError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ProxyError::Upstream(msg) => (StatusCode::BAD_GATEWAY, format!("Upstream error: {}", msg)),
            ProxyError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", msg)),
            ProxyError::Serialization(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialization error: {}", msg)),
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "type": "proxy_error",
                "code": null
            }
        }));

        (status, body).into_response()
    }
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ProxyError::Upstream(msg) => write!(f, "Upstream Error: {}", msg),
            ProxyError::Internal(msg) => write!(f, "Internal Error: {}", msg),
            ProxyError::Serialization(msg) => write!(f, "Serialization Error: {}", msg),
        }
    }
}

impl std::error::Error for ProxyError {}

/// # From Trait Implementations for Better Error Handling
/// 
/// These implementations allow automatic conversion from common error types
/// to ProxyError, making error handling more ergonomic throughout the codebase.
/// This is similar to exception hierarchies in C++ but with compile-time safety.
impl From<reqwest::Error> for ProxyError {
    /// Convert reqwest HTTP client errors to ProxyError with appropriate categorization.
    /// 
    /// This provides intelligent error classification based on the underlying
    /// HTTP error type, similar to catching specific exception types in C++.
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ProxyError::Upstream("Request timeout - backend service did not respond in time".to_string())
        } else if err.is_connect() {
            ProxyError::Upstream("Connection failed - unable to reach backend service".to_string())
        } else if err.is_request() {
            ProxyError::BadRequest(format!("Invalid request: {}", err))
        } else if err.status().is_some() {
            let status = err.status().unwrap();
            ProxyError::Upstream(format!("HTTP {}: {}", status.as_u16(), err))
        } else {
            ProxyError::Upstream(format!("HTTP client error: {}", err))
        }
    }
}

impl From<serde_json::Error> for ProxyError {
    /// Convert JSON serialization/deserialization errors to ProxyError.
    /// 
    /// This handles both parsing errors (malformed JSON) and serialization
    /// errors (invalid data structures), similar to JSON parsing exceptions in C++.
    fn from(err: serde_json::Error) -> Self {
        ProxyError::Serialization(format!("JSON error: {}", err))
    }
}

#[cfg(feature = "server")]
impl From<axum::http::Error> for ProxyError {
    /// Convert Axum HTTP errors to ProxyError.
    ///
    /// This handles HTTP protocol errors and invalid header construction,
    /// similar to HTTP library exceptions in C++ frameworks.
    fn from(err: axum::http::Error) -> Self {
        ProxyError::Internal(format!("HTTP protocol error: {}", err))
    }
}

// Note: axum::body::BodyError doesn't exist in the current version
// This implementation is commented out until the correct error type is available
// impl From<axum::body::BodyError> for ProxyError {
//     /// Convert Axum body processing errors to ProxyError.
//     /// 
//     /// This handles request/response body parsing and streaming errors,
//     /// similar to stream processing exceptions in C++.
//     fn from(err: axum::body::BodyError) -> Self {
//         ProxyError::BadRequest(format!("Body processing error: {}", err))
//     }
// }

impl From<std::io::Error> for ProxyError {
    /// Convert I/O errors to ProxyError.
    /// 
    /// This handles file system and network I/O errors, similar to
    /// std::ios::failure in C++.
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                ProxyError::BadRequest("Resource not found".to_string())
            }
            std::io::ErrorKind::PermissionDenied => {
                ProxyError::BadRequest("Permission denied".to_string())
            }
            std::io::ErrorKind::TimedOut => {
                ProxyError::Upstream("I/O operation timed out".to_string())
            }
            _ => ProxyError::Internal(format!("I/O error: {}", err))
        }
    }
}

impl From<url::ParseError> for ProxyError {
    /// Convert URL parsing errors to ProxyError.
    /// 
    /// This handles malformed URLs in configuration and requests,
    /// similar to URL parsing exceptions in C++ networking libraries.
    fn from(err: url::ParseError) -> Self {
        ProxyError::BadRequest(format!("Invalid URL: {}", err))
    }
}

impl From<uuid::Error> for ProxyError {
    /// Convert UUID generation/parsing errors to ProxyError.
    /// 
    /// This handles UUID-related errors, similar to UUID library
    /// exceptions in C++.
    fn from(err: uuid::Error) -> Self {
        ProxyError::Internal(format!("UUID error: {}", err))
    }
}