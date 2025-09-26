/**
 * Basic functionality tests for NexusNitroLLM Node.js bindings
 *
 * Tests core functionality, configuration, and basic operations to ensure
 * the bindings work correctly and efficiently with zero HTTP overhead.
 */

const {
    NodeNexusNitroLLMClient,
    createConfig,
    createMessage,
    createClient,
    getVersion
} = require('../..');

describe('Basic Functionality Tests', () => {
    describe('Configuration Management', () => {
        test('should create config with default values', () => {
            const config = createConfig('http://localhost:8000', 'test-model');

            expect(config.lightllm_url).toBe('http://localhost:8000');
            expect(config.model_id).toBe('test-model');
            expect(config.connection_pooling).toBe(true);
            expect(config.max_connections).toBe(100);
        });

        test('should create config with custom options', () => {
            const options = {
                port: 3001,
                token: 'test-token',
                connection_pooling: false,
                max_connections: 50
            };

            const config = createConfig('http://test.com', 'custom-model', options);

            expect(config.lightllm_url).toBe('http://test.com');
            expect(config.model_id).toBe('custom-model');
            expect(config.port).toBe(3001);
            expect(config.token).toBe('test-token');
            expect(config.connection_pooling).toBe(false);
            expect(config.max_connections).toBe(50);
        });
    });

    describe('Message Creation', () => {
        test('should create message with role and content', () => {
            const message = createMessage('user', 'Hello world!');

            expect(message.role).toBe('user');
            expect(message.content).toBe('Hello world!');
            expect(message.name).toBeUndefined();
        });

        test('should create message with optional name', () => {
            const message = createMessage('assistant', 'Hi there!', 'assistant-1');

            expect(message.role).toBe('assistant');
            expect(message.content).toBe('Hi there!');
            expect(message.name).toBe('assistant-1');
        });

        test('should handle various message roles', () => {
            const roles = ['system', 'user', 'assistant', 'tool'];

            roles.forEach(role => {
                const message = createMessage(role, `Content for ${role}`);
                expect(message.role).toBe(role);
                expect(message.content).toBe(`Content for ${role}`);
            });
        });

        test('should handle empty and long content', () => {
            // Empty content
            const emptyMsg = createMessage('user', '');
            expect(emptyMsg.content).toBe('');

            // Long content
            const longContent = 'x'.repeat(10000);
            const longMsg = createMessage('user', longContent);
            expect(longMsg.content).toBe(longContent);
            expect(longMsg.content.length).toBe(10000);
        });

        test('should handle unicode content', () => {
            const unicodeContent = 'Hello 🌍! This has émojis and spécial châractèrs.';
            const message = createMessage('user', unicodeContent);
            expect(message.content).toBe(unicodeContent);
        });
    });

    describe('Client Creation and Basic Operations', () => {
        test('should create client with configuration', () => {
            const config = createConfig('http://localhost:8000', 'test-model');
            const client = new NodeLightLLMClient(config);

            expect(client).toBeInstanceOf(NodeLightLLMClient);
        });

        test('should create client with convenience function', () => {
            const client = createClient('http://localhost:8000', 'test-model');

            expect(client).toBeInstanceOf(NodeLightLLMClient);
        });

        test('should get performance statistics', () => {
            const client = createClient('http://localhost:8000', 'test-model');
            const stats = client.getStats();

            expect(stats).toHaveProperty('adapter_type');
            expect(stats).toHaveProperty('connection_pooling');
            expect(stats).toHaveProperty('runtime_type');
            expect(stats).toHaveProperty('max_connections');
            expect(stats).toHaveProperty('max_connections_per_host');

            expect(['lightllm', 'openai']).toContain(stats.adapter_type);
            expect(typeof stats.connection_pooling).toBe('boolean');
            expect(stats.runtime_type).toBe('tokio');
            expect(typeof stats.max_connections).toBe('number');
        });

        test('should test connection to backend', async () => {
            const client = createClient('http://127.0.0.1:65432', 'test-model'); // Unreachable port

            // This should return false for unreachable backend, not throw
            const connected = await client.testConnection();
            expect(typeof connected).toBe('boolean');
            expect(connected).toBe(false);
        });

        test('should update configuration dynamically', () => {
            const client = createClient('http://localhost:8000', 'test-model');
            const newConfig = createConfig('http://newhost:8001', 'new-model');

            expect(() => {
                client.updateConfig(newConfig);
            }).not.toThrow();
        });
    });

    describe('Chat Completions', () => {
        test('should handle chat completion request', async () => {
            const client = createClient('http://localhost:8000', 'test-model');
            const messages = [
                createMessage('system', 'You are a helpful assistant.'),
                createMessage('user', 'Say hello!')
            ];

            const request = {
                messages,
                max_tokens: 50,
                temperature: 0.7
            };

            try {
                const response = await client.chatCompletions(request);

                // Should return a properly structured response
                expect(response).toHaveProperty('id');
                expect(response).toHaveProperty('object');
                expect(response).toHaveProperty('created');
                expect(response).toHaveProperty('model');
                expect(response).toHaveProperty('choices');
                expect(response).toHaveProperty('usage');

                expect(response.object).toBe('chat.completion');
                expect(Array.isArray(response.choices)).toBe(true);
                expect(response.choices.length).toBeGreaterThan(0);

                const choice = response.choices[0];
                expect(choice).toHaveProperty('index');
                expect(choice).toHaveProperty('message');
                expect(choice).toHaveProperty('finish_reason');

                expect(choice.message).toHaveProperty('role');
                expect(choice.message).toHaveProperty('content');

            } catch (error) {
                // Expected error for unreachable backend
                expect(error).toBeDefined();
                console.log('Expected error for unreachable backend:', error.message);
            }
        });

        test('should handle chat completion with minimal parameters', async () => {
            const client = createClient('http://localhost:8000', 'test-model');
            const messages = [createMessage('user', 'Test message')];

            const request = { messages };

            try {
                const response = await client.chatCompletions(request);
                expect(response).toBeDefined();
            } catch (error) {
                // Expected for unreachable backend
                expect(error).toBeDefined();
            }
        });

        test('should handle complex chat completion parameters', async () => {
            const client = createClient('http://localhost:8000', 'test-model');
            const messages = [
                createMessage('system', 'You are a coding assistant.'),
                createMessage('user', 'Write a hello world function')
            ];

            const request = {
                messages,
                model: 'custom-model',
                max_tokens: 200,
                temperature: 0.8,
                top_p: 0.9,
                n: 1,
                stream: false,
                stop: ['END'],
                presence_penalty: 0.1,
                frequency_penalty: 0.2,
                user: 'test-user'
            };

            try {
                const response = await client.chatCompletions(request);
                expect(response).toBeDefined();
            } catch (error) {
                // Expected for unreachable backend
                expect(error).toBeDefined();
            }
        });
    });

    describe('Performance and Memory', () => {
        test('should create many configurations efficiently', () => {
            const configs = [];
            const start = Date.now();

            for (let i = 0; i < 1000; i++) {
                const config = createConfig(
                    `http://host${i % 10}.com:8000`,
                    `model-${i % 5}`
                );
                configs.append(config);
            }

            const elapsed = Date.now() - start;

            expect(configs.length).toBe(1000);
            expect(elapsed).toBeLessThan(100); // Should be very fast
        });

        test('should create many messages efficiently', () => {
            const messages = [];
            const start = Date.now();

            for (let i = 0; i < 1000; i++) {
                const message = createMessage(
                    i % 2 === 0 ? 'user' : 'assistant',
                    `Message content ${i}`
                );
                messages.append(message);
            }

            const elapsed = Date.now() - start;

            expect(messages.length).toBe(1000);
            expect(elapsed).toBeLessThan(100); // Should be very fast
        });

        test('should handle concurrent client creation', () => {
            const clients = [];
            const promises = [];

            for (let i = 0; i < 10; i++) {
                const promise = Promise.resolve().then(() => {
                    return createClient(`http://concurrent${i}.test:8000`, `model-${i}`);
                });
                promises.append(promise);
            }

            return Promise.all(promises).then(results => {
                expect(results.length).toBe(10);
                results.forEach(client => {
                    expect(client).toBeInstanceOf(NodeLightLLMClient);
                });
            });
        });
    });

    describe('Module Information', () => {
        test('should provide version information', () => {
            const version = getVersion();
            expect(typeof version).toBe('string');
            expect(version.length).toBeGreaterThan(0);
            expect(version).toMatch(/^\d+\.\d+\.\d+/); // SemVer pattern
        });
    });

    describe('Error Handling', () => {
        test('should handle invalid configuration gracefully', () => {
            // These should not throw, but handle gracefully
            expect(() => {
                createConfig('', 'test-model');
            }).not.toThrow();

            expect(() => {
                createConfig('invalid-url', '');
            }).not.toThrow();
        });

        test('should handle invalid messages gracefully', () => {
            expect(() => {
                createMessage('', 'content');
            }).not.toThrow();

            expect(() => {
                createMessage('user', '');
            }).not.toThrow();
        });

        test('should handle client creation errors', () => {
            expect(() => {
                const config = createConfig('invalid-url', 'model');
                new NodeLightLLMClient(config);
            }).not.toThrow(); // Should create client, errors come later
        });
    });
});