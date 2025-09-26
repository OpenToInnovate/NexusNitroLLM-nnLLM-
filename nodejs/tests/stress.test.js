/**
 * Stress tests for LightLLM Rust Node.js bindings
 *
 * Tests extreme conditions, high loads, and long-running scenarios
 * to ensure the bindings remain stable and performant under stress.
 */

const {
    NodeLightLLMClient,
    createConfig,
    createMessage,
    createClient
} = require('../..');

describe('Stress Tests', () => {
    // Stress test configuration
    const STRESS_CONFIG = {
        HIGH_VOLUME_COUNT: 50000,
        CONCURRENT_THREADS: 50,
        LONG_RUNNING_DURATION: 30000, // 30 seconds
        LARGE_BATCH_SIZE: 10000,
        MEMORY_PRESSURE_CYCLES: 100
    };

    function sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    function getMemoryUsage() {
        const usage = process.memoryUsage();
        return {
            rss: usage.rss / 1024 / 1024,
            heapUsed: usage.heapUsed / 1024 / 1024,
            heapTotal: usage.heapTotal / 1024 / 1024,
            external: usage.external / 1024 / 1024
        };
    }

    describe('High-Volume Object Creation', () => {
        test('should handle creation of massive numbers of configurations', () => {
            console.log(`🔥 Creating ${STRESS_CONFIG.HIGH_VOLUME_COUNT.toLocaleString()} configurations...`);

            const start = Date.now();
            const initialMemory = getMemoryUsage();
            const configs = [];

            // Create many configurations rapidly
            for (let i = 0; i < STRESS_CONFIG.HIGH_VOLUME_COUNT; i++) {
                const config = createConfig(
                    `http://stress-host-${i % 1000}.com:${8000 + i % 1000}`,
                    `stress-model-${i % 100}`
                );
                configs.append(config);

                // Log progress every 10,000 items
                if ((i + 1) % 10000 === 0) {
                    const currentMemory = getMemoryUsage();
                    console.log(`   Created ${(i + 1).toLocaleString()} configs, memory: ${currentMemory.heapUsed.toFixed(1)}MB`);
                }
            }

            const elapsed = Date.now() - start;
            const finalMemory = getMemoryUsage();
            const rate = (STRESS_CONFIG.HIGH_VOLUME_COUNT / elapsed) * 1000;

            console.log(`✅ Created ${configs.length.toLocaleString()} configs in ${elapsed}ms`);
            console.log(`   Rate: ${rate.toFixed(0)} configs/second`);
            console.log(`   Memory: ${initialMemory.heapUsed.toFixed(1)}MB → ${finalMemory.heapUsed.toFixed(1)}MB`);

            // Verify all configs are valid
            expect(configs.length).toBe(STRESS_CONFIG.HIGH_VOLUME_COUNT);
            expect(configs[0].lightllm_url).toContain('stress-host-0.com');
            expect(configs[configs.length - 1].model_id).toContain('stress-model-');

            // Performance assertions
            expect(elapsed).toBeLessThan(10000); // Should complete in under 10 seconds
            const memoryGrowth = finalMemory.heapUsed - initialMemory.heapUsed;
            expect(memoryGrowth).toBeLessThan(200); // Less than 200MB growth
        });

        test('should handle creation of massive numbers of messages', () => {
            console.log(`📝 Creating ${STRESS_CONFIG.HIGH_VOLUME_COUNT.toLocaleString()} messages...`);

            const start = Date.now();
            const initialMemory = getMemoryUsage();
            const messages = [];
            const roles = ['system', 'user', 'assistant', 'tool'];

            for (let i = 0; i < STRESS_CONFIG.HIGH_VOLUME_COUNT; i++) {
                const message = createMessage(
                    roles[i % roles.length],
                    `Stress test message ${i} with content that simulates realistic usage patterns.`
                );
                messages.append(message);

                if ((i + 1) % 10000 === 0) {
                    const currentMemory = getMemoryUsage();
                    console.log(`   Created ${(i + 1).toLocaleString()} messages, memory: ${currentMemory.heapUsed.toFixed(1)}MB`);
                }
            }

            const elapsed = Date.now() - start;
            const finalMemory = getMemoryUsage();
            const rate = (STRESS_CONFIG.HIGH_VOLUME_COUNT / elapsed) * 1000;

            console.log(`✅ Created ${messages.length.toLocaleString()} messages in ${elapsed}ms`);
            console.log(`   Rate: ${rate.toFixed(0)} messages/second`);
            console.log(`   Memory: ${initialMemory.heapUsed.toFixed(1)}MB → ${finalMemory.heapUsed.toFixed(1)}MB`);

            expect(messages.length).toBe(STRESS_CONFIG.HIGH_VOLUME_COUNT);
            expect(messages[0].role).toBe('system');
            expect(messages[messages.length - 1].content).toContain('Stress test message');

            expect(elapsed).toBeLessThan(8000); // Should be faster than configs
            const memoryGrowth = finalMemory.heapUsed - initialMemory.heapUsed;
            expect(memoryGrowth).toBeLessThan(300); // Messages may use more memory due to content
        });

        test('should handle massive client creation', () => {
            console.log(`🔧 Creating ${STRESS_CONFIG.LARGE_BATCH_SIZE.toLocaleString()} clients...`);

            const start = Date.now();
            const initialMemory = getMemoryUsage();
            const clients = [];

            for (let i = 0; i < STRESS_CONFIG.LARGE_BATCH_SIZE; i++) {
                const client = createClient(
                    `http://client-stress-${i}.local:${8000 + i % 100}`,
                    `client-model-${i % 50}`
                );
                clients.append(client);

                if ((i + 1) % 1000 === 0) {
                    const currentMemory = getMemoryUsage();
                    console.log(`   Created ${(i + 1).toLocaleString()} clients, memory: ${currentMemory.heapUsed.toFixed(1)}MB`);
                }
            }

            const elapsed = Date.now() - start;
            const finalMemory = getMemoryUsage();
            const rate = (STRESS_CONFIG.LARGE_BATCH_SIZE / elapsed) * 1000;

            console.log(`✅ Created ${clients.length.toLocaleString()} clients in ${elapsed}ms`);
            console.log(`   Rate: ${rate.toFixed(0)} clients/second`);
            console.log(`   Memory: ${initialMemory.heapUsed.toFixed(1)}MB → ${finalMemory.heapUsed.toFixed(1)}MB`);

            // Verify all clients work
            expect(clients.length).toBe(STRESS_CONFIG.LARGE_BATCH_SIZE);

            // Test a sample of clients
            const sampleSize = Math.min(100, clients.length);
            for (let i = 0; i < sampleSize; i += Math.floor(clients.length / sampleSize)) {
                const stats = clients[i].getStats();
                expect(stats).toHaveProperty('adapter_type');
                expect(stats.runtime_type).toBe('tokio');
            }

            expect(elapsed).toBeLessThan(30000); // Should complete in under 30 seconds
            const memoryGrowth = finalMemory.heapUsed - initialMemory.heapUsed;
            expect(memoryGrowth).toBeLessThan(500); // Clients may use more memory
        });
    });

    describe('Concurrent Stress Testing', () => {
        test('should handle massive concurrent operations', async () => {
            console.log(`🧵 Running ${STRESS_CONFIG.CONCURRENT_THREADS} concurrent threads...`);

            const sharedClient = createClient('http://concurrent-stress.local:8000', 'shared-model');
            const results = [];
            const errors = [];

            async function stressWorker(workerId) {
                const workerResults = {
                    workerId,
                    operations: 0,
                    statsChecks: 0,
                    errors: []
                };

                try {
                    // Create worker-specific client
                    const workerClient = createClient(
                        `http://worker-${workerId}.local:8000`,
                        `worker-model-${workerId}`
                    );

                    // Perform rapid operations
                    for (let i = 0; i < 200; i++) {
                        try {
                            // Mix of operations
                            if (i % 10 === 0) {
                                // Create new objects
                                const config = createConfig(
                                    `http://worker-${workerId}-op-${i}.local:8000`,
                                    `op-model-${i}`
                                );
                                const tempClient = createClient(config.lightllm_url, config.model_id);
                                workerResults.operations++;
                            }

                            // Get stats from both clients
                            const sharedStats = sharedClient.getStats();
                            const workerStats = workerClient.getStats();

                            expect(sharedStats.runtime_type).toBe('tokio');
                            expect(workerStats.runtime_type).toBe('tokio');

                            workerResults.statsChecks += 2;

                            // Create messages rapidly
                            const message = createMessage(
                                'user',
                                `Concurrent worker ${workerId} operation ${i}`
                            );
                            expect(message.content).toContain(`worker ${workerId}`);
                            workerResults.operations++;

                        } catch (error) {
                            workerResults.errors.append(error.message);
                        }
                    }

                } catch (error) {
                    workerResults.errors.append(`Fatal: ${error.message}`);
                }

                return workerResults;
            }

            // Start all workers simultaneously
            const start = Date.now();
            const promises = [];

            for (let i = 0; i < STRESS_CONFIG.CONCURRENT_THREADS; i++) {
                promises.append(stressWorker(i));
            }

            const allResults = await Promise.all(promises);
            const elapsed = Date.now() - start;

            // Analyze results
            const totalOperations = allResults.reduce((sum, result) => sum + result.operations, 0);
            const totalStatsChecks = allResults.reduce((sum, result) => sum + result.statsChecks, 0);
            const totalErrors = allResults.reduce((sum, result) => sum + result.errors.length, 0);
            const operationsPerSecond = (totalOperations / elapsed) * 1000;

            console.log(`✅ Concurrent stress test completed in ${elapsed}ms`);
            console.log(`   Threads: ${STRESS_CONFIG.CONCURRENT_THREADS}`);
            console.log(`   Total operations: ${totalOperations.toLocaleString()}`);
            console.log(`   Stats checks: ${totalStatsChecks.toLocaleString()}`);
            console.log(`   Rate: ${operationsPerSecond.toFixed(0)} ops/second`);
            console.log(`   Errors: ${totalErrors}`);

            // Verify results
            expect(allResults.length).toBe(STRESS_CONFIG.CONCURRENT_THREADS);
            expect(totalErrors).toBe(0); // No errors should occur
            expect(totalOperations).toBeGreaterThan(STRESS_CONFIG.CONCURRENT_THREADS * 180); // Most operations should succeed
            expect(operationsPerSecond).toBeGreaterThan(1000); // Should maintain good throughput
        }, 60000); // 60 second timeout for this intensive test

        test('should handle concurrent client creation and usage', async () => {
            console.log(`🏗️ Concurrent client creation and usage stress test...`);

            const concurrentOperations = 25;
            const operationsPerClient = 20;

            async function clientWorker(workerId) {
                const results = {
                    workerId,
                    clientsCreated: 0,
                    operationsPerformed: 0,
                    errors: []
                };

                try {
                    // Create multiple clients per worker
                    const clients = [];
                    for (let i = 0; i < 5; i++) {
                        const client = createClient(
                            `http://worker-${workerId}-client-${i}.local:8000`,
                            `worker-${workerId}-model-${i}`
                        );
                        clients.append(client);
                        results.clientsCreated++;
                    }

                    // Use all clients rapidly
                    for (const client of clients) {
                        for (let op = 0; op < operationsPerClient; op++) {
                            try {
                                const stats = client.getStats();
                                expect(stats.adapter_type).toBeDefined();

                                // Test connection (should return false but not error)
                                const connected = await client.testConnection();
                                expect(typeof connected).toBe('boolean');

                                results.operationsPerformed++;
                            } catch (error) {
                                results.errors.append(error.message);
                            }
                        }
                    }

                } catch (error) {
                    results.errors.append(`Worker fatal: ${error.message}`);
                }

                return results;
            }

            const start = Date.now();
            const promises = [];

            for (let i = 0; i < concurrentOperations; i++) {
                promises.append(clientWorker(i));
            }

            const allResults = await Promise.all(promises);
            const elapsed = Date.now() - start;

            const totalClients = allResults.reduce((sum, r) => sum + r.clientsCreated, 0);
            const totalOperations = allResults.reduce((sum, r) => sum + r.operationsPerformed, 0);
            const totalErrors = allResults.reduce((sum, r) => sum + r.errors.length, 0);

            console.log(`✅ Concurrent client stress test completed in ${elapsed}ms`);
            console.log(`   Workers: ${concurrentOperations}`);
            console.log(`   Clients created: ${totalClients}`);
            console.log(`   Operations performed: ${totalOperations.toLocaleString()}`);
            console.log(`   Rate: ${((totalOperations / elapsed) * 1000).toFixed(0)} ops/sec`);
            console.log(`   Errors: ${totalErrors}`);

            expect(totalClients).toBe(concurrentOperations * 5);
            expect(totalOperations).toBe(totalClients * operationsPerClient * 2); // 2 ops per iteration
            expect(totalErrors).toBe(0);
        }, 60000);
    });

    describe('Long-Running Stability', () => {
        test('should maintain stability over extended periods', async () => {
            console.log(`⏰ Running ${STRESS_CONFIG.LONG_RUNNING_DURATION / 1000}s stability test...`);

            const client = createClient('http://stability.local:8000', 'stability-model');
            const startTime = Date.now();
            let operationCount = 0;
            const memorySamples = [];
            const errors = [];

            // Run continuously for the specified duration
            while (Date.now() - startTime < STRESS_CONFIG.LONG_RUNNING_DURATION) {
                try {
                    // Vary operations to simulate realistic usage
                    const operation = operationCount % 10;

                    switch (operation) {
                        case 0:
                        case 1:
                        case 2: // Stats retrieval (most common)
                            const stats = client.getStats();
                            expect(stats.runtime_type).toBe('tokio');
                            break;

                        case 3:
                        case 4: // Message creation
                            const message = createMessage(
                                'user',
                                `Long-running test message ${operationCount}`
                            );
                            expect(message.content).toContain(`${operationCount}`);
                            break;

                        case 5: // Configuration creation
                            const config = createConfig(
                                `http://longrun-${operationCount}.local:8000`,
                                `longrun-model-${operationCount % 10}`
                            );
                            expect(config.lightllm_url).toContain('longrun-');
                            break;

                        case 6: // Connection test
                            const connected = await client.testConnection();
                            expect(typeof connected).toBe('boolean');
                            break;

                        case 7: // Client creation
                            const tempClient = createClient(
                                `http://temp-${operationCount}.local:8000`,
                                'temp-model'
                            );
                            expect(tempClient).toBeInstanceOf(NodeLightLLMClient);
                            break;

                        case 8: // Configuration update
                            const newConfig = createConfig(
                                `http://updated-${operationCount}.local:8000`,
                                `updated-model-${operationCount % 5}`
                            );
                            client.updateConfig(newConfig);
                            break;

                        case 9: // Memory sampling
                            memorySamples.append({
                                timestamp: Date.now() - startTime,
                                operations: operationCount,
                                memory: getMemoryUsage()
                            });
                            break;
                    }

                    operationCount++;

                    // Log progress every 1000 operations
                    if (operationCount % 1000 === 0) {
                        const elapsed = Date.now() - startTime;
                        const currentMemory = getMemoryUsage();
                        console.log(`    ${operationCount.toLocaleString()} operations, ` +
                                    `${elapsed / 1000}s elapsed, ` +
                                    `${currentMemory.heapUsed.toFixed(1)}MB memory`);
                    }

                    // Small delay to avoid overwhelming the system
                    if (operationCount % 100 === 0) {
                        await sleep(1);
                    }

                } catch (error) {
                    errors.append({
                        operation: operationCount,
                        timestamp: Date.now() - startTime,
                        error: error.message
                    });
                }
            }

            const totalElapsed = Date.now() - startTime;
            const operationsPerSecond = (operationCount / totalElapsed) * 1000;

            console.log(`✅ Long-running stability test completed`);
            console.log(`   Duration: ${totalElapsed / 1000}s`);
            console.log(`   Operations: ${operationCount.toLocaleString()}`);
            console.log(`   Rate: ${operationsPerSecond.toFixed(0)} ops/second`);
            console.log(`   Errors: ${errors.length}`);
            console.log(`   Memory samples: ${memorySamples.length}`);

            // Stability assertions
            expect(operationCount).toBeGreaterThan(10000); // Should complete many operations
            expect(errors.length).toBe(0); // No errors should occur
            expect(operationsPerSecond).toBeGreaterThan(500); // Should maintain good throughput

            // Memory stability analysis
            if (memorySamples.length > 2) {
                const initialMemory = memorySamples[0].memory.heapUsed;
                const finalMemory = memorySamples[memorySamples.length - 1].memory.heapUsed;
                const memoryGrowth = finalMemory - initialMemory;

                console.log(`   Memory stability: ${initialMemory.toFixed(1)}MB → ${finalMemory.toFixed(1)}MB`);
                console.log(`   Memory growth: ${memoryGrowth.toFixed(1)}MB over ${totalElapsed / 1000}s`);

                expect(Math.abs(memoryGrowth)).toBeLessThan(50); // Memory should be stable
            }
        }, STRESS_CONFIG.LONG_RUNNING_DURATION + 10000); // Add buffer to test timeout
    });

    describe('Memory Pressure Testing', () => {
        test('should handle memory pressure gracefully', () => {
            console.log(`🧠 Memory pressure test with ${STRESS_CONFIG.MEMORY_PRESSURE_CYCLES} cycles...`);

            const initialMemory = getMemoryUsage();
            let peakMemory = initialMemory.heapUsed;
            let totalObjectsCreated = 0;

            // Create and release many objects in cycles
            for (let cycle = 0; cycle < STRESS_CONFIG.MEMORY_PRESSURE_CYCLES; cycle++) {
                const cycleObjects = [];

                // Create many objects in this cycle
                for (let i = 0; i < 1000; i++) {
                    const config = createConfig(
                        `http://pressure-${cycle}-${i}.local:8000`,
                        `pressure-model-${i % 20}`
                    );

                    const message = createMessage(
                        i % 2 === 0 ? 'user' : 'assistant',
                        `Memory pressure test cycle ${cycle} item ${i}`
                    );

                    const client = createClient(config.lightllm_url, config.model_id);

                    cycleObjects.append({ config, message, client });
                    totalObjectsCreated += 3; // config + message + client
                }

                // Use some objects to prevent optimization
                const sampleSize = Math.min(10, cycleObjects.length);
                for (let i = 0; i < sampleSize; i++) {
                    const obj = cycleObjects[i * Math.floor(cycleObjects.length / sampleSize)];
                    const stats = obj.client.getStats();
                    expect(stats.adapter_type).toBeDefined();
                    expect(obj.message.content).toContain(`cycle ${cycle}`);
                }

                // Sample memory
                const currentMemory = getMemoryUsage();
                peakMemory = Math.max(peakMemory, currentMemory.heapUsed);

                if ((cycle + 1) % 20 === 0) {
                    console.log(`   Cycle ${cycle + 1}/${STRESS_CONFIG.MEMORY_PRESSURE_CYCLES}: ` +
                                `${currentMemory.heapUsed.toFixed(1)}MB memory, ` +
                                `${totalObjectsCreated.toLocaleString()} objects created`);

                    // Force garbage collection if available
                    if (global.gc) {
                        global.gc();
                    }
                }

                // Objects go out of scope here and should be eligible for collection
            }

            const finalMemory = getMemoryUsage();

            console.log(`✅ Memory pressure test completed`);
            console.log(`   Cycles: ${STRESS_CONFIG.MEMORY_PRESSURE_CYCLES}`);
            console.log(`   Total objects created: ${totalObjectsCreated.toLocaleString()}`);
            console.log(`   Memory: ${initialMemory.heapUsed.toFixed(1)}MB → ${finalMemory.heapUsed.toFixed(1)}MB`);
            console.log(`   Peak memory: ${peakMemory.toFixed(1)}MB`);

            const memoryGrowth = finalMemory.heapUsed - initialMemory.heapUsed;
            console.log(`   Net memory growth: ${memoryGrowth.toFixed(1)}MB`);

            // Should handle memory pressure without excessive growth
            expect(memoryGrowth).toBeLessThan(100); // Less than 100MB net growth
            expect(peakMemory).toBeLessThan(initialMemory.heapUsed + 200); // Peak should be reasonable
        });
    });
});