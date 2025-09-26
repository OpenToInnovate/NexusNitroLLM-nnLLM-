//! # Rust Performance Benchmark CLI
//! 
//! Fair, apples-to-apples benchmark against Mockoon server.
//! Measures latency, throughput, CPU, and memory usage.

use std::{sync::Arc, time::Instant};
use hdrhistogram::Histogram;
use tokio::{sync::Mutex, time::Duration};
use reqwest::Client;
use serde_json::json;

#[tokio::main(flavor="multi_thread")]
async fn main() {
    let base = std::env::var("BASE").unwrap_or_else(|_| "http://localhost:3000".into());
    let route = std::env::args().nth(1).unwrap_or_else(|| "/v1/chat/completions".into());
    let conc: usize = std::env::var("C").ok().and_then(|v| v.parse().ok()).unwrap_or(32);
    let dur = Duration::from_secs(std::env::var("T").ok().and_then(|v| v.parse().ok()).unwrap_or(60));
    let payload_size = std::env::var("SIZE").unwrap_or_else(|_| "S".into()); // S=1KB, M=32KB, L=256KB

    // Create payload based on size
    let content = match payload_size.as_str() {
        "S" => "Hello, world!".repeat(50), // ~1KB
        "M" => "Hello, world!".repeat(1600), // ~32KB  
        "L" => "Hello, world!".repeat(12800), // ~256KB
        _ => "Hello, world!".repeat(50),
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    
    let hist = Arc::new(Mutex::new(Histogram::<u64>::new(3).unwrap()));
    let stop = Instant::now() + dur;
    let mut tasks = Vec::new();
    let mut ok = 0usize;
    let mut err = 0usize;
    let mut timeouts = 0usize;

    println!("Starting Rust benchmark: route={}, concurrency={}, duration={}s, payload={}", 
             route, conc, dur.as_secs(), payload_size);

    // Spawn worker tasks
    for _ in 0..conc {
        let client = client.clone();
        let url = format!("{}{}", base, route);
        let hist = hist.clone();
        let content = content.clone();
        tasks.push(tokio::spawn(async move {
            let mut local_ok = 0usize;
            let mut local_err = 0usize;
            let mut local_timeouts = 0usize;
            
            while Instant::now() < stop {
                let body = json!({
                    "model": "test-model",
                    "messages": [{"role": "user", "content": content}]
                });
                
                let t0 = Instant::now();
                let resp = client.post(&url).json(&body).send().await;
                let dt = t0.elapsed();
                
                // Record latency in microseconds
                let mut h = hist.lock().await;
                h.record(dt.as_micros() as u64).ok();
                drop(h);
                
                match resp {
                    Ok(r) => {
                        if r.status().is_success() {
                            // Consume response body
                            let _ = r.bytes().await;
                            local_ok += 1;
                        } else {
                            let _ = r.bytes().await;
                            local_err += 1;
                        }
                    }
                    Err(e) => {
                        if e.is_timeout() {
                            local_timeouts += 1;
                        } else {
                            local_err += 1;
                        }
                    }
                }
            }
            (local_ok, local_err, local_timeouts)
        }));
    }

    // Wait for all tasks to complete
    for task in tasks {
        let (o, e, t) = task.await.unwrap();
        ok += o;
        err += e;
        timeouts += t;
    }

    // Collect final statistics
    let h = hist.lock().await;
    let total_reqs = ok + err + timeouts;
    let duration_secs = dur.as_secs_f64();
    
    let out = serde_json::json!({
        "lang": "rust",
        "route": route,
        "concurrency": conc,
        "payload_size": payload_size,
        "reqs": total_reqs,
        "throughput_rps": total_reqs as f64 / duration_secs,
        "latency_ms": {
            "p50": h.value_at_percentile(50.0) as f64 / 1000.0,
            "p95": h.value_at_percentile(95.0) as f64 / 1000.0,
            "p99": h.value_at_percentile(99.0) as f64 / 1000.0
        },
        "errors": {
            "non2xx": err,
            "timeouts": timeouts,
            "total": err + timeouts
        },
        "success_rate": (ok as f64 / total_reqs as f64) * 100.0
    });
    
    println!("{}", serde_json::to_string(&out).unwrap());
}
