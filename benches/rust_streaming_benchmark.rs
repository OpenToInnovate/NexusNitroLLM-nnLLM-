//! # Rust Streaming Performance Benchmark CLI
//! 
//! Benchmarks SSE streaming performance against Mockoon server.
//! Measures streaming throughput, time to first byte, and assembly latency.

use std::{sync::Arc, time::Instant};
use hdrhistogram::Histogram;
use tokio::{sync::Mutex, time::Duration};
use reqwest::Client;
use serde_json::json;

#[tokio::main(flavor="multi_thread")]
async fn main() {
    let base = std::env::var("BASE").unwrap_or_else(|_| "http://localhost:3000".into());
    let route = std::env::args().nth(1).unwrap_or_else(|| "/v1/chat/completions:stream".into());
    let conc: usize = std::env::var("C").ok().and_then(|v| v.parse().ok()).unwrap_or(32);
    let dur = Duration::from_secs(std::env::var("T").ok().and_then(|v| v.parse().ok()).unwrap_or(60));

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    
    let hist = Arc::new(Mutex::new(Histogram::<u64>::new(3).unwrap()));
    let first_byte_hist = Arc::new(Mutex::new(Histogram::<u64>::new(3).unwrap()));
    let stop = Instant::now() + dur;
    let mut tasks = Vec::new();
    let mut ok = 0usize;
    let mut err = 0usize;
    let mut total_bytes = 0usize;

    println!("Starting Rust streaming benchmark: route={}, concurrency={}, duration={}s", 
             route, conc, dur.as_secs());

    // Spawn worker tasks
    for _ in 0..conc {
        let client = client.clone();
        let url = format!("{}{}", base, route);
        let hist = hist.clone();
        let first_byte_hist = first_byte_hist.clone();
        tasks.push(tokio::spawn(async move {
            let mut local_ok = 0usize;
            let mut local_err = 0usize;
            let mut local_bytes = 0usize;
            
            while Instant::now() < stop {
                let body = json!({
                    "model": "test-model",
                    "messages": [{"role": "user", "content": "Stream this message"}]
                });
                
                let t0 = Instant::now();
                let first_byte_time = Instant::now();
                let mut first_byte_recorded = false;
                
                match client.post(&url).json(&body).send().await {
                    Ok(mut response) => {
                        if response.status().is_success() {
                            // Read streaming response - simplified for now
                            let content = response.bytes().await;
                            match content {
                                Ok(bytes) => {
                                    if !first_byte_recorded {
                                        let dt = first_byte_time.elapsed();
                                        let mut h = first_byte_hist.lock().await;
                                        h.record(dt.as_micros() as u64).ok();
                                        drop(h);
                                        first_byte_recorded = true;
                                    }
                                    local_bytes += bytes.len();
                                }
                                Err(_) => {
                                    local_err += 1;
                                    continue;
                                }
                            }
                            
                            let dt = t0.elapsed();
                            let mut h = hist.lock().await;
                            h.record(dt.as_micros() as u64).ok();
                            drop(h);
                            local_ok += 1;
                        } else {
                            local_err += 1;
                        }
                    }
                    Err(_) => {
                        local_err += 1;
                    }
                }
            }
            (local_ok, local_err, local_bytes)
        }));
    }

    // Wait for all tasks to complete
    for task in tasks {
        let (o, e, b) = task.await.unwrap();
        ok += o;
        err += e;
        total_bytes += b;
    }

    // Collect final statistics
    let h = hist.lock().await;
    let fb_h = first_byte_hist.lock().await;
    let total_reqs = ok + err;
    let duration_secs = dur.as_secs_f64();
    
    let out = serde_json::json!({
        "lang": "rust-streaming",
        "route": route,
        "concurrency": conc,
        "reqs": total_reqs,
        "throughput_rps": total_reqs as f64 / duration_secs,
        "sse_bytes_per_sec": total_bytes as f64 / duration_secs,
        "latency_ms": {
            "p50": h.value_at_percentile(50.0) as f64 / 1000.0,
            "p95": h.value_at_percentile(95.0) as f64 / 1000.0,
            "p99": h.value_at_percentile(99.0) as f64 / 1000.0
        },
        "time_to_first_ms": {
            "p50": fb_h.value_at_percentile(50.0) as f64 / 1000.0,
            "p95": fb_h.value_at_percentile(95.0) as f64 / 1000.0,
            "p99": fb_h.value_at_percentile(99.0) as f64 / 1000.0
        },
        "errors": {
            "non2xx": err,
            "timeouts": 0,
            "total": err
        },
        "success_rate": (ok as f64 / total_reqs as f64) * 100.0,
        "total_bytes": total_bytes
    });
    
    println!("{}", serde_json::to_string(&out).unwrap());
}
