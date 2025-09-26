//! # Legacy Routes Compatibility Module
//!
//! This module provides backward compatibility for the old routes interface.
//! All functionality has been moved to the server module, but this provides
//! re-exports for existing code.

// Re-export all server functionality
pub use crate::server::routes::*;