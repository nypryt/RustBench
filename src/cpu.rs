use std::time::Instant;
use rayon::prelude::*;
use crate::BenchmarkResult;

pub struct CpuBenchmark;

impl CpuBenchmark {
    pub fn new() -> Self {
        Self
    }

    pub fn run_all(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        results.push(self.integer_math());
        results.push(self.floating_point());
        results.push(self.memory_bandwidth());
        results.push(self.parallel_scaling());
        results
    }

    fn integer_math(&self) -> BenchmarkResult {
        const ITERATIONS: u64 = 100_000_000;
        let start = Instant::now();
        let mut sum: u64 = 0;
        for i in 0..ITERATIONS {
            sum = sum.wrapping_add(i).wrapping_mul(31) ^ 0x9e3779b97f4a7c15;
        }
        let duration = start.elapsed().as_secs_f64() * 1000.0;
        let score = (ITERATIONS as f64 / duration) / 1_000_000.0;
        BenchmarkResult {
            name: "Integer Math".into(),
            backend: "CPU".into(),
            device: cpu_name(),
            duration_ms: duration,
            score,
            iterations: ITERATIONS,
        }
    }

    fn floating_point(&self) -> BenchmarkResult {
        const ITERATIONS: u64 = 50_000_000;
        let start = Instant::now();
        let mut _sum: f64 = 0.0;
        for i in 0..ITERATIONS {
            let x = i as f64 * 0.000001;
            _sum += x.sin().cos().tan().sqrt().max(0.0);
        }
        let duration = start.elapsed().as_secs_f64() * 1000.0;
        let score = (ITERATIONS as f64 / duration) / 1_000_000.0;
        BenchmarkResult {
            name: "Floating Point".into(),
            backend: "CPU".into(),
            device: cpu_name(),
            duration_ms: duration,
            score,
            iterations: ITERATIONS,
        }
    }

    fn memory_bandwidth(&self) -> BenchmarkResult {
        const SIZE: usize = 64 * 1024 * 1024; // 64MB
        const ITERATIONS: u64 = 10;
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
        let duration = start.elapsed().as_secs_f64() * 1000.0;
        let gb_processed = (SIZE as f64 * ITERATIONS as f64) / (1024.0 * 1024.0 * 1024.0);
        let score = gb_processed / (duration / 1000.0);
        BenchmarkResult {
            name: "Memory Bandwidth".into(),
            backend: "CPU".into(),
            device: cpu_name(),
            duration_ms: duration,
            score,
            iterations: ITERATIONS,
        }
    }

    fn parallel_scaling(&self) -> BenchmarkResult {
        const SIZE: usize = 10_000_000;
        let data: Vec<f64> = (0..SIZE).map(|i| i as f64 * 0.001).collect();

        let start = Instant::now();
        let sum: f64 = data.par_iter()
            .map(|&x| x.sin().cos().sqrt().max(0.0))
            .sum();
        let duration = start.elapsed().as_secs_f64() * 1000.0;
        std::hint::black_box(sum);

        let score = (SIZE as f64 / duration) / 1_000_000.0;
        BenchmarkResult {
            name: "Parallel Scaling".into(),
            backend: "CPU (Rayon)".into(),
            device: cpu_name(),
            duration_ms: duration,
            score,
            iterations: SIZE as u64,
        }
    }
}

fn cpu_name() -> String {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("wmic")
            .args(["cpu", "get", "name"])
            .output();
        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            return s.lines().nth(1).unwrap_or("Unknown CPU").trim().to_string();
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(s) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in s.lines() {
                if line.starts_with("model name") {
                    return line.split(':').nth(1).unwrap_or("").trim().to_string();
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("sysctl").arg("-n").arg("machdep.cpu.brand_string").output();
        if let Ok(out) = output {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    "Unknown CPU".into()
}