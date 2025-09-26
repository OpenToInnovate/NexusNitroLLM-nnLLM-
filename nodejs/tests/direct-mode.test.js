/**
 * Direct Mode Tests for LightLLM Rust Node.js bindings.
 * 
 * Tests the new direct integration mode that bypasses HTTP entirely
 * for maximum performance and proper functionality.
 */

const { 
    NodeLightLLMClient, 
    NodeConfig, 
    NodeMessage,
    create_direct_client,
    create_http_client
} = require('../index');

describe('Direct Mode Tests', () => {
    let directClient;
    let httpClient;

    beforeAll(() => {
        // Create direct mode client
        directClient = create_direct_client('test-model', null);
        
        // Create HTTP mode client for comparison
        httpClient = create_http_client('http://localhost:8000', 'test-model', null);
    });

    describe('Client Creation', () => {
        test('should create direct client successfully', () => {
            expect(directClient).toBeDefined();
            expect(directClient).toBeInstanceOf(NodeLightLLMClient);
        });

        test('should create HTTP client successfully', () => {
            expect(httpClient).toBeDefined();
            expect(httpClient).toBeInstanceOf(NodeLightLLMClient);
        });

        test('should create direct client with custom config', () => {
            const config = new NodeConfig();
            config.lightllm_url = null; // Direct mode
            config.model_id = 'custom-model';
            config.token = 'test-token';
            
            const client = new NodeLightLLMClient(config);
            expect(client).toBeDefined();
        });
    });

    describe('Configuration and Stats', () => {
        test('should show direct mode in stats', () => {
            const stats = directClient.get_stats();
            
            expect(stats).toBeDefined();
            expect(stats.adapter_type).toBe('direct');
            expect(stats.is_direct_mode).toBe(true);
            expect(stats.performance_mode).toBe('Maximum (Direct Mode)');
            expect(stats.backend_url).toBe('direct');
        });

        test('should show HTTP mode in stats', () => {
            const stats = httpClient.get_stats();
            
            expect(stats).toBeDefined();
            expect(stats.adapter_type).toBe('lightllm');
            expect(stats.is_direct_mode).toBe(false);
            expect(stats.performance_mode).toBe('High (HTTP Mode)');
            expect(stats.backend_url).toBe('http://localhost:8000');
        });

        test('should have correct configuration values', () => {
            const stats = directClient.get_stats();
            
            expect(stats.model_id).toBe('test-model');
            expect(stats.connection_pooling).toBe(true);
            expect(stats.max_connections).toBeGreaterThan(0);
            expect(stats.max_connections_per_host).toBeGreaterThan(0);
            expect(stats.timeout_seconds).toBeGreaterThan(0);
        });
    });

    describe('Chat Completions', () => {
        test('should handle direct mode chat completions', async () => {
            const messages = [new NodeMessage('user', 'Hello, this is a test message.')];
            
            const response = await directClient.chat_completions({
                messages,
                max_tokens: 50,
                temperature: 0.7
            });
            
            expect(response).toBeDefined();
            expect(response.id).toBeDefined();
            expect(response.object).toBe('chat.completion');
            expect(response.choices).toBeDefined();
            expect(response.choices.length).toBeGreaterThan(0);
            expect(response.choices[0].message).toBeDefined();
            expect(response.choices[0].message.role).toBe('assistant');
            expect(response.choices[0].message.content).toBeDefined();
            expect(response.usage).toBeDefined();
        });

        test('should handle HTTP mode chat completions', async () => {
            const messages = [new NodeMessage('user', 'Hello, this is a test message.')];
            
            const response = await httpClient.chat_completions({
                messages,
                max_tokens: 50,
                temperature: 0.7
            });
            
            expect(response).toBeDefined();
            expect(response.id).toBeDefined();
            expect(response.object).toBe('chat.completion');
            expect(response.choices).toBeDefined();
            expect(response.choices.length).toBeGreaterThan(0);
        });

        test('should handle multiple messages in direct mode', async () => {
            const messages = [
                new NodeMessage('system', 'You are a helpful assistant.'),
                new NodeMessage('user', 'What is machine learning?'),
                new NodeMessage('assistant', 'Machine learning is a subset of AI.'),
                new NodeMessage('user', 'Can you explain more?')
            ];
            
            const response = await directClient.chat_completions({
                messages,
                max_tokens: 100,
                temperature: 0.5
            });
            
            expect(response).toBeDefined();
            expect(response.choices).toBeDefined();
            expect(response.choices.length).toBeGreaterThan(0);
        });

        test('should handle different temperature values', async () => {
            const messages = [new NodeMessage('user', 'Tell me a short story.')];
            
            const response = await directClient.chat_completions({
                messages,
                max_tokens: 50,
                temperature: 0.9 // High creativity
            });
            
            expect(response).toBeDefined();
            expect(response.choices).toBeDefined();
        });
    });

    describe('Error Handling', () => {
        test('should handle empty messages array', async () => {
            await expect(directClient.chat_completions({
                messages: [],
                max_tokens: 50
            })).rejects.toThrow();
        });

        test('should handle invalid temperature values', async () => {
            const messages = [new NodeMessage('user', 'Test message.')];
            
            await expect(directClient.chat_completions({
                messages,
                max_tokens: 50,
                temperature: -1.0 // Invalid temperature
            })).rejects.toThrow();
        });

        test('should handle very large max_tokens', async () => {
            const messages = [new NodeMessage('user', 'Test message.')];
            
            const response = await directClient.chat_completions({
                messages,
                max_tokens: 10000, // Very large value
                temperature: 0.7
            });
            
            expect(response).toBeDefined();
        });
    });

    describe('Performance Tests', () => {
        test('should handle concurrent requests in direct mode', async () => {
            const prompts = [
                'What is artificial intelligence?',
                'Explain machine learning.',
                'Define neural networks.',
                'What is deep learning?',
                'Explain natural language processing.'
            ];
            
            const startTime = Date.now();
            const promises = prompts.map(prompt => 
                directClient.chat_completions({
                    messages: [new NodeMessage('user', prompt)],
                    max_tokens: 30,
                    temperature: 0.7
                })
            );
            
            const responses = await Promise.all(prompts);
            const elapsed = Date.now() - startTime;
            
            expect(responses).toHaveLength(prompts.length);
            expect(elapsed).toBeLessThan(10000); // Should complete within 10 seconds
            
            // All responses should be valid
            responses.forEach(response => {
                expect(response).toBeDefined();
                expect(response.choices).toBeDefined();
                expect(response.choices.length).toBeGreaterThan(0);
            });
        });

        test('should handle rapid sequential requests', async () => {
            const messages = [new NodeMessage('user', 'Quick test message.')];
            
            const startTime = Date.now();
            const promises = Array(10).fill().map(() => 
                directClient.chat_completions({
                    messages,
                    max_tokens: 20,
                    temperature: 0.7
                })
            );
            
            const responses = await Promise.all(prompts);
            const elapsed = Date.now() - startTime;
            
            expect(responses).toHaveLength(10);
            expect(elapsed).toBeLessThan(5000); // Should complete within 5 seconds
        });
    });

    describe('Mode Comparison', () => {
        test('should have different adapter types', () => {
            const directStats = directClient.get_stats();
            const httpStats = httpClient.get_stats();
            
            expect(directStats.adapter_type).toBe('direct');
            expect(httpStats.adapter_type).toBe('lightllm');
        });

        test('should have different performance modes', () => {
            const directStats = directClient.get_stats();
            const httpStats = httpClient.get_stats();
            
            expect(directStats.performance_mode).toBe('Maximum (Direct Mode)');
            expect(httpStats.performance_mode).toBe('High (HTTP Mode)');
        });

        test('should have different backend URLs', () => {
            const directStats = directClient.get_stats();
            const httpStats = httpClient.get_stats();
            
            expect(directStats.backend_url).toBe('direct');
            expect(httpStats.backend_url).toBe('http://localhost:8000');
        });
    });

    describe('Convenience Functions', () => {
        test('create_direct_client should work with minimal parameters', () => {
            const client = create_direct_client();
            expect(client).toBeDefined();
            expect(client).toBeInstanceOf(NodeLightLLMClient);
            
            const stats = client.get_stats();
            expect(stats.is_direct_mode).toBe(true);
        });

        test('create_direct_client should work with custom model', () => {
            const client = create_direct_client('custom-model', 'test-token');
            expect(client).toBeDefined();
            
            const stats = client.get_stats();
            expect(stats.model_id).toBe('custom-model');
            expect(stats.is_direct_mode).toBe(true);
        });

        test('create_http_client should work with URL', () => {
            const client = create_http_client('http://localhost:8000');
            expect(client).toBeDefined();
            expect(client).toBeInstanceOf(NodeLightLLMClient);
            
            const stats = client.get_stats();
            expect(stats.is_direct_mode).toBe(false);
            expect(stats.backend_url).toBe('http://localhost:8000');
        });
    });

    describe('Memory and Resource Management', () => {
        test('should handle multiple client instances', () => {
            const clients = Array(5).fill().map(() => create_direct_client('test-model'));
            
            clients.forEach(client => {
                expect(client).toBeDefined();
                const stats = client.get_stats();
                expect(stats.is_direct_mode).toBe(true);
            });
        });

        test('should maintain consistent stats across calls', () => {
            const stats1 = directClient.get_stats();
            const stats2 = directClient.get_stats();
            
            expect(stats1.adapter_type).toBe(stats2.adapter_type);
            expect(stats1.is_direct_mode).toBe(stats2.is_direct_mode);
            expect(stats1.performance_mode).toBe(stats2.performance_mode);
        });
    });
});
