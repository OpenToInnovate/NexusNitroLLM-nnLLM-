//! # HTTP Client Factory
//!
//! Centralized HTTP client creation and configuration to eliminate
//! duplication across the codebase and ensure consistent client settings.

use crate::config::Config;
use reqwest::Client;
use std::time::Duration;
use thiserror::Error;

/// HTTP client configuration errors
#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("Failed to build HTTP client: {0}")]
    BuildError(#[from] reqwest::Error),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// HTTP client pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_idle_per_host: usize,
    pub idle_timeout: Duration,
    pub keepalive: Option<Duration>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 10,
            idle_timeout: Duration::from_secs(90),
            keepalive: Some(Duration::from_secs(60)),
        }
    }
}

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub pool: PoolConfig,
    pub compression: bool,
    pub http2_prior_knowledge: bool,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            pool: PoolConfig::default(),
            compression: true,
            http2_prior_knowledge: false,
        }
    }
}

impl From<&Config> for HttpClientConfig {
    fn from(config: &Config) -> Self {
        Self {
            timeout: Duration::from_secs(config.http_client_timeout),
            connect_timeout: Duration::from_secs(10),
            pool: PoolConfig {
                max_idle_per_host: config.http_client_max_connections_per_host,
                idle_timeout: Duration::from_secs(120),
                keepalive: Some(Duration::from_secs(60)),
            },
            compression: true,
            http2_prior_knowledge: false,
        }
    }
}

/// HTTP client builder with configurable options
pub struct HttpClientBuilder {
    config: HttpClientConfig,
}

impl HttpClientBuilder {
    /// Create a new HTTP client builder with default configuration
    pub fn new() -> Self {
        Self {
            config: HttpClientConfig::default(),
        }
    }

    /// Create HTTP client builder from application configuration
    pub fn from_config(config: &Config) -> Self {
        Self {
            config: HttpClientConfig::from(config),
        }
    }

    /// Create production-optimized HTTP client configuration
    pub fn production() -> Self {
        Self {
            config: HttpClientConfig {
                timeout: Duration::from_secs(30),
                connect_timeout: Duration::from_secs(10),
                pool: PoolConfig {
                    max_idle_per_host: 20,
                    idle_timeout: Duration::from_secs(120),
                    keepalive: Some(Duration::from_secs(60)),
                },
                compression: true,
                http2_prior_knowledge: true,
            },
        }
    }

    /// Create development-optimized HTTP client configuration
    pub fn development() -> Self {
        Self {
            config: HttpClientConfig {
                timeout: Duration::from_secs(60),
                connect_timeout: Duration::from_secs(15),
                pool: PoolConfig {
                    max_idle_per_host: 5,
                    idle_timeout: Duration::from_secs(60),
                    keepalive: Some(Duration::from_secs(30)),
                },
                compression: false,
                http2_prior_knowledge: false,
            },
        }
    }

    /// Set request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set connection timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Set pool configuration
    pub fn pool_config(mut self, pool: PoolConfig) -> Self {
        self.config.pool = pool;
        self
    }

    /// Enable or disable compression
    pub fn compression(mut self, enabled: bool) -> Self {
        self.config.compression = enabled;
        self
    }

    /// Build the HTTP client
    pub fn build(self) -> Result<Client, HttpClientError> {
        let mut builder = Client::builder()
            .timeout(self.config.timeout)
            .connect_timeout(self.config.connect_timeout)
            .pool_max_idle_per_host(self.config.pool.max_idle_per_host)
            .pool_idle_timeout(self.config.pool.idle_timeout);

        if let Some(keepalive) = self.config.pool.keepalive {
            builder = builder.tcp_keepalive(keepalive);
        }

        if self.config.compression {
            builder = builder.gzip(true).brotli(true);
        }

        if self.config.http2_prior_knowledge {
            builder = builder.http2_prior_knowledge();
        }

        builder.build().map_err(HttpClientError::from)
    }
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_client_builder() {
        let client = HttpClientBuilder::new().build().unwrap();
        // Basic smoke test - if it builds, the configuration is valid
        assert!(client.get("https://httpbin.org/get").build().is_ok());
    }

    #[test]
    fn test_production_client_builder() {
        let client = HttpClientBuilder::production().build().unwrap();
        assert!(client.get("https://httpbin.org/get").build().is_ok());
    }

    #[test]
    fn test_custom_timeout() {
        let client = HttpClientBuilder::new()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap();
        assert!(client.get("https://httpbin.org/get").build().is_ok());
    }
}