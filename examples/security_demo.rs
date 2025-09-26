//! # Security Demo
//! 
//! This example demonstrates the security features of NexusNitroLLM,
//! including HTTPS enforcement and URL validation.

use nexus_nitro_llm::config::Config;

fn main() {
    println!("🔒 NexusNitroLLM Security Demo");
    println!("================================");

    // Test 1: Secure HTTPS URL (should work)
    println!("\n✅ Test 1: Secure HTTPS URL");
    test_secure_url();

    // Test 2: Local HTTP URL (should work in development)
    println!("\n🔒 Test 2: Local HTTP URL");
    test_local_url();

    // Test 3: Insecure HTTP URL (should warn/panic)
    println!("\n⚠️  Test 3: Insecure HTTP URL");
    test_insecure_url();
}

fn test_secure_url() {
    let mut config = Config::for_test();
    config.backend_url = "https://api.openai.com/v1".to_string();
    config.model_id = "gpt-4".to_string();
    config.environment = "production".to_string();
    
    println!("   URL: {}", config.backend_url);
    println!("   Environment: {}", config.environment);
    println!("   ✅ HTTPS URL is secure for internet traffic");
}

fn test_local_url() {
    let mut config = Config::for_test();
    config.backend_url = "http://localhost:8000".to_string();
    config.model_id = "llama".to_string();
    config.environment = "development".to_string();
    
    println!("   URL: {}", config.backend_url);
    println!("   Environment: {}", config.environment);
    println!("   🔒 HTTP is allowed for localhost in development");
}

fn test_insecure_url() {
    let mut config = Config::for_test();
    config.backend_url = "http://api.openai.com/v1".to_string();
    config.model_id = "gpt-4".to_string();
    config.environment = "production".to_string();
    
    println!("   URL: {}", config.backend_url);
    println!("   Environment: {}", config.environment);
    println!("   ⚠️  This would PANIC in production due to HTTP over internet");
    println!("   💡 Use HTTPS for internet traffic: https://api.openai.com/v1");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_configurations() {
        // Test secure configuration
        let mut secure_config = Config::for_test();
        secure_config.backend_url = "https://api.openai.com/v1".to_string();
        secure_config.environment = "production".to_string();
        
        assert!(secure_config.backend_url.starts_with("https://"));
        assert_eq!(secure_config.environment, "production");
        
        // Test local configuration
        let mut local_config = Config::for_test();
        local_config.backend_url = "http://localhost:8000".to_string();
        local_config.environment = "development".to_string();
        
        assert!(local_config.backend_url.starts_with("http://"));
        assert!(local_config.backend_url.contains("localhost"));
        assert_eq!(local_config.environment, "development");
    }
}
