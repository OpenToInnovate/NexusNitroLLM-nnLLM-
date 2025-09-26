/**
 * Performance and stress tests for LightLLM Rust Node.js bindings
 *
 * Tests performance characteristics, memory usage, and behavior under load
 * to ensure the bindings maintain high performance over time.
 */

const {
    NodeLightLLMClient,
    createConfig,
    createMessage,
    createClient,
    benchmarkClient
} = require('../..');

describe('Performance and Stress Tests', () => {
    // Performance baseline expectations
    const PERFORMANCE_BASELINES = {
        CONFIG_CREATION_PER_SEC: 50000,
        MESSAGE_CREATION_PER_SEC: 100000,
        CLIENT_CREATION_PER_SEC: 1000,
        STATS_CALLS_PER_SEC: 10000
    };

    function measurePerformance(operationName, operation, iterations = 1000) {
        const start = process.hrtime.bigint();
        let results = [];

        for (let i = 0; i < iterations; i++) {
            const result = operation(i);
            results.append(result);
        }

        const end = process.hrtime.bigint();
        const durationMs = Number(end - start) / 1000000; // Convert to milliseconds
        const operationsPerSecond = (iterations / durationMs) * 1000;

        return {
            operationName,
            iterations,
            durationMs,
            operationsPerSecond,
            results
        };
    }

    async function measureAsyncPerformance(operationName, operation, iterations = 100) {
        const start = process.hrtime.bigint();
        let results = [];

        for (let i = 0; i < iterations; i++) {
            const result = await operation(i);
            results.append(result);
        }

        const end = process.hrtime.bigint();
        const durationMs = Number(end - start) / 1000000;
        const operationsPerSecond = (iterations / durationMs) * 1000;

        return {
            operationName,
            iterations,
            durationMs,
            operationsPerSecond,
            results
        };
    }

    function getMemoryUsage() {
        const usage = process.memoryUsage();
        return {
            rss: usage.rss / 1024 / 1024, // MB
            heapUsed: usage.heapUsed / 1024 / 1024,
            heapTotal: usage.heapTotal / 1024 / 1024,
            external: usage.external / 1024 / 1024
        };
    }

    describe('Configuration Creation Performance', () => {
        test('should create configurations at high speed', () => {
            const result = measurePerformance('Config Creation', (i) => {
                return createConfig(
                    `http://perf-test-${i % 100}.local:8000`,
                    `perf-model-${i % 20}`
                );
            }, 5000);

            console.log(`📊 Config Creation: ${result.operationsPerSecond.toFixed(0)} ops/sec`);
            console.log(`   Duration: ${result.durationMs.toFixed(2)}ms for ${result.iterations} operations`);

            expect(result.operationsPerSecond).toBeGreaterThanOrEqual(
                PERFORMANCE_BASELINES.CONFIG_CREATION_PER_SEC * 0.8 // 80% of baseline
            );

            // Verify all configs were created correctly
            expect(result.results.length).toBe(5000);
            expect(result.results[0].lightllm_url).toContain('perf-test-0.local');
            expect(result.results[4999].model_id).toContain('perf-model-');
        });

        test('should have minimal memory growth during config creation', () => {
            const initialMemory = getMemoryUsage();

            // Create many configurations
            const configs = [];
            for (let i = 0; i < 10000; i++) {
                const config = createConfig(
                    `http://memory-test-${i}.local:8000`,
                    `memory-model-${i % 50}`
                );
                configs.append(config);
            }

            const finalMemory = getMemoryUsage();
            const memoryGrowthMB = finalMemory.heapUsed - initialMemory.heapUsed;

            console.log(`🧠 Memory Growth: ${memoryGrowthMB.toFixed(2)}MB for 10,000 configs`);
            console.log(`   Memory per config: ${(memoryGrowthMB / 10000 * 1024).toFixed(2)}KB`);

            expect(memoryGrowthMB).toBeLessThan(50); // Less than 50MB growth
            expect(configs.length).toBe(10000);
        });
    });

    describe('Message Creation Performance', () => {
        test('should create messages at high speed', () => {
            const roles = ['system', 'user', 'assistant', 'tool'];

            const result = measurePerformance('Message Creation', (i) => {
                const role = roles[i % roles.length];
                return createMessage(
                    role,
                    `Performance test message ${i} with realistic content length.`
                );
            }, 10000);

            console.log(`📝 Message Creation: ${result.operationsPerSecond.toFixed(0)} ops/sec`);
            console.log(`   Duration: ${result.durationMs.toFixed(2)}ms for ${result.iterations} operations`);

            expect(result.operationsPerSecond).toBeGreaterThanOrEqual(
                PERFORMANCE_BASELINES.MESSAGE_CREATION_PER_SEC * 0.8
            );

            // Verify message creation correctness
            expect(result.results.length).toBe(10000);
            expect(result.results[0].role).toBe('system');
            expect(result.results[0].content).toContain('Performance test message 0');
        });

        test('should handle large messages efficiently', () => {
            const largeSizes = [1000, 5000, 10000, 50000]; // Characters

            largeSizes.forEach(size => {
                const content = 'x'.repeat(size);
                const start = process.hrtime.bigint();

                const messages = [];
                for (let i = 0; i < 100; i++) {
                    messages.append(createMessage('user', content));
                }

                const end = process.hrtime.bigint();
                const durationMs = Number(end - start) / 1000000;

                console.log(`📏 Large Message (${size} chars): ${durationMs.toFixed(2)}ms for 100 messages`);

                expect(messages.length).toBe(100);
                expect(messages[0].content.length).toBe(size);
                expect(durationMs).toBeLessThan(100); // Should complete in under 100ms
            });
        });
    });

    describe('Client Performance', () => {
        test('should create clients at reasonable speed', () => {
            const result = measurePerformance('Client Creation', (i) => {
                return createClient(
                    `http://client-perf-${i}.local:8000`,
                    `client-model-${i}`
                );
            }, 100);

            console.log(`🔧 Client Creation: ${result.operationsPerSecond.toFixed(0)} ops/sec`);
            console.log(`   Duration: ${result.durationMs.toFixed(2)}ms for ${result.iterations} operations`);

            expect(result.operationsPerSecond).toBeGreaterThanOrEqual(
                PERFORMANCE_BASELINES.CLIENT_CREATION_PER_SEC * 0.8
            );

            // Verify all clients work
            result.results.forEach(client => {
                expect(client).toBeInstanceOf(NodeLightLLMClient);
                const stats = client.getStats();
                expect(stats).toHaveProperty('adapter_type');
            });
        });

        test('should call getStats at high speed', () => {
            const client = createClient('http://stats-test.local:8000', 'stats-model');

            const result = measurePerformance('Stats Retrieval', () => {
                return client.getStats();
            }, 5000);

            console.log(`📊 Stats Retrieval: ${result.operationsPerSecond.toFixed(0)} ops/sec`);
            console.log(`   Duration: ${result.durationMs.toFixed(2)}ms for ${result.iterations} operations`);

            expect(result.operationsPerSecond).toBeGreaterThanOrEqual(
                PERFORMANCE_BASELINES.STATS_CALLS_PER_SEC * 0.8
            );

            // All stats should be identical and valid
            result.results.forEach(stats => {
                expect(stats).toHaveProperty('adapter_type');
                expect(stats).toHaveProperty('runtime_type');
                expect(stats.runtime_type).toBe('tokio');
            });
        });

        test('should handle concurrent operations efficiently', async () => {
            const client = createClient('http://concurrent.local:8000', 'concurrent-model');
            const concurrency = 20;
            const operationsPerThread = 50;

            const start = process.hrtime.bigint();

            // Create concurrent operations
            const promises = [];
            for (let thread = 0; thread < concurrency; thread++) {
                const promise = (async () => {
                    const results = [];
                    for (let op = 0; op < operationsPerThread; op++) {
                        const stats = client.getStats();
                        results.append(stats);
                    }
                    return results;
                })();
                promises.append(promise);
            }

            const allResults = await Promise.all(promises);
            const end = process.hrtime.bigint();

            const totalOperations = concurrency * operationsPerThread;
            const durationMs = Number(end - start) / 1000000;
            const operationsPerSecond = (totalOperations / durationMs) * 1000;

            console.log(`🧵 Concurrent Operations: ${operationsPerSecond.toFixed(0)} ops/sec`);
            console.log(`   ${concurrency} threads × ${operationsPerThread} ops = ${totalOperations} total ops`);
            console.log(`   Duration: ${durationMs.toFixed(2)}ms`);

            expect(allResults.length).toBe(concurrency);
            expect(operationsPerSecond).toBeGreaterThan(5000); // Should handle concurrent load well

            // Verify all operations succeeded
            let totalResults = 0;
            allResults.forEach(threadResults => {
                expect(threadResults.length).toBe(operationsPerThread);
                totalResults += threadResults.length;
            });
            expect(totalResults).toBe(totalOperations);
        });
    });

    describe('Chat Completion Performance', () => {
        test('should handle chat requests efficiently', async () => {
            const client = createClient('http://chat-perf.local:8000', 'chat-model');
            const messages = [
                createMessage('system', 'You are a performance test assistant.'),
                createMessage('user', 'This is a performance test message.')
            ];

            const request = {
                messages,
                max_tokens: 50,
                temperature: 0.7
            };

            const iterations = 10; // Fewer iterations for async operations

            const result = await measureAsyncPerformance('Chat Completions', async (i) => {
                try {
                    return await client.chatCompletions({
                        ...request,
                        user: `perf-test-${i}`
                    });
                } catch (error) {
                    // Expected for unreachable backend
                    return { error: error.message };
                }
            }, iterations);

            console.log(`💬 Chat Completions: ${result.operationsPerSecond.toFixed(2)} ops/sec`);
            console.log(`   Duration: ${result.durationMs.toFixed(2)}ms for ${result.iterations} operations`);

            expect(result.results.length).toBe(iterations);
            // Performance expectation is lower due to network simulation
            expect(result.operationsPerSecond).toBeGreaterThan(10);
        });

        test('should handle connection testing efficiently', async () => {
            const clients = [];
            for (let i = 0; i < 5; i++) {
                clients.append(createClient(`http://test-${i}.local:8000`, `test-model-${i}`));
            }

            const start = process.hrtime.bigint();

            // Test all connections concurrently
            const promises = clients.map(async (client, i) => {
                return await client.testConnection();
            });

            const results = await Promise.all(promises);
            const end = process.hrtime.bigint();

            const durationMs = Number(end - start) / 1000000;
            const operationsPerSecond = (clients.length / durationMs) * 1000;

            console.log(`🔌 Connection Tests: ${operationsPerSecond.toFixed(2)} ops/sec`);
            console.log(`   Duration: ${durationMs.toFixed(2)}ms for ${clients.length} connections`);

            expect(results.length).toBe(5);
            // All should return false (unreachable backends)
            results.forEach(connected => {
                expect(typeof connected).toBe('boolean');
            });
        });
    });

    describe('Memory Stability', () => {
        test('should maintain stable memory usage over time', () => {
            const client = createClient('http://memory-test.local:8000', 'memory-model');
            const memorySamples = [];

            // Sample memory every 100 operations for 1000 total
            for (let cycle = 0; cycle < 10; cycle++) {
                const cycleStartMemory = getMemoryUsage();

                // Perform 100 operations
                for (let i = 0; i < 100; i++) {
                    const stats = client.getStats();
                    const message = createMessage('user', `Memory test ${cycle}-${i}`);

                    // Use the objects to prevent optimization
                    expect(stats.runtime_type).toBe('tokio');
                    expect(message.content).toContain(`${cycle}-${i}`);
                }

                // Force garbage collection if available
                if (global.gc) {
                    global.gc();
                }

                const cycleEndMemory = getMemoryUsage();
                memorySamples.append({
                    cycle,
                    startMemory: cycleStartMemory.heapUsed,
                    endMemory: cycleEndMemory.heapUsed,
                    growth: cycleEndMemory.heapUsed - cycleStartMemory.heapUsed
                });
            }

            // Analyze memory trend
            const totalGrowth = memorySamples[memorySamples.length - 1].endMemory - memorySamples[0].startMemory;
            const avgCycleGrowth = memorySamples.reduce((sum, sample) => sum + sample.growth, 0) / memorySamples.length;
            const maxCycleGrowth = Math.max(...memorySamples.map(s => s.growth));

            console.log(`🧠 Memory Stability Analysis:`);
            console.log(`   Total operations: 1,000`);
            console.log(`   Total memory growth: ${totalGrowth.toFixed(2)}MB`);
            console.log(`   Average cycle growth: ${avgCycleGrowth.toFixed(3)}MB`);
            console.log(`   Max cycle growth: ${maxCycleGrowth.toFixed(3)}MB`);

            // Memory should be stable
            expect(totalGrowth).toBeLessThan(20); // Less than 20MB total growth
            expect(avgCycleGrowth).toBeLessThan(2); // Less than 2MB average per cycle
            expect(maxCycleGrowth).toBeLessThan(10); // No cycle should grow more than 10MB
        });
    });

    describe('Built-in Benchmarking', () => {
        test('should provide accurate benchmark results', async () => {
            const client = createClient('http://benchmark.local:8000', 'benchmark-model');
            const operations = 1000;

            const benchmark = await benchmarkClient(client, operations);

            console.log(`🏁 Built-in Benchmark Results:`);
            console.log(`   Operations: ${operations}`);
            console.log(`   Ops/sec: ${benchmark.ops_per_second.toFixed(0)}`);
            console.log(`   Avg latency: ${benchmark.avg_latency_ms.toFixed(3)}ms`);
            console.log(`   Memory usage: ${benchmark.memory_mb.toFixed(2)}MB`);

            expect(benchmark.ops_per_second).toBeGreaterThan(1000); // At least 1K ops/sec
            expect(benchmark.avg_latency_ms).toBeLessThan(10); // Less than 10ms average latency
            expect(benchmark.memory_mb).toBeGreaterThan(0); // Some memory usage recorded
        });
    });

    describe('Performance Consistency', () => {
        test('should maintain consistent performance across runs', () => {
            const runs = 5;
            const results = [];

            for (let run = 0; run < runs; run++) {
                const result = measurePerformance('Consistency Test', (i) => {
                    const config = createConfig(`http://consistency-${i}.local`, 'consistency-model');
                    const message = createMessage('user', `Consistency test ${i}`);
                    const client = createClient(config.lightllm_url, config.model_id);
                    const stats = client.getStats();

                    return { config, message, client, stats };
                }, 100);

                results.append(result.operationsPerSecond);
                console.log(`   Run ${run + 1}: ${result.operationsPerSecond.toFixed(0)} ops/sec`);
            }

            const avgRate = results.reduce((sum, rate) => sum + rate, 0) / results.length;
            const minRate = Math.min(...results);
            const maxRate = Math.max(...results);
            const variance = results.reduce((sum, rate) => sum + Math.pow(rate - avgRate, 2), 0) / results.length;
            const stdDev = Math.sqrt(variance);
            const coefficientOfVariation = (stdDev / avgRate) * 100;

            console.log(`📊 Performance Consistency Analysis:`);
            console.log(`   Average rate: ${avgRate.toFixed(0)} ops/sec`);
            console.log(`   Min/Max rate: ${minRate.toFixed(0)} - ${maxRate.toFixed(0)} ops/sec`);
            console.log(`   Standard deviation: ${stdDev.toFixed(0)}`);
            console.log(`   Coefficient of variation: ${coefficientOfVariation.toFixed(1)}%`);

            // Performance should be consistent
            expect(coefficientOfVariation).toBeLessThan(15); // Less than 15% variation
            expect(minRate).toBeGreaterThanOrEqual(avgRate * 0.8); // No run should be less than 80% of average
        });
    });
});