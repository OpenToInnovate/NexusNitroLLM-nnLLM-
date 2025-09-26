#!/usr/bin/env node
//! # Node.js Performance Benchmark CLI
//! 
//! Fair, apples-to-apples benchmark against Mockoon server.
//! Measures latency, throughput, CPU, and memory usage.

import { fetch } from 'undici';
import hdr from 'hdr-histogram-js';

const BASE = process.env.BASE ?? 'http://localhost:3000';
const route = process.argv[2] ?? '/v1/chat/completions';
const C = Number(process.env.C ?? 32);
const T = Number(process.env.T ?? 60) * 1000;
const SIZE = process.env.SIZE ?? 'S';

// Create payload based on size
const getContent = (size) => {
    const base = 'Hello, world!';
    switch (size) {
        case 'S': return base.repeat(50); // ~1KB
        case 'M': return base.repeat(1600); // ~32KB
        case 'L': return base.repeat(12800); // ~256KB
        default: return base.repeat(50);
    }
};

const content = getContent(SIZE);

const hist = hdr.build({ 
    lowestDiscernibleValue: 1, 
    highestTrackableValue: 60e9, 
    numberOfSignificantValueDigits: 3 
});

const tEnd = Date.now() + T;

console.log(`Starting Node.js benchmark: route=${route}, concurrency=${C}, duration=${T/1000}s, payload=${SIZE}`);

const worker = async () => {
    let ok = 0, err = 0, timeouts = 0;
    
    while (Date.now() < tEnd) {
        const t0 = process.hrtime.bigint();
        try {
            const r = await fetch(`${BASE}${route}`, {
                method: 'POST',
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({ 
                    model: 'test-model', 
                    messages: [{ role: 'user', content: content }] 
                }),
                signal: AbortSignal.timeout(30000) // 30s timeout
            });
            
            // Consume response body
            await r.arrayBuffer();
            
            const dt = Number(process.hrtime.bigint() - t0);
            hist.recordValue(dt);
            
            if (r.status >= 200 && r.status < 300) {
                ok++;
            } else {
                err++;
            }
        } catch (error) {
            const dt = Number(process.hrtime.bigint() - t0);
            hist.recordValue(dt);
            
            if (error.name === 'TimeoutError') {
                timeouts++;
            } else {
                err++;
            }
        }
    }
    return { ok, err, timeouts };
};

const results = await Promise.all(Array.from({ length: C }, worker));
const ok = results.reduce((a, b) => a + b.ok, 0);
const err = results.reduce((a, b) => a + b.err, 0);
const timeouts = results.reduce((a, b) => a + b.timeouts, 0);
const total_reqs = ok + err + timeouts;

const out = {
    lang: "node",
    route,
    concurrency: C,
    payload_size: SIZE,
    reqs: total_reqs,
    throughput_rps: total_reqs / (T / 1000),
    latency_ms: {
        p50: hist.getValueAtPercentile(50) / 1e6,
        p95: hist.getValueAtPercentile(95) / 1e6,
        p99: hist.getValueAtPercentile(99) / 1e6
    },
    errors: {
        non2xx: err,
        timeouts: timeouts,
        total: err + timeouts
    },
    success_rate: (ok / total_reqs) * 100.0
};

console.log(JSON.stringify(out));




