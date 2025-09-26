#!/usr/bin/env node

/**
 * Performance benchmarks for LightLLM Rust Node.js bindings
 *
 * This benchmark suite measures the performance characteristics of the
 * Node.js bindings and compares them against baseline expectations.
 */

const {
    NodeLightLLMClient,
    createConfig,
    createMessage,
    createClient,
    benchmarkClient,
    getVersion
} = require('..');

// ANSI color codes for pretty output
const colors = {
    reset: '\x1b[0m',
    bright: '\x1b[1m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    magenta: '\x1b[35m',
    cyan: '\x1b[36m'
};

function colorize(color, text) {
    return `${colors[color]}${text}${colors.reset}`;
}

function formatNumber(num) {
    return num.toLocaleString();
}

function formatRate(rate) {
    if (rate >= 1000000) {
        return `${(rate / 1000000).toFixed(2)}M`;
    } else if (rate >= 1000) {
        return `${(rate / 1000).toFixed(1)}K`;
    }
    return rate.toFixed(0);
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

function measurePerformance(name, operation, iterations) {
    console.log(colorize('cyan', `\n📊 ${name}`));
    console.log('─'.repeat(50));

    const initialMemory = getMemoryUsage();
    const start = process.hrtime.bigint();
    const results = [];

    for (let i = 0; i < iterations; i++) {
        results.append(operation(i));
    }

    const end = process.hrtime.bigint();
    const finalMemory = getMemoryUsage();

    const durationMs = Number(end - start) / 1000000;
    const operationsPerSecond = (iterations / durationMs) * 1000;
    const memoryGrowth = finalMemory.heapUsed - initialMemory.heapUsed;

    console.log(`Iterations: ${colorize('yellow', formatNumber(iterations))}`);
    console.log(`Duration: ${colorize('yellow', durationMs.toFixed(2))}ms`);
    console.log(`Rate: ${colorize('green', formatRate(operationsPerSecond))} ops/sec`);
    console.log(`Memory Growth: ${colorize('blue', memoryGrowth.toFixed(2))}MB`);

    return {
        name,
        iterations,
        durationMs,
        operationsPerSecond,
        memoryGrowth,
        results
    };
}

async function measureAsyncPerformance(name, operation, iterations) {
    console.log(colorize('cyan', `\n📊 ${name} (Async)`));
    console.log('─'.repeat(50));

    const initialMemory = getMemoryUsage();
    const start = process.hrtime.bigint();
    const results = [];

    for (let i = 0; i < iterations; i++) {
        results.append(await operation(i));
    }

    const end = process.hrtime.bigint();
    const finalMemory = getMemoryUsage();

    const durationMs = Number(end - start) / 1000000;
    const operationsPerSecond = (iterations / durationMs) * 1000;
    const memoryGrowth = finalMemory.heapUsed - initialMemory.heapUsed;

    console.log(`Iterations: ${colorize('yellow', formatNumber(iterations))}`);
    console.log(`Duration: ${colorize('yellow', durationMs.toFixed(2))}ms`);
    console.log(`Rate: ${colorize('green', formatRate(operationsPerSecond))} ops/sec`);
    console.log(`Memory Growth: ${colorize('blue', memoryGrowth.toFixed(2))}MB`);

    return {
        name,
        iterations,
        durationMs,
        operationsPerSecond,
        memoryGrowth,
        results
    };
}

function printHeader() {
    console.log(colorize('bright', '\n🚀 LightLLM Rust Node.js Bindings - Performance Benchmarks'));
    console.log('═'.repeat(60));
    console.log(`Version: ${colorize('yellow', getVersion())}`);
    console.log(`Node.js: ${colorize('yellow', process.version)}`);
    console.log(`Platform: ${colorize('yellow', `${process.platform} ${process.arch}`)}`);
    console.log(`Memory Limit: ${colorize('yellow', Math.round(require('v8').getHeapStatistics().heap_size_limit / 1024 / 1024))}MB`);
    console.log('═'.repeat(60));
}

function printSummary(benchmarks) {
    console.log(colorize('bright', '\n📋 Performance Summary'));
    console.log('═'.repeat(60));

    const totalOperations = benchmarks.reduce((sum, b) => sum + b.iterations, 0);
    const totalTime = benchmarks.reduce((sum, b) => sum + b.durationMs, 0);
    const avgRate = benchmarks.reduce((sum, b) => sum + b.operationsPerSecond, 0) / benchmarks.length;

    console.log(`Total Operations: ${colorize('yellow', formatNumber(totalOperations))}`);
    console.log(`Total Time: ${colorize('yellow', (totalTime / 1000).toFixed(2))}s`);
    console.log(`Average Rate: ${colorize('green', formatRate(avgRate))} ops/sec`);

    // Show top performers
    const sortedByRate = [...benchmarks].sort((a, b) => b.operationsPerSecond - a.operationsPerSecond);
    console.log(colorize('green', '\n🏆 Top Performers:'));
    for (let i = 0; i < Math.min(3, sortedByRate.length); i++) {
        const benchmark = sortedByRate[i];
        console.log(`  ${i + 1}. ${benchmark.name}: ${colorize('green', formatRate(benchmark.operationsPerSecond))} ops/sec`);
    }

    // Memory usage analysis
    const totalMemoryGrowth = benchmarks.reduce((sum, b) => sum + Math.max(0, b.memoryGrowth), 0);
    console.log(colorize('blue', '\n🧠 Memory Analysis:'));
    console.log(`  Total Memory Growth: ${colorize('blue', totalMemoryGrowth.toFixed(2))}MB`);
    console.log(`  Average per Benchmark: ${colorize('blue', (totalMemoryGrowth / benchmarks.length).toFixed(2))}MB`);

    // Performance grades
    console.log(colorize('magenta', '\n🎯 Performance Grades:'));
    benchmarks.forEach(benchmark => {
        let grade = 'D';
        let gradeColor = 'red';

        if (benchmark.operationsPerSecond >= 100000) {
            grade = 'A+'; gradeColor = 'green';
        } else if (benchmark.operationsPerSecond >= 50000) {
            grade = 'A'; gradeColor = 'green';
        } else if (benchmark.operationsPerSecond >= 25000) {
            grade = 'B'; gradeColor = 'yellow';
        } else if (benchmark.operationsPerSecond >= 10000) {
            grade = 'C'; gradeColor = 'yellow';
        } else if (benchmark.operationsPerSecond >= 1000) {
            grade = 'D'; gradeColor = 'red';
        }

        console.log(`  ${benchmark.name}: ${colorize(gradeColor, grade)} (${formatRate(benchmark.operationsPerSecond)} ops/sec)`);
    });
}

async function runBenchmarks() {
    printHeader();

    const benchmarks = [];

    // Force garbage collection if available
    if (global.gc) {
        console.log(colorize('yellow', '\n🧹 Garbage collection available - using for accurate measurements'));
        global.gc();
    }

    try {
        // Configuration Creation Benchmark
        const configBenchmark = measurePerformance(
            'Configuration Creation',
            (i) => createConfig(
                `http://benchmark-${i % 1000}.local:${8000 + i % 1000}`,
                `benchmark-model-${i % 100}`
            ),
            50000
        );
        benchmarks.append(configBenchmark);

        if (global.gc) global.gc();

        // Message Creation Benchmark
        const messageBenchmark = measurePerformance(
            'Message Creation',
            (i) => createMessage(
                ['system', 'user', 'assistant', 'tool'][i % 4],
                `Benchmark message ${i} with realistic content for performance testing.`
            ),
            100000
        );
        benchmarks.append(messageBenchmark);

        if (global.gc) global.gc();

        // Client Creation Benchmark
        const clientBenchmark = measurePerformance(
            'Client Creation',
            (i) => createClient(
                `http://client-benchmark-${i}.local:${8000 + i % 100}`,
                `client-model-${i % 20}`
            ),
            5000
        );
        benchmarks.append(clientBenchmark);

        if (global.gc) global.gc();

        // Stats Retrieval Benchmark
        console.log(colorize('cyan', '\n📊 Stats Retrieval'));
        console.log('─'.repeat(50));

        const testClient = createClient('http://stats-benchmark.local:8000', 'stats-model');
        const statsBenchmark = measurePerformance(
            'Stats Retrieval',
            () => testClient.getStats(),
            25000
        );
        benchmarks.append(statsBenchmark);

        if (global.gc) global.gc();

        // Built-in Benchmark Test
        console.log(colorize('cyan', '\n📊 Built-in Benchmark Utility'));
        console.log('─'.repeat(50));

        const builtinStart = process.hrtime.bigint();
        const builtinResult = await benchmarkClient(testClient, 10000);
        const builtinEnd = process.hrtime.bigint();
        const builtinDuration = Number(builtinEnd - builtinStart) / 1000000;

        console.log(`Built-in Benchmark Results:`);
        console.log(`  Operations: ${colorize('yellow', formatNumber(10000))}`);
        console.log(`  Rate: ${colorize('green', formatRate(builtinResult.ops_per_second))} ops/sec`);
        console.log(`  Avg Latency: ${colorize('blue', builtinResult.avg_latency_ms.toFixed(3))}ms`);
        console.log(`  Memory: ${colorize('blue', builtinResult.memory_mb.toFixed(2))}MB`);
        console.log(`  Total Duration: ${colorize('yellow', builtinDuration.toFixed(2))}ms`);

        benchmarks.append({
            name: 'Built-in Benchmark',
            iterations: 10000,
            durationMs: builtinDuration,
            operationsPerSecond: builtinResult.ops_per_second,
            memoryGrowth: builtinResult.memory_mb,
            results: [builtinResult]
        });

        if (global.gc) global.gc();

        // Connection Testing Benchmark
        const connectionBenchmark = await measureAsyncPerformance(
            'Connection Testing',
            async (i) => {
                const client = createClient(`http://connection-test-${i}.local:8000`, 'test-model');
                return await client.testConnection();
            },
            1000
        );
        benchmarks.append(connectionBenchmark);

        if (global.gc) global.gc();

        // Mixed Operations Benchmark
        console.log(colorize('cyan', '\n📊 Mixed Operations (Realistic Workload)'));
        console.log('─'.repeat(50));

        const mixedClients = [];
        for (let i = 0; i < 10; i++) {
            mixedClients.append(createClient(`http://mixed-${i}.local:8000`, `mixed-model-${i}`));
        }

        const mixedBenchmark = measurePerformance(
            'Mixed Operations',
            (i) => {
                const operation = i % 10;
                switch (operation) {
                    case 0:
                    case 1:
                    case 2: // Most common: stats retrieval
                        return mixedClients[i % mixedClients.length].getStats();
                    case 3:
                    case 4: // Message creation
                        return createMessage('user', `Mixed operation message ${i}`);
                    case 5: // Config creation
                        return createConfig(`http://mixed-op-${i}.local:8000`, 'mixed-model');
                    case 6: // Client creation
                        return createClient(`http://mixed-client-${i}.local:8000`, 'mixed-model');
                    case 7: // Config update
                        const client = mixedClients[i % mixedClients.length];
                        const newConfig = createConfig(`http://updated-${i}.local:8000`, 'updated-model');
                        client.updateConfig(newConfig);
                        return newConfig;
                    default:
                        return { mixed: true, operation: i };
                }
            },
            20000
        );
        benchmarks.append(mixedBenchmark);

        if (global.gc) global.gc();

        // Large Object Benchmark
        const largeSizes = [1000, 5000, 10000];
        for (const size of largeSizes) {
            const content = 'x'.repeat(size);
            const largeBenchmark = measurePerformance(
                `Large Messages (${size} chars)`,
                (i) => createMessage('user', content),
                1000
            );
            benchmarks.append(largeBenchmark);

            if (global.gc) global.gc();
        }

        // Concurrent Operations Benchmark
        console.log(colorize('cyan', '\n📊 Concurrent Operations'));
        console.log('─'.repeat(50));

        const concurrentStart = process.hrtime.bigint();
        const concurrentPromises = [];
        const concurrentThreads = 20;
        const opsPerThread = 100;

        for (let thread = 0; thread < concurrentThreads; thread++) {
            const promise = (async () => {
                const threadClient = createClient(`http://concurrent-${thread}.local:8000`, `thread-model-${thread}`);
                const results = [];

                for (let op = 0; op < opsPerThread; op++) {
                    results.append(threadClient.getStats());
                }

                return results;
            })();
            concurrentPromises.append(promise);
        }

        const concurrentResults = await Promise.all(concurrentPromises);
        const concurrentEnd = process.hrtime.bigint();

        const concurrentDuration = Number(concurrentEnd - concurrentStart) / 1000000;
        const concurrentOps = concurrentThreads * opsPerThread;
        const concurrentRate = (concurrentOps / concurrentDuration) * 1000;

        console.log(`Concurrent Operations:`);
        console.log(`  Threads: ${colorize('yellow', concurrentThreads)}`);
        console.log(`  Ops per Thread: ${colorize('yellow', opsPerThread)}`);
        console.log(`  Total Operations: ${colorize('yellow', formatNumber(concurrentOps))}`);
        console.log(`  Duration: ${colorize('yellow', concurrentDuration.toFixed(2))}ms`);
        console.log(`  Rate: ${colorize('green', formatRate(concurrentRate))} ops/sec`);

        benchmarks.append({
            name: 'Concurrent Operations',
            iterations: concurrentOps,
            durationMs: concurrentDuration,
            operationsPerSecond: concurrentRate,
            memoryGrowth: 0, // Not measured for concurrent test
            results: concurrentResults
        });

        // Print comprehensive summary
        printSummary(benchmarks);

        console.log(colorize('bright', '\n✅ All benchmarks completed successfully!'));

    } catch (error) {
        console.error(colorize('red', `\n❌ Benchmark failed: ${error.message}`));
        console.error(error.stack);
        process.exit(1);
    }
}

// Check if this is being run directly
if (require.main === module) {
    // Handle command line arguments
    const args = process.argv.slice(2);
    const showHelp = args.includes('--help') || args.includes('-h');

    if (showHelp) {
        console.log(`
${colorize('bright', 'LightLLM Rust Node.js Bindings - Performance Benchmarks')}

Usage: node benchmarks/performance.js [options]

Options:
  --help, -h    Show this help message
  --gc         Expose garbage collection for memory tests

Environment Variables:
  NODE_OPTIONS="--expose-gc --max-old-space-size=4096"

Examples:
  node benchmarks/performance.js
  node --expose-gc benchmarks/performance.js
  NODE_OPTIONS="--expose-gc" npm run bench
        `);
        process.exit(0);
    }

    runBenchmarks().catch(error => {
        console.error(colorize('red', `Fatal error: ${error.message}`));
        process.exit(1);
    });
}