#!/usr/bin/env node

/**
 * Basic usage example for NexusNitroLLM Node.js bindings
 *
 * Demonstrates the core functionality and high-performance features
 * of the Node.js bindings with zero HTTP overhead.
 */

const {
    NodeNexusNitroLLMClient,
    createConfig,
    createMessage,
    createClient,
    getVersion
} = require('../../index.js');

console.log('🚀 NexusNitroLLM Node.js Bindings - Basic Usage Example');
console.log('═══════════════════════════════════════════════════════');
console.log(`Library Version: ${getVersion()}`);
console.log(`Node.js Version: ${process.version}`);
console.log();

async function basicUsageDemo() {
    try {
        // 1. Configuration Creation
        console.log('📝 Step 1: Creating high-performance configuration...');

        const config = createConfig(
            'http://localhost:8000',  // LLM backend URL
            'llama',                  // Default model
            {
                port: 3000,
                connection_pooling: true,     // Enable for maximum performance
                max_connections: 100,         // High connection limit
                max_connections_per_host: 20, // Per-host connection limit
                token: 'your-auth-token'      // Optional authentication
            }
        );

        console.log(`   ✅ Configuration created:`);
        console.log(`      Backend URL: ${config.backend_url}`);
        console.log(`      Model: ${config.model_id}`);
        console.log(`      Connection Pooling: ${config.connection_pooling}`);
        console.log(`      Max Connections: ${config.max_connections}`);
        console.log();

        // 2. Client Creation
        console.log('🔧 Step 2: Creating high-performance client...');

        // Method 1: Using configuration object
        const client1 = new NodeNexusNitroLLMClient(config);

        // Method 2: Using convenience function
        const client2 = createClient('http://localhost:8000', 'llama');

        console.log('   ✅ Clients created successfully');
        console.log();

        // 3. Performance Statistics
        console.log('📊 Step 3: Getting performance statistics...');

        const stats = client1.getStats();
        console.log(`   ✅ Performance stats:`);
        console.log(`      Adapter Type: ${stats.adapter_type}`);
        console.log(`      Runtime: ${stats.runtime_type}`);
        console.log(`      Connection Pooling: ${stats.connection_pooling}`);
        console.log(`      Max Connections: ${stats.max_connections}`);
        console.log(`      Max Connections per Host: ${stats.max_connections_per_host}`);
        console.log();

        // 4. Message Creation
        console.log('📝 Step 4: Creating optimized messages...');

        const systemMessage = createMessage(
            'system',
            'You are a helpful AI assistant with expertise in programming and technology.'
        );

        const userMessage = createMessage(
            'user',
            'Explain the benefits of zero-copy data transfer in high-performance applications.'
        );

        const namedMessage = createMessage(
            'assistant',
            'Zero-copy data transfer eliminates unnecessary memory copies...',
            'technical-assistant'
        );

        console.log('   ✅ Messages created:');
        console.log(`      System: "${systemMessage.content.substring(0, 50)}..."`);
        console.log(`      User: "${userMessage.content.substring(0, 50)}..."`);
        console.log(`      Named Assistant: "${namedMessage.content.substring(0, 30)}..." (${namedMessage.name})`);
        console.log();

        // 5. Connection Testing
        console.log('🔌 Step 5: Testing backend connection...');

        const isConnected = await client1.testConnection();
        console.log(`   Connection Status: ${isConnected ? '✅ Connected' : '❌ Backend unreachable'}`);

        if (!isConnected) {
            console.log('   ℹ️  This is expected if LightLLM backend is not running');
        }
        console.log();

        // 6. Chat Completion Request
        console.log('💬 Step 6: Sending chat completion request...');

        const messages = [systemMessage, userMessage];

        const chatRequest = {
            messages: messages,
            max_tokens: 150,
            temperature: 0.7,
            top_p: 0.9,
            stream: false,
            user: 'demo-user'
        };

        try {
            console.log('   📤 Sending request with direct Rust function call...');
            const response = await client1.chatCompletions(chatRequest);

            console.log('   ✅ Response received:');
            console.log(`      ID: ${response.id}`);
            console.log(`      Model: ${response.model}`);
            console.log(`      Choices: ${response.choices.length}`);
            console.log(`      Usage: ${response.usage.total_tokens} tokens`);

            if (response.choices.length > 0) {
                const choice = response.choices[0];
                console.log(`      Response: "${choice.message.content}"`);
                console.log(`      Finish Reason: ${choice.finish_reason}`);
            }

        } catch (error) {
            console.log('   ❌ Expected error (backend not reachable):');
            console.log(`      ${error.message}`);
            console.log('   ℹ️  This demonstrates graceful error handling');
        }
        console.log();

        // 7. Dynamic Configuration Updates
        console.log('⚙️  Step 7: Dynamic configuration updates...');

        const newConfig = createConfig(
            'http://updated-backend:8001',
            'updated-model',
            {
                connection_pooling: false,
                max_connections: 50
            }
        );

        client1.updateConfig(newConfig);
        console.log('   ✅ Configuration updated dynamically');

        const updatedStats = client1.getStats();
        console.log(`      New Backend: ${newConfig.backend_url}`);
        console.log(`      New Model: ${newConfig.model_id}`);
        console.log(`      Updated Connection Pooling: ${updatedStats.connection_pooling}`);
        console.log();

        // 8. Performance Demonstration
        console.log('🏎️  Step 8: Performance demonstration...');

        const performanceStart = process.hrtime.bigint();
        const operationCount = 1000;

        console.log(`   Performing ${operationCount} rapid operations...`);

        for (let i = 0; i < operationCount; i++) {
            // Mix of high-performance operations
            const tempConfig = createConfig(`http://perf${i}.local:8000`, `model${i}`);
            const tempMessage = createMessage('user', `Performance test ${i}`);
            const tempStats = client2.getStats();

            // Verify operations work correctly
            if (i === 0) {
                console.log(`      First operation results: Config URL contains "perf0": ${tempConfig.backend_url.includes('perf0')}`);
                console.log(`      Message content: "${tempMessage.content}"`);
                console.log(`      Stats adapter: "${tempStats.adapter_type}"`);
            }
        }

        const performanceEnd = process.hrtime.bigint();
        const duration = Number(performanceEnd - performanceStart) / 1000000; // Convert to ms
        const operationsPerSecond = (operationCount / duration) * 1000;

        console.log(`   ✅ Performance results:`);
        console.log(`      Operations: ${operationCount.toLocaleString()}`);
        console.log(`      Duration: ${duration.toFixed(2)}ms`);
        console.log(`      Rate: ${operationsPerSecond.toFixed(0)} operations/second`);
        console.log(`      🚀 Demonstrating zero-overhead Rust performance!`);
        console.log();

        // 9. Memory Usage Analysis
        console.log('🧠 Step 9: Memory usage analysis...');

        const memoryUsage = process.memoryUsage();
        console.log('   📈 Current memory usage:');
        console.log(`      RSS: ${(memoryUsage.rss / 1024 / 1024).toFixed(2)} MB`);
        console.log(`      Heap Used: ${(memoryUsage.heapUsed / 1024 / 1024).toFixed(2)} MB`);
        console.log(`      Heap Total: ${(memoryUsage.heapTotal / 1024 / 1024).toFixed(2)} MB`);
        console.log(`      External: ${(memoryUsage.external / 1024 / 1024).toFixed(2)} MB`);
        console.log();

        // 10. Advanced Features Preview
        console.log('🎯 Step 10: Advanced features preview...');

        console.log('   Available advanced features:');
        console.log('   • Zero-copy data transfer between Node.js and Rust');
        console.log('   • Native async/await with proper event loop integration');
        console.log('   • Connection pooling for maximum backend throughput');
        console.log('   • Dynamic configuration updates without restart');
        console.log('   • Built-in performance monitoring and statistics');
        console.log('   • Thread-safe operations for concurrent usage');
        console.log('   • Automatic memory management via Rust ownership');
        console.log('   • TypeScript definitions (auto-generated from Rust)');
        console.log();

        console.log('🎉 Basic usage demonstration completed successfully!');
        console.log();
        console.log('Next Steps:');
        console.log('• Start an LLM backend server to see real responses');
        console.log('• Run performance benchmarks: npm run bench');
        console.log('• Explore stress tests: npm test stress');
        console.log('• Check out advanced examples in the examples/ directory');

    } catch (error) {
        console.error('❌ Demo failed:', error.message);
        console.error(error.stack);
        process.exit(1);
    }
}

// Helper function for pretty printing objects
function prettyPrint(label, obj) {
    console.log(`${label}:`);
    console.log(JSON.stringify(obj, null, 2));
    console.log();
}

// Run the demo if this file is executed directly
if (require.main === module) {
    basicUsageDemo();
}