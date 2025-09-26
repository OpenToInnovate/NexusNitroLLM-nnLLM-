#!/usr/bin/env node
/**
 * # Performance Benchmark Controller
 * 
 * Orchestrates comprehensive performance benchmarks across Rust, Node.js, and Python
 * implementations against Mockoon server endpoints.
 */

import { spawn } from 'node:child_process';
import { promises as fs } from 'node:fs';
import path from 'node:path';

const MOCKOON_BASE = process.env.MOCKOON_BASE || 'http://localhost:3000';
const RESULTS_FILE = process.env.RESULTS_FILE || 'benchmark_results.json';
const WARMUP_SECONDS = parseInt(process.env.WARMUP_SECONDS || '30');
const BENCHMARK_SECONDS = parseInt(process.env.BENCHMARK_SECONDS || '60');

// Test configuration matrix
const routes = [
    '/v1/chat/completions',           // 200 JSON happy path
    '/v1/chat/completions:stream',    // Streaming SSE
    '/v1/chat/completions:rate',      // 429 + Retry-After
    '/v1/chat/completions:error',     // 500 error
    '/v1/chat/completions:malformed', // Malformed JSON
];
const payloadSizes = ['S', 'M', 'L']; // 1KB, 32KB, 256KB
const concurrencyLevels = [1, 8, 32, 128];

// Benchmark implementations
const implementations = [
    {
        lang: 'rust',
        cmd: ['cargo', 'run', '--bin', 'rust_benchmark', '--release'],
        cwd: process.cwd(),
        env: { RUST_LOG: 'error' }
    },
    {
        lang: 'node',
        cmd: ['node', path.join(process.cwd(), 'benches', 'node_benchmark.js')],
        cwd: process.cwd(),
        env: {}
    },
    {
        lang: 'python',
        cmd: ['python3', path.join(process.cwd(), 'benches', 'python_benchmark.py')],
        cwd: process.cwd(),
        env: {}
    }
];

// Streaming implementations (separate from regular)
const streamingImplementations = [
    {
        lang: 'rust-streaming',
        cmd: ['cargo', 'run', '--bin', 'rust_streaming_benchmark', '--release'],
        cwd: process.cwd(),
        env: { RUST_LOG: 'error' }
    }
];

async function runBenchmark(impl, route, concurrency, payloadSize, isStreaming = false) {
    const env = {
        ...process.env,
        ...impl.env,
        BASE: MOCKOON_BASE,
        ROUTE: route,
        C: String(concurrency),
        T: String(BENCHMARK_SECONDS),
        SIZE: payloadSize
    };

    console.log(`\n🔬 Running ${impl.lang} benchmark:`);
    console.log(`   Route: ${route}`);
    console.log(`   Concurrency: ${concurrency}`);
    console.log(`   Payload: ${payloadSize}`);
    console.log(`   Duration: ${BENCHMARK_SECONDS}s`);

    return new Promise((resolve, reject) => {
        const proc = spawn(impl.cmd[0], impl.cmd.slice(1), {
            env,
            cwd: impl.cwd,
            stdio: ['ignore', 'pipe', 'pipe']
        });

        const chunks = [];
        let errorOutput = '';

        proc.stdout.on('data', (data) => {
            chunks.push(data);
        });

        proc.stderr.on('data', (data) => {
            errorOutput += data.toString();
        });

        proc.on('close', (code) => {
            if (code !== 0) {
                console.error(`❌ Benchmark failed with code ${code}`);
                console.error(`Error output: ${errorOutput}`);
                reject(new Error(`Benchmark failed with code ${code}`));
                return;
            }

            try {
                const output = Buffer.concat(chunks).toString().trim();
                const result = JSON.parse(output);
                result.timestamp = new Date().toISOString();
                result.test_config = {
                    route,
                    concurrency,
                    payload_size: payloadSize,
                    duration_seconds: BENCHMARK_SECONDS,
                    mockoon_base: MOCKOON_BASE
                };
                resolve(result);
            } catch (parseError) {
                console.error(`❌ Failed to parse benchmark output: ${parseError.message}`);
                console.error(`Raw output: ${Buffer.concat(chunks).toString()}`);
                reject(parseError);
            }
        });

        proc.on('error', (error) => {
            console.error(`❌ Failed to start benchmark: ${error.message}`);
            reject(error);
        });
    });
}

async function runWarmup(impl, route) {
    console.log(`🔥 Warming up ${impl.lang} for ${WARMUP_SECONDS}s...`);
    
    const env = {
        ...process.env,
        ...impl.env,
        BASE: MOCKOON_BASE,
        ROUTE: route,
        C: '8',
        T: String(WARMUP_SECONDS)
    };

    return new Promise((resolve) => {
        const proc = spawn(impl.cmd[0], impl.cmd.slice(1), {
            env,
            cwd: impl.cwd,
            stdio: ['ignore', 'ignore', 'ignore']
        });

        proc.on('close', () => {
            resolve();
        });

        proc.on('error', () => {
            resolve(); // Continue even if warmup fails
        });
    });
}

function generateReport(allResults) {
    console.log('\n📊 PERFORMANCE BENCHMARK RESULTS');
    console.log('=====================================');

    // Group results by route
    const groupedResults = {};
    allResults.forEach(result => {
        const key = result.route;
        if (!groupedResults[key]) {
            groupedResults[key] = [];
        }
        groupedResults[key].push(result);
    });

    // Generate report for each route
    Object.entries(groupedResults).forEach(([route, results]) => {
        console.log(`\n🎯 Route: ${route}`);
        console.log('─'.repeat(80));
        
        // Group by concurrency and payload size
        const configGroups = {};
        results.forEach(result => {
            const configKey = `c${result.concurrency}_${result.payload_size}`;
            if (!configGroups[configKey]) {
                configGroups[configKey] = [];
            }
            configGroups[configKey].push(result);
        });

        Object.entries(configGroups).forEach(([configKey, configResults]) => {
            console.log(`\n  Configuration: ${configKey}`);
            
            // Sort by throughput (descending)
            configResults.sort((a, b) => b.throughput_rps - a.throughput_rps);
            
            configResults.forEach(result => {
                const latency = result.latency_ms;
                const errors = result.errors;
                console.log(`    ${result.lang.padEnd(12)} ` +
                          `rps=${result.throughput_rps.toFixed(0).padStart(6)} ` +
                          `p50=${latency.p50.toFixed(2).padStart(6)}ms ` +
                          `p95=${latency.p95.toFixed(2).padStart(6)}ms ` +
                          `p99=${latency.p99.toFixed(2).padStart(6)}ms ` +
                          `err=${errors.total.toString().padStart(3)} ` +
                          `succ=${result.success_rate.toFixed(1).padStart(5)}%`);
            });
        });
    });

    // Summary statistics
    console.log('\n📈 SUMMARY STATISTICS');
    console.log('─'.repeat(80));
    
    const langStats = {};
    allResults.forEach(result => {
        if (!langStats[result.lang]) {
            langStats[result.lang] = {
                total_requests: 0,
                avg_throughput: 0,
                avg_latency_p50: 0,
                avg_latency_p99: 0,
                total_errors: 0,
                count: 0
            };
        }
        
        const stats = langStats[result.lang];
        stats.total_requests += result.reqs;
        stats.avg_throughput += result.throughput_rps;
        stats.avg_latency_p50 += result.latency_ms.p50;
        stats.avg_latency_p99 += result.latency_ms.p99;
        stats.total_errors += result.errors.total;
        stats.count += 1;
    });

    // Calculate averages
    Object.keys(langStats).forEach(lang => {
        const stats = langStats[lang];
        stats.avg_throughput /= stats.count;
        stats.avg_latency_p50 /= stats.count;
        stats.avg_latency_p99 /= stats.count;
    });

    console.log('Language      | Total Reqs | Avg RPS | Avg P50 | Avg P99 | Total Errors');
    console.log('─'.repeat(70));
    Object.entries(langStats).forEach(([lang, stats]) => {
        console.log(`${lang.padEnd(12)} | ` +
                   `${stats.total_requests.toLocaleString().padStart(10)} | ` +
                   `${stats.avg_throughput.toFixed(0).padStart(7)} | ` +
                   `${stats.avg_latency_p50.toFixed(1).padStart(7)}ms | ` +
                   `${stats.avg_latency_p99.toFixed(1).padStart(7)}ms | ` +
                   `${stats.total_errors.toLocaleString().padStart(11)}`);
    });
}

async function main() {
    console.log('🚀 Starting comprehensive performance benchmark suite');
    console.log(`📡 Mockoon server: ${MOCKOON_BASE}`);
    console.log(`⏱️  Warmup: ${WARMUP_SECONDS}s, Benchmark: ${BENCHMARK_SECONDS}s`);
    console.log(`📁 Results will be saved to: ${RESULTS_FILE}`);

    const allResults = [];

    try {
        // Check if Mockoon is running
        const testResponse = await fetch(`${MOCKOON_BASE}/health`);
        if (!testResponse.ok) {
            throw new Error(`Mockoon server not responding at ${MOCKOON_BASE}`);
        }
        console.log('✅ Mockoon server is responding');

        // Run benchmarks for regular endpoints
        for (const route of routes) {
            // Skip streaming routes for non-streaming implementations
            if (route.includes(':stream')) continue;
            
            for (const payloadSize of payloadSizes) {
                for (const concurrency of concurrencyLevels) {
                    for (const impl of implementations) {
                        try {
                            // Warmup
                            await runWarmup(impl, route);
                            
                            // Run benchmark
                            const result = await runBenchmark(impl, route, concurrency, payloadSize);
                            allResults.push(result);
                            
                            // Small delay between benchmarks
                            await new Promise(resolve => setTimeout(resolve, 1000));
                        } catch (error) {
                            console.error(`❌ Benchmark failed for ${impl.lang} on ${route}: ${error.message}`);
                        }
                    }
                }
            }
        }

        // Run streaming benchmarks
        const streamingRoutes = routes.filter(r => r.includes(':stream'));
        for (const route of streamingRoutes) {
            for (const concurrency of concurrencyLevels) {
                for (const impl of streamingImplementations) {
                    try {
                        await runWarmup(impl, route);
                        const result = await runBenchmark(impl, route, concurrency, 'S', true);
                        allResults.push(result);
                        await new Promise(resolve => setTimeout(resolve, 1000));
                    } catch (error) {
                        console.error(`❌ Streaming benchmark failed for ${impl.lang} on ${route}: ${error.message}`);
                    }
                }
            }
        }

        // Generate and save results
        generateReport(allResults);
        
        // Save detailed results to JSON file
        const resultsData = {
            metadata: {
                timestamp: new Date().toISOString(),
                mockoon_base: MOCKOON_BASE,
                warmup_seconds: WARMUP_SECONDS,
                benchmark_seconds: BENCHMARK_SECONDS,
                total_tests: allResults.length
            },
            results: allResults
        };

        await fs.writeFile(RESULTS_FILE, JSON.stringify(resultsData, null, 2));
        console.log(`\n💾 Detailed results saved to: ${RESULTS_FILE}`);

        // Generate professional report
        console.log('\n📊 Generating professional benchmark report...');
        try {
            const { spawn } = await import('node:child_process');
            const reportProcess = spawn('node', ['report_generator.js', RESULTS_FILE], {
                stdio: 'inherit',
                cwd: process.cwd()
            });
            
            await new Promise((resolve, reject) => {
                reportProcess.on('close', (code) => {
                    if (code === 0) {
                        console.log('✅ Professional report generated successfully!');
                        resolve();
                    } else {
                        console.log(`⚠️  Report generation completed with code ${code}`);
                        resolve(); // Don't fail the whole benchmark suite
                    }
                });
                reportProcess.on('error', reject);
            });
        } catch (error) {
            console.log(`⚠️  Could not generate report: ${error.message}`);
            console.log('💡 Run manually: node report_generator.js benchmark_results.json');
        }

    } catch (error) {
        console.error(`❌ Benchmark suite failed: ${error.message}`);
        process.exit(1);
    }
}

// Handle graceful shutdown
process.on('SIGINT', () => {
    console.log('\n⏹️  Benchmark suite interrupted');
    process.exit(0);
});

if (import.meta.url === `file://${process.argv[1]}`) {
    main().catch(console.error);
}
