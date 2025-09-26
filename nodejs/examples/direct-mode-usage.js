#!/usr/bin/env node
/**
 * Direct Mode Usage Example for LightLLM Rust Node.js bindings.
 * 
 * This example demonstrates the new direct integration mode that bypasses HTTP entirely
 * for maximum performance. Perfect for Node.js applications that want direct access
 * to LightLLM without network overhead.
 * 
 * Key benefits of direct mode:
 * - Zero HTTP overhead (no network serialization/deserialization)
 * - Direct memory access between Node.js and Rust
 * - Minimal latency (direct function calls)
 * - Maximum throughput (no network bottlenecks)
 * - No need for running LightLLM server separately
 */

const { 
    NodeLightLLMClient, 
    NodeConfig, 
    NodeMessage,
    create_direct_client,
    create_http_client
} = require('../index');

const performance = require('perf_hooks').performance;

class DirectModeProcessor {
    constructor(modelId = 'llama', token = null) {
        this.modelId = modelId;
        this.token = token;
        
        // Create direct mode client (no HTTP overhead)
        this.directClient = create_direct_client(modelId, token);
        
        console.log('🚀 Direct mode processor initialized');
        console.log(`   Model: ${modelId}`);
        console.log('   Mode: Direct (no HTTP)');
        console.log(`   Token: ${token ? 'Set' : 'Not set'}`);
        console.log('   Performance: Maximum (zero network overhead)');
    }

    async singleRequest(prompt, maxTokens = 100) {
        const messages = [new NodeMessage('user', prompt)];
        
        const startTime = performance.now();
        const response = await this.directClient.chat_completions({
            messages,
            max_tokens: maxTokens,
            temperature: 0.7
        });
        const elapsed = performance.now() - startTime;
        
        console.log(`⚡ Direct request completed in ${elapsed.toFixed(1)}ms`);
        return response;
    }

    async concurrentRequests(prompts, maxTokens = 50) {
        console.log(`🔄 Processing ${prompts.length} direct requests concurrently...`);
        const startTime = performance.now();
        
        // Create promises for concurrent execution
        const promises = prompts.map(prompt => 
            this.singleRequest(prompt, maxTokens)
        );
        
        // Execute all requests concurrently
        const responses = await Promise.all(prompts);
        const elapsed = performance.now() - startTime;
        
        console.log(`✅ Direct concurrent processing completed: ${responses.length} responses in ${elapsed.toFixed(1)}ms`);
        console.log(`   Average per request: ${(elapsed / prompts.length).toFixed(1)}ms`);
        console.log(`   Throughput: ${(prompts.length / (elapsed / 1000)).toFixed(1)} requests/second`);
        
        return responses;
    }

    getPerformanceStats() {
        const stats = this.directClient.get_stats();
        console.log('\n📊 Direct Mode Performance Statistics:');
        console.log(`   Adapter type: ${stats.adapter_type}`);
        console.log(`   Backend URL: ${stats.backend_url}`);
        console.log(`   Model ID: ${stats.model_id}`);
        console.log(`   Direct mode: ${stats.is_direct_mode}`);
        console.log(`   Performance mode: ${stats.performance_mode}`);
        console.log(`   Connection pooling: ${stats.connection_pooling}`);
        console.log(`   Max connections: ${stats.max_connections}`);
        console.log(`   Timeout: ${stats.timeout_seconds}s`);
    }

    performanceComparison() {
        console.log('\n📈 Direct Mode vs HTTP Mode Performance Comparison');
        console.log('=' * 60);
        
        const prompts = [
            'What is artificial intelligence?',
            'Explain machine learning briefly.',
            'What are neural networks?',
            'Define deep learning.',
            'What is natural language processing?',
        ];
        
        console.log('Direct mode advantages:');
        console.log('  • Zero HTTP overhead');
        console.log('  • Direct memory access');
        console.log('  • No network serialization');
        console.log('  • Maximum performance');
        console.log('  • No server setup required');
    }
}

class ModeComparison {
    constructor() {
        // Create both direct and HTTP clients for comparison
        this.directClient = create_direct_client('llama', null);
        this.httpClient = create_http_client('http://localhost:8000', 'llama', null);
        
        console.log('🚀 Mode comparison initialized');
        console.log('   Direct Mode: Zero HTTP overhead');
        console.log('   HTTP Mode: Traditional proxy communication');
    }

    async testDirectMode(prompt, maxTokens = 50) {
        const messages = [new NodeMessage('user', prompt)];
        
        const startTime = performance.now();
        const response = await this.directClient.chat_completions({
            messages,
            max_tokens: maxTokens,
            temperature: 0.7
        });
        const elapsed = performance.now() - startTime;
        
        console.log(`⚡ Direct mode request completed in ${elapsed.toFixed(1)}ms`);
        return response;
    }

    async testHttpMode(prompt, maxTokens = 50) {
        const messages = [new NodeMessage('user', prompt)];
        
        const startTime = performance.now();
        const response = await this.httpClient.chat_completions({
            messages,
            max_tokens: maxTokens,
            temperature: 0.7
        });
        const elapsed = performance.now() - startTime;
        
        console.log(`🌐 HTTP mode request completed in ${elapsed.toFixed(1)}ms`);
        return response;
    }

    async performanceComparison() {
        console.log('\n📈 Performance Comparison: Direct vs HTTP Mode');
        console.log('=' * 60);
        
        const prompts = [
            'What is artificial intelligence?',
            'Explain machine learning briefly.',
            'What are neural networks?',
            'Define deep learning.',
            'What is natural language processing?',
        ];
        
        // Test direct mode
        console.log('\n⚡ Testing Direct Mode...');
        const directStartTime = performance.now();
        const directPromises = prompts.map(prompt => this.testDirectMode(prompt, 30));
        const directResponses = await Promise.all(directPromises);
        const directElapsed = performance.now() - directStartTime;
        
        console.log(`Direct mode: ${directResponses.length} responses in ${directElapsed.toFixed(1)}ms`);
        console.log(`Direct throughput: ${(prompts.length / (directElapsed / 1000)).toFixed(1)} requests/second`);
        
        // Test HTTP mode
        console.log('\n🌐 Testing HTTP Mode...');
        const httpStartTime = performance.now();
        const httpPromises = prompts.map(prompt => this.testHttpMode(prompt, 30));
        const httpResponses = await Promise.all(httpPromises);
        const httpElapsed = performance.now() - httpStartTime;
        
        console.log(`HTTP mode: ${httpResponses.length} responses in ${httpElapsed.toFixed(1)}ms`);
        console.log(`HTTP throughput: ${(prompts.length / (httpElapsed / 1000)).toFixed(1)} requests/second`);
        
        // Calculate speedup
        if (directElapsed < httpElapsed) {
            const speedup = httpElapsed / directElapsed;
            console.log(`\n📊 Direct Mode Speedup: ${speedup.toFixed(1)}x faster`);
        } else {
            console.log('\n📊 Both modes performed similarly');
        }
    }

    getModeStatistics() {
        console.log('\n📊 Mode Statistics:');
        console.log('-'.repeat(30));
        
        // Direct Mode stats
        const directStats = this.directClient.get_stats();
        console.log('⚡ Direct Mode:');
        console.log(`   Adapter type: ${directStats.adapter_type}`);
        console.log(`   Backend URL: ${directStats.backend_url}`);
        console.log(`   Direct mode: ${directStats.is_direct_mode}`);
        console.log(`   Performance mode: ${directStats.performance_mode}`);
        
        // HTTP Mode stats
        const httpStats = this.httpClient.get_stats();
        console.log('🌐 HTTP Mode:');
        console.log(`   Adapter type: ${httpStats.adapter_type}`);
        console.log(`   Backend URL: ${httpStats.backend_url}`);
        console.log(`   Direct mode: ${httpStats.is_direct_mode}`);
        console.log(`   Performance mode: ${httpStats.performance_mode}`);
    }

    usageRecommendations() {
        console.log('\n💡 Usage Recommendations:');
        console.log('=' * 40);
        
        console.log('🌐 Use HTTP Mode when:');
        console.log('  • You have a running LightLLM server');
        console.log('  • You need to share the same backend across multiple applications');
        console.log('  • You want to use existing LightLLM infrastructure');
        console.log('  • You need to scale horizontally across multiple servers');
        console.log('  • You want to use LightLLM\'s built-in features (batching, routing, etc.)');
        
        console.log('\n⚡ Use Direct Mode when:');
        console.log('  • You want maximum performance with minimal latency');
        console.log('  • You\'re building a Node.js application that needs direct integration');
        console.log('  • You don\'t want to run a separate LightLLM server');
        console.log('  • You\'re building embedded applications');
        console.log('  • You need zero network overhead');
        console.log('  • You\'re doing high-performance computing or real-time applications');
    }
}

async function main() {
    console.log('🚀 LightLLM Rust Node.js Bindings - Direct Mode Usage');
    console.log('='.repeat(70));
    
    try {
        // Initialize direct mode processor
        const processor = new DirectModeProcessor('llama', null);
        
        // Test single direct request
        console.log('\n1. 🎯 Single Direct Request Test');
        console.log('-'.repeat(40));
        
        const response = await processor.singleRequest(
            'Explain the benefits of direct integration in one sentence.',
            50
        );
        console.log('✅ Single direct request successful');
        if (response.choices && response.choices.length > 0) {
            console.log(`   Response: ${response.choices[0].message.content}`);
        }
        
        // Test concurrent direct requests
        console.log('\n2. 🔄 Concurrent Direct Requests Test');
        console.log('-'.repeat(40));
        
        const prompts = [
            'What is machine learning?',
            'Explain neural networks.',
            'Define artificial intelligence.',
            'What is deep learning?',
            'Explain natural language processing.',
        ];
        
        const concurrentResponses = await processor.concurrentRequests(prompts, 30);
        console.log(`✅ Concurrent direct processing successful: ${concurrentResponses.length} responses`);
        
        // Performance comparison
        processor.performanceComparison();
        
        // Get performance stats
        processor.getPerformanceStats();
        
        // Mode comparison
        console.log('\n3. 📊 Mode Comparison');
        console.log('-'.repeat(40));
        
        const comparison = new ModeComparison();
        await comparison.performanceComparison();
        comparison.getModeStatistics();
        comparison.usageRecommendations();
        
        console.log('\n✨ Direct mode demonstration complete!');
        console.log('\n🎯 Direct Mode Benefits:');
        console.log('  • Zero HTTP overhead (no network serialization)');
        console.log('  • Direct memory access between Node.js and Rust');
        console.log('  • Minimal latency (direct function calls)');
        console.log('  • Maximum throughput (no network bottlenecks)');
        console.log('  • No need for separate LightLLM server');
        console.log('  • Perfect for embedded applications');
        console.log('  • Ideal for high-performance computing');
        
    } catch (error) {
        console.error('❌ Error:', error.message);
        console.error('💡 Make sure the LightLLM Rust bindings are properly installed');
    }
}

// Run the demonstration
if (require.main === module) {
    main().catch(console.error);
}

module.exports = {
    DirectModeProcessor,
    ModeComparison
};
