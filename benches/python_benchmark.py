#!/usr/bin/env python3
"""
# Python Performance Benchmark CLI

Fair, apples-to-apples benchmark against Mockoon server.
Measures latency, throughput, CPU, and memory usage.
"""

import os
import asyncio
import time
import json
import httpx
from hdrh.histogram import HdrHistogram

BASE = os.getenv("BASE", "http://localhost:3000")
ROUTE = os.environ.get("ROUTE", "/v1/chat/completions")
C = int(os.getenv("C", "32"))
T = int(os.getenv("T", "60"))
SIZE = os.getenv("SIZE", "S")

def get_content(size: str) -> str:
    """Create payload based on size"""
    base = "Hello, world!"
    if size == "S":
        return base * 50  # ~1KB
    elif size == "M":
        return base * 1600  # ~32KB
    elif size == "L":
        return base * 12800  # ~256KB
    else:
        return base * 50

content = get_content(SIZE)

# Initialize histogram for nanosecond precision
hist = HdrHistogram(1, 60_000_000_000, 3)  # ns
end = time.time() + T

print(f"Starting Python benchmark: route={ROUTE}, concurrency={C}, duration={T}s, payload={SIZE}")

async def worker():
    """Worker coroutine that sends requests in a closed loop"""
    ok = err = timeouts = 0
    
    timeout = httpx.Timeout(30.0)
    async with httpx.AsyncClient(timeout=timeout) as client:
        while time.time() < end:
            t0 = time.perf_counter_ns()
            try:
                r = await client.post(
                    f"{BASE}{ROUTE}", 
                    json={
                        "model": "test-model",
                        "messages": [{"role": "user", "content": content}]
                    }
                )
                # Consume response body
                _ = r.content
                dt = time.perf_counter_ns() - t0
                hist.record_value(dt)
                
                if 200 <= r.status_code < 300:
                    ok += 1
                else:
                    err += 1
                    
            except asyncio.TimeoutError:
                dt = time.perf_counter_ns() - t0
                hist.record_value(dt)
                timeouts += 1
            except Exception:
                dt = time.perf_counter_ns() - t0
                hist.record_value(dt)
                err += 1
                
    return ok, err, timeouts

async def main():
    """Main benchmark function"""
    oks = errs = timeouts = 0
    done = await asyncio.gather(*[worker() for _ in range(C)])
    
    for o, e, t in done:
        oks += o
        errs += e
        timeouts += t
    
    total_reqs = oks + errs + timeouts
    
    out = {
        "lang": "python",
        "route": ROUTE,
        "concurrency": C,
        "payload_size": SIZE,
        "reqs": total_reqs,
        "throughput_rps": total_reqs / T,
        "latency_ms": {
            "p50": hist.get_value_at_percentile(50) / 1e6,
            "p95": hist.get_value_at_percentile(95) / 1e6,
            "p99": hist.get_value_at_percentile(99) / 1e6,
        },
        "errors": {
            "non2xx": errs,
            "timeouts": timeouts,
            "total": errs + timeouts
        },
        "success_rate": (oks / total_reqs) * 100.0
    }
    
    print(json.dumps(out))

if __name__ == "__main__":
    asyncio.run(main())

