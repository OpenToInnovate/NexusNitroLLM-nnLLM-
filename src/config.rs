#[cfg(feature = "cli")]
use clap::Parser;
use std::env;
use url::Url;

/// # NexusNitroLLM Configuration
/// 
/// Comprehensive configuration system supporting command-line arguments,
/// environment variables, and .env file loading for secure configuration management.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "cli", derive(Parser))]
#[cfg_attr(feature = "cli", command(name = "nexus-nitro-llm"))]
#[cfg_attr(feature = "cli", command(about = "A universal Rust HTTP proxy that adapts OpenAI's chat completions API to work with multiple LLM backends"))]
#[cfg_attr(feature = "cli", command(version))]
pub struct Config {
    // =============================================================================
    // CORE SERVER CONFIGURATION
    // =============================================================================
    
    /// Server port to listen on
    #[cfg_attr(feature = "cli", arg(short, long, env = "PORT", default_value = "8080"))]
    pub port: u16,

    /// Server host to bind to
    #[cfg_attr(feature = "cli", arg(long, env = "HOST", default_value = "0.0.0.0"))]
    pub host: String,

    // =============================================================================
    // LLM BACKEND CONFIGURATION
    // =============================================================================
    
    /// LLM backend URL (supports LightLLM, vLLM, OpenAI, Azure, AWS, etc.)
    #[cfg_attr(feature = "cli", arg(long, env = "nnLLM_URL", default_value = "http://localhost:8000"))]
    pub backend_url: String,

    /// LLM backend type (lightllm, vllm, openai, azure, aws, etc.)
    #[cfg_attr(feature = "cli", arg(long, env = "nnLLM_BACKEND_TYPE", default_value = "lightllm"))]
    pub backend_type: String,

    /// Default model ID to use (set to "auto" for automatic detection)
    #[cfg_attr(feature = "cli", arg(long, env = "nnLLM_MODEL", default_value = "llama"))]
    pub model_id: String,

    /// Authentication token for LLM backend (supports all providers)
    #[cfg_attr(feature = "cli", arg(long, env = "nnLLM_TOKEN"))]
    pub backend_token: Option<String>,

    // =============================================================================
    // UI CONFIGURATION
    // =============================================================================
    
    /// Username for admin UI (if enabled)
    #[cfg_attr(feature = "cli", arg(long, env = "UI_USERNAME"))]
    pub ui_username: Option<String>,

    /// Password for admin UI (if enabled)
    #[cfg_attr(feature = "cli", arg(long, env = "UI_PASSWORD"))]
    pub ui_password: Option<String>,

    // =============================================================================
    // LITELLM PROXY CONFIGURATION
    // =============================================================================
    
    /// LiteLLM proxy base URL (for virtual key generation)
    #[cfg_attr(feature = "cli", arg(long, env = "LITELLM_BASE_URL"))]
    pub litellm_base_url: Option<String>,

    /// LiteLLM admin token (for virtual key generation)
    #[cfg_attr(feature = "cli", arg(long, env = "LITELLM_ADMIN_TOKEN"))]
    pub litellm_admin_token: Option<String>,

    /// LiteLLM virtual key (pre-generated)
    #[cfg_attr(feature = "cli", arg(long, env = "LITELLM_VIRTUAL_KEY"))]
    pub litellm_virtual_key: Option<String>,

    // =============================================================================
    // PERFORMANCE AND OPTIMIZATION
    // =============================================================================
    
    /// HTTP client timeout in seconds
    #[cfg_attr(feature = "cli", arg(long, env = "HTTP_CLIENT_TIMEOUT", default_value = "30"))]
    pub http_client_timeout: u64,

    /// Maximum number of HTTP connections
    #[cfg_attr(feature = "cli", arg(long, env = "HTTP_CLIENT_MAX_CONNECTIONS", default_value = "100"))]
    pub http_client_max_connections: usize,

    /// Maximum connections per host
    #[cfg_attr(feature = "cli", arg(long, env = "HTTP_CLIENT_MAX_CONNECTIONS_PER_HOST", default_value = "10"))]
    pub http_client_max_connections_per_host: usize,

    /// Streaming chunk size in bytes
    #[cfg_attr(feature = "cli", arg(long, env = "STREAMING_CHUNK_SIZE", default_value = "1024"))]
    pub streaming_chunk_size: usize,

    /// Streaming timeout in seconds
    #[cfg_attr(feature = "cli", arg(long, env = "STREAMING_TIMEOUT", default_value = "300"))]
    pub streaming_timeout: u64,

    /// Streaming keep-alive interval in seconds
    #[cfg_attr(feature = "cli", arg(long, env = "STREAMING_KEEP_ALIVE_INTERVAL", default_value = "30"))]
    pub streaming_keep_alive_interval: u64,

    // =============================================================================
    // FEATURE FLAGS
    // =============================================================================
    
    /// Enable streaming support
    #[cfg_attr(feature = "cli", arg(long, env = "ENABLE_STREAMING", default_value = "true"))]
    pub enable_streaming: bool,

    /// Enable request batching
    #[cfg_attr(feature = "cli", arg(long, env = "ENABLE_BATCHING", default_value = "false"))]
    pub enable_batching: bool,

    /// Enable rate limiting
    #[cfg_attr(feature = "cli", arg(long, env = "ENABLE_RATE_LIMITING", default_value = "true"))]
    pub enable_rate_limiting: bool,

    /// Enable response caching
    #[cfg_attr(feature = "cli", arg(long, env = "ENABLE_CACHING", default_value = "false"))]
    pub enable_caching: bool,

    /// Enable metrics collection
    #[cfg_attr(feature = "cli", arg(long, env = "ENABLE_METRICS", default_value = "true"))]
    pub enable_metrics: bool,

    /// Enable health checks
    #[cfg_attr(feature = "cli", arg(long, env = "ENABLE_HEALTH_CHECKS", default_value = "true"))]
    pub enable_health_checks: bool,

    /// Force specific adapter (auto, lightllm, openai)
    #[cfg_attr(feature = "cli", arg(long, env = "FORCE_ADAPTER", default_value = "auto"))]
    pub force_adapter: String,

    // =============================================================================
    // LOGGING AND MONITORING
    // =============================================================================
    
    /// Log level (error, warn, info, debug, trace)
    #[cfg_attr(feature = "cli", arg(long, env = "RUST_LOG", default_value = "info"))]
    pub log_level: String,

    /// Enable backtrace on panic
    #[cfg_attr(feature = "cli", arg(long, env = "RUST_BACKTRACE"))]
    pub rust_backtrace: Option<String>,

    /// Environment (development, staging, production)
    #[cfg_attr(feature = "cli", arg(long, env = "ENVIRONMENT", default_value = "development"))]
    pub environment: String,

    // =============================================================================
    // SECURITY CONFIGURATION
    // =============================================================================
    
    /// CORS origin (use * for development only)
    #[cfg_attr(feature = "cli", arg(long, env = "CORS_ORIGIN", default_value = "*"))]
    pub cors_origin: String,

    /// CORS methods
    #[cfg_attr(feature = "cli", arg(long, env = "CORS_METHODS", default_value = "GET,POST,OPTIONS"))]
    pub cors_methods: String,

    /// CORS headers
    #[cfg_attr(feature = "cli", arg(long, env = "CORS_HEADERS", default_value = "*"))]
    pub cors_headers: String,

    /// API key validation header name
    #[cfg_attr(feature = "cli", arg(long, env = "API_KEY_HEADER", default_value = "X-API-Key"))]
    pub api_key_header: String,

    /// Enable API key validation
    #[cfg_attr(feature = "cli", arg(long, env = "API_KEY_VALIDATION_ENABLED", default_value = "false"))]
    pub api_key_validation_enabled: bool,

    // =============================================================================
    // RATE LIMITING CONFIGURATION
    // =============================================================================
    
    /// Rate limit: requests per minute
    #[cfg_attr(feature = "cli", arg(long, env = "RATE_LIMIT_REQUESTS_PER_MINUTE", default_value = "60"))]
    pub rate_limit_requests_per_minute: u32,

    /// Rate limit: burst size
    #[cfg_attr(feature = "cli", arg(long, env = "RATE_LIMIT_BURST_SIZE", default_value = "10"))]
    pub rate_limit_burst_size: u32,

    // =============================================================================
    // CACHING CONFIGURATION
    // =============================================================================
    
    /// Cache TTL in seconds
    #[cfg_attr(feature = "cli", arg(long, env = "CACHE_TTL_SECONDS", default_value = "300"))]
    pub cache_ttl_seconds: u64,

    /// Maximum cache size
    #[cfg_attr(feature = "cli", arg(long, env = "CACHE_MAX_SIZE", default_value = "1000"))]
    pub cache_max_size: usize,
}

impl Config {
    /// Parse configuration from command line arguments and environment variables.
    ///
    /// This method:
    /// 1. Loads environment variables from .env file if it exists
    /// 2. Parses command line arguments
    /// 3. Validates configuration
    /// 4. Sets up logging
    ///
    /// A validated `Config` instance ready for use.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let config = Config::parse_args();
    /// println!("Server will run on port: {}", config.port);
    /// ```
    #[cfg(feature = "cli")]
    pub fn parse_args() -> Self {
        // Load .env file if it exists (ignore errors if file doesn't exist)
        #[cfg(feature = "cli")]
        let _ = dotenv::dotenv();

        let config = Self::parse();

        // Set up logging based on configuration
        config.setup_logging();

        // Validate configuration
        if let Err(err) = config.validate() {
            eprintln!("Configuration validation failed: {}", err);
            std::process::exit(1);
        }

        config
    }

    /// Auto-detect model based on token format and URL
    /// 
    /// This method analyzes the token format and URL to suggest an appropriate
    /// default model for the detected backend type.
    /// 
    /// # Returns
    /// 
    /// A suggested model identifier based on the backend type.
    pub fn auto_detect_model(&self) -> String {
        // If model is not "auto", return as-is
        if self.model_id != "auto" {
            return self.model_id.clone();
        }

        // Auto-detect based on token format and URL
        if let Some(ref token) = self.backend_token {
            // OpenAI API key format (starts with "sk-")
            if token.starts_with("sk-") {
                if self.backend_url.contains("openai.azure.com") || self.backend_url.contains("azure.com") {
                    return "gpt-35-turbo".to_string(); // Azure OpenAI default
                } else if self.backend_url.contains("litellm") || self.backend_url.contains("proxy") {
                    return "openai/gpt-3.5-turbo".to_string(); // LiteLLM proxy format
                } else {
                    return "gpt-3.5-turbo".to_string(); // OpenAI API default
                }
            }
            
            // AWS Bedrock access key format (starts with "AKIA")
            if token.starts_with("AKIA") {
                return "anthropic.claude-3-sonnet-20240229-v1:0".to_string();
            }
            
            // Azure OpenAI API key format (32+ characters, alphanumeric)
            if token.len() >= 32 && token.chars().all(|c| c.is_alphanumeric()) {
                return "gpt-35-turbo".to_string();
            }
        }

        // Fallback based on URL patterns
        if self.backend_url.contains("openai.azure.com") || self.backend_url.contains("azure.com") {
            "gpt-35-turbo".to_string()
        } else if self.backend_url.contains("bedrock") || self.backend_url.contains("amazonaws.com") {
            "anthropic.claude-3-sonnet-20240229-v1:0".to_string()
        } else if self.backend_url.contains("/v1") || self.backend_url.contains("openai.com") {
            "gpt-3.5-turbo".to_string()
        } else if self.backend_url.contains("vllm") || self.backend_url.contains("vllm.ai") ||
                  self.backend_url.contains("lightllm") || self.backend_url.contains("localhost") {
            "llama-2-7b-chat".to_string()
        } else {
            // Default fallback
            "gpt-3.5-turbo".to_string()
        }
    }

    /// Get the effective model ID (auto-detected if needed)
    /// 
    /// This method returns the actual model ID to use, performing auto-detection
    /// if the model is set to "auto".
    /// 
    /// # Returns
    /// 
    /// The effective model identifier to use for requests.
    pub fn get_effective_model_id(&self) -> String {
        self.auto_detect_model()
    }

    /// Create a test configuration with minimal required fields.
    /// This is used for testing purposes only.
    pub fn for_test() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            backend_url: "http://localhost:8000".to_string(),
            backend_type: "lightllm".to_string(),
            model_id: "llama".to_string(),
            backend_token: None,
            ui_username: None,
            ui_password: None,
            litellm_base_url: None,
            litellm_admin_token: None,
            litellm_virtual_key: None,
            http_client_timeout: 30,
            http_client_max_connections: 100,
            http_client_max_connections_per_host: 10,
            streaming_chunk_size: 1024,
            streaming_timeout: 300,
            streaming_keep_alive_interval: 30,
            enable_streaming: true,
            enable_batching: false,
            enable_rate_limiting: true,
            enable_caching: false,
            enable_metrics: true,
            enable_health_checks: true,
            force_adapter: "auto".to_string(),
            log_level: "info".to_string(),
            rust_backtrace: None,
            environment: "development".to_string(),
            cors_origin: "*".to_string(),
            cors_methods: "GET,POST,OPTIONS".to_string(),
            cors_headers: "*".to_string(),
            api_key_header: "X-API-Key".to_string(),
            api_key_validation_enabled: false,
            rate_limit_requests_per_minute: 60,
            rate_limit_burst_size: 10,
            cache_ttl_seconds: 300,
            cache_max_size: 1000,
        }
    }

    /// Set up logging configuration based on environment variables.
    /// 
    /// This method configures the tracing subscriber with the appropriate
    /// log level and format based on the configuration.
    fn setup_logging(&self) {
        // Set RUST_BACKTRACE if specified
        if let Some(backtrace) = &self.rust_backtrace {
            env::set_var("RUST_BACKTRACE", backtrace);
        }

        // Initialize tracing subscriber with environment filter
        #[cfg(feature = "cli")]
        let _ = tracing_subscriber::fmt()
            .with_env_filter(&self.log_level)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .try_init();
    }

    /// Validate configuration values and provide helpful error messages.
    /// 
    /// This method performs comprehensive validation of all configuration
    /// parameters, ensuring they meet security, performance, and functionality
    /// requirements. Similar to configuration validation in enterprise C++
    /// applications but with compile-time safety guarantees.
    /// 
    /// # Panics
    /// 
    /// Panics if configuration is invalid with a helpful error message.
    pub fn validate(&self) -> Result<(), String> {
        // Validate port range
        if self.port == 0 {
            return Err("Port cannot be 0. Please specify a valid port number (1-65535).".to_string());
        }
        // Port validation: u16 automatically ensures port <= 65535
        
        // Warn about privileged ports in production
        if self.port < 1024 && cfg!(not(debug_assertions)) {
            eprintln!(
                "⚠️  Warning: Using privileged port {} may require root access. \
                Consider using a port >= 1024 for better security.",
                self.port
            );
        }

        // Validate host format
        if self.host.is_empty() {
            return Err("Host cannot be empty. Please specify a valid host (e.g., '0.0.0.0', 'localhost', or an IP address).".to_string());
        }
        
        // Validate host format for IP addresses
        if !self.host.eq("0.0.0.0") && !self.host.eq("localhost") && !self.host.eq("127.0.0.1") {
            // Try to parse as IP address
            if self.host.parse::<std::net::IpAddr>().is_err() {
                eprintln!(
                    "⚠️  Warning: Host '{}' is not a recognized format. \
                    Use '0.0.0.0' for all interfaces, 'localhost' for local access, or a valid IP address.",
                    self.host
                );
            }
        }

        // Validate LightLLM URL format
        if self.backend_url.is_empty() {
            return Err("LightLLM URL cannot be empty. Please specify a valid backend URL.".to_string());
        }
        
        // Validate URL format
        match Url::parse(&self.backend_url) {
            Ok(url) => {
                // Validate URL scheme
                if !["http", "https"].contains(&url.scheme()) {
                    return Err(format!(
                        "Invalid URL scheme '{}'. Only 'http' and 'https' are supported.",
                        url.scheme()
                    ));
                }
                
                // Validate URL has host
                if url.host().is_none() {
                    return Err("LightLLM URL must include a host (e.g., 'http://localhost:8000').".to_string());
                }
                
                // Warn about HTTP in production
                if self.environment == "production" && url.scheme() == "http" {
                    eprintln!(
                        "⚠️  Warning: Using HTTP in production is not recommended. \
                        Consider using HTTPS for better security."
                    );
                }
            }
            Err(err) => {
                return Err(format!(
                    "Invalid LightLLM URL format '{}': {}. \
                    Please provide a valid URL (e.g., 'http://localhost:8000').",
                    self.backend_url, err
                ));
            }
        }

        // Validate model ID
        if self.model_id.is_empty() {
            return Err("Model ID cannot be empty. Please specify a valid model identifier.".to_string());
        }
        
        // Validate model ID format (alphanumeric, hyphens, underscores)
        if !self.model_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(format!(
                "Model ID '{}' contains invalid characters. \
                Only alphanumeric characters, hyphens, and underscores are allowed.",
                self.model_id
            ));
        }

        // Validate adapter selection
        let valid_adapters = ["auto", "lightllm", "openai"];
        if !valid_adapters.contains(&self.force_adapter.as_str()) {
            return Err(format!(
                "Invalid adapter '{}'. Valid options are: {}",
                self.force_adapter,
                valid_adapters.join(", ")
            ));
        }

        // Validate environment
        let valid_environments = ["development", "staging", "production"];
        if !valid_environments.contains(&self.environment.as_str()) {
            return Err(format!(
                "Invalid environment '{}'. Valid options are: {}",
                self.environment,
                valid_environments.join(", ")
            ));
        }

        // Validate HTTP client configuration
        if self.http_client_timeout == 0 {
            return Err("HTTP client timeout must be greater than 0 seconds.".to_string());
        }
        if self.http_client_timeout > 300 {
            eprintln!(
                "⚠️  Warning: HTTP client timeout of {} seconds is very high. \
                Consider using a smaller timeout (30-60 seconds) for better responsiveness.",
                self.http_client_timeout
            );
        }
        
        if self.http_client_max_connections == 0 {
            return Err("HTTP client max connections must be greater than 0.".to_string());
        }
        if self.http_client_max_connections > 1000 {
            eprintln!(
                "⚠️  Warning: HTTP client max connections of {} is very high. \
                Consider using a smaller value (100-500) unless you have specific requirements.",
                self.http_client_max_connections
            );
        }
        
        if self.http_client_max_connections_per_host == 0 {
            return Err("HTTP client max connections per host must be greater than 0.".to_string());
        }
        if self.http_client_max_connections_per_host > self.http_client_max_connections {
            eprintln!(
                "⚠️  Warning: Max connections per host ({}) exceeds total max connections ({}). \
                This may cause unexpected behavior.",
                self.http_client_max_connections_per_host,
                self.http_client_max_connections
            );
        }

        // Validate streaming configuration
        if self.streaming_timeout == 0 {
            return Err("Streaming timeout must be greater than 0 seconds.".to_string());
        }
        if self.streaming_chunk_size == 0 {
            return Err("Streaming chunk size must be greater than 0 bytes.".to_string());
        }
        if self.streaming_chunk_size > 1024 * 1024 { // 1MB
            eprintln!(
                "⚠️  Warning: Streaming chunk size of {} bytes is very large. \
                Consider using smaller chunks (1-64KB) for better streaming performance.",
                self.streaming_chunk_size
            );
        }

        // Validate rate limiting configuration
        if self.rate_limit_requests_per_minute == 0 {
            eprintln!(
                "⚠️  Warning: Rate limit of 0 requests per minute will block all requests. \
                Consider setting a reasonable limit (e.g., 60 requests/minute)."
            );
        }
        if self.rate_limit_burst_size == 0 {
            return Err("Rate limit burst size must be greater than 0.".to_string());
        }
        if self.rate_limit_burst_size > self.rate_limit_requests_per_minute {
            eprintln!(
                "⚠️  Warning: Burst size ({}) exceeds requests per minute limit ({}). \
                This may cause unexpected rate limiting behavior.",
                self.rate_limit_burst_size,
                self.rate_limit_requests_per_minute
            );
        }

        // Validate caching configuration
        if self.cache_ttl_seconds == 0 {
            eprintln!(
                "⚠️  Warning: Cache TTL of 0 seconds will effectively disable caching. \
                Consider setting a reasonable TTL (e.g., 300-3600 seconds)."
            );
        }
        if self.cache_max_size == 0 {
            eprintln!(
                "⚠️  Warning: Cache max size of 0 will effectively disable caching. \
                Consider setting a reasonable cache size (e.g., 100-10000 entries)."
            );
        }

        // Validate CORS configuration for production
        if self.environment == "production" {
            if self.cors_origin == "*" {
                eprintln!(
                    "⚠️  Warning: Using CORS origin '*' in production is not recommended. \
                    Consider specifying specific origins for better security."
                );
            }
            
            if self.log_level == "debug" || self.log_level == "trace" {
                eprintln!(
                    "⚠️  Warning: Using debug/trace logging in production may impact performance \
                    and expose sensitive information in logs."
                );
            }
        }

        // Validate token requirements
        if self.backend_url.contains("/v1/") && self.backend_token.is_none() {
            eprintln!(
                "⚠️  Warning: Using LiteLLM proxy URL without token. \
                You may need to set nnLLM_TOKEN for authentication."
            );
        }
        
        // Validate backend_type
        let valid_backend_types = ["lightllm", "vllm", "openai", "azure", "aws", "custom", "direct"];
        if !valid_backend_types.contains(&self.backend_type.as_str()) {
            eprintln!(
                "⚠️  Warning: Unknown backend type '{}'. Valid options are: {}",
                self.backend_type,
                valid_backend_types.join(", ")
            );
        }
        
        // Validate URL format
        if self.backend_url != "direct" && !self.backend_url.starts_with("http://") && !self.backend_url.starts_with("https://") {
            eprintln!(
                "⚠️  Warning: Backend URL '{}' should start with http:// or https://, or be 'direct' for direct mode",
                self.backend_url
            );
        }
        
        // Validate log level
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!(
                "Invalid log level '{}'. Valid options are: {}",
                self.log_level,
                valid_log_levels.join(", ")
            ));
        }

        // Validate CORS configuration
        if self.cors_methods.is_empty() {
            return Err("CORS methods cannot be empty. Please specify valid HTTP methods.".to_string());
        }
        if self.cors_headers.is_empty() {
            return Err("CORS headers cannot be empty. Please specify valid header names or use '*'.".to_string());
        }

        // Performance warnings
        if self.enable_caching && self.cache_max_size > 10000 {
            eprintln!(
                "⚠️  Warning: Large cache size of {} entries may consume significant memory. \
                Monitor memory usage in production.",
                self.cache_max_size
            );
        }
        
        if self.enable_batching && !self.enable_streaming {
            eprintln!(
                "⚠️  Warning: Batching is enabled but streaming is disabled. \
                Consider enabling streaming for better performance with batching."
            );
        }

        Ok(())
    }

    /// Get the effective LightLLM token, checking multiple sources.
    /// 
    /// This method checks for tokens in the following order:
    /// 1. Explicitly provided nnLLM_TOKEN
    /// 2. LiteLLM virtual key (LITELLM_VIRTUAL_KEY)
    /// 3. None (no authentication)
    /// 
    /// The most appropriate token for authentication, or None if no token is available.
    pub fn get_effective_token(&self) -> Option<String> {
        self.backend_token
            .clone()
            .or_else(|| self.litellm_virtual_key.clone())
    }

    /// Check if this configuration is for a LiteLLM proxy backend.
    /// 
    /// LiteLLM proxy backends typically have URLs containing "/v1/" and
    /// require virtual keys (starting with "sk-") for authentication.
    /// 
    /// True if this appears to be a LiteLLM proxy configuration.
    pub fn is_litellm_proxy(&self) -> bool {
        self.backend_url.contains("/v1/") || self.backend_url.contains("openai")
    }

    /// Check if this configuration is for a raw LightLLM server.
    /// 
    /// Raw LightLLM servers typically don't have "/v1/" in their URLs and
    /// use the native LightLLM API format.
    /// 
    /// True if this appears to be a raw LightLLM server configuration.
    pub fn is_raw_lightllm(&self) -> bool {
        !self.is_litellm_proxy()
    }

}
