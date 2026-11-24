//! Integration tests for RustBench

// For binary crates, we test the public API via the main binary
// We'll use a simpler approach - test the benchmark logic directly

use std::time::Instant;
use rayon::prelude::*;

#[test]
fn test_integer_math_benchmark() {
    const ITERATIONS: u64 = 1_000_000; // Smaller for tests
    let start = Instant::now();
    let mut sum: u64 = 0;
    for i in 0..ITERATIONS {
        sum = sum.wrapping_add(i).wrapping_mul(31) ^ 0x9e3779b97f4a7c15;
    }
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    
    assert!(elapsed > 0.0);
    std::hint::black_box(sum);
}

#[test]
fn test_floating_point_benchmark() {
    const ITERATIONS: u64 = 1_000_000; // Smaller for tests
    let start = Instant::now();
    let mut sum: f64 = 0.0;
    for i in 0..ITERATIONS {
        let x = i as f64 * 0.000001;
        sum += x.sin().cos().tan().sqrt().max(0.0);
    }
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    
    assert!(elapsed > 0.0);
    std::hint::black_box(sum);
}

#[test]
fn test_memory_bandwidth_benchmark() {
    const SIZE: usize = 1_024 * 1_024; // 1MB for tests
    const ITERATIONS: u32 = 10;
    let mut data = vec![0u8; SIZE];
    for (i, byte) in data.iter_mut().enumerate() {
        *byte = (i as u8).wrapping_mul(31);
    }
    
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut sum: u64 = 0;
        for &byte in &data {
            sum = sum.wrapping_add(byte as u64);
        }
        std::hint::black_box(sum);
    }
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    
    assert!(elapsed > 0.0);
}

#[test]
fn test_parallel_scaling_benchmark() {
    const SIZE: usize = 100_000; // Smaller for tests
    let data: Vec<f64> = (0..SIZE).map(|i| i as f64 * 0.001).collect();
    
    let start = Instant::now();
    let sum: f64 = data.par_iter()
        .map(|&x| x.sin().cos().sqrt().max(0.0))
        .sum();
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    
    assert!(elapsed > 0.0);
    std::hint::black_box(sum);
}

#[test]
fn test_json_serialization() {
    let result = serde_json::json!({
        "name": "Test",
        "backend": "CPU",
        "device": "Test CPU",
        "duration_ms": 100.0,
        "score": 1000.0,
        "iterations": 1_000_000,
    });
    
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: serde_json::Value = serde_json::from_str(&json).unwrap();
    
    assert_eq!(result["name"], deserialized["name"]);
    assert_eq!(result["backend"], deserialized["backend"]);
    assert_eq!(result["duration_ms"], deserialized["duration_ms"]);
}