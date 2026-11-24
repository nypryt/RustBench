mod cpu;
mod gpu;
mod opengl_backend;
mod vulkan_backend;
#[cfg(all(target_os = "macos", feature = "metal_backend"))]
mod metal_backend;

use cpu::CpuBenchmark;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub backend: String,
    pub device: String,
    pub duration_ms: f64,
    pub score: f64,
    pub iterations: u64,
}

fn main() {
    env_logger::init();
    log::info!("Starting RustBench v{}", env!("CARGO_PKG_VERSION"));

    let mut results = Vec::new();

    // CPU Benchmarks
    log::info!("Running CPU benchmarks...");
    let cpu_bench = CpuBenchmark::new();
    results.extend(cpu_bench.run_all());

    // GPU Benchmarks (commented out for now - need proper SPIR-V shaders)
    log::info!("Running GPU benchmarks...");
    // #[cfg(feature = "opengl")]
    // {
    //     if let Ok(bench) = opengl_backend::OpenGlBenchmark::new() {
    //         results.extend(bench.run_all());
    //     }
    // }
    // #[cfg(feature = "vulkan")]
    // {
    //     match vulkan_backend::VulkanBenchmark::new() {
    //         Ok(bench) => results.extend(bench.run_all()),
    //         Err(e) => log::warn!("Vulkan backend failed to initialize: {}", e),
    //     }
    // }

    #[cfg(all(target_os = "macos", feature = "metal_backend"))]
    {
        if let Ok(bench) = metal_backend::MetalBenchmark::new() {
            results.extend(bench.run_all());
        }
    }

    // Output results
    print_results(&results);
    save_results(&results);
}

fn print_results(results: &[BenchmarkResult]) {
    println!("\n{}", "=".repeat(80));
    println!("{:^80}", "RUSTBENCH RESULTS");
    println!("{}", "=".repeat(80));
    println!("{:<30} {:<12} {:<15} {:>12} {:>10}", "Test", "Backend", "Device", "Time (ms)", "Score");
    println!("{}", "-".repeat(80));
    for r in results {
        println!("{:<30} {:<12} {:<15} {:>12.2} {:>10.2}", r.name, r.backend, r.device, r.duration_ms, r.score);
    }
    println!("{}", "=".repeat(80));
}

fn save_results(results: &[BenchmarkResult]) {
    let json = serde_json::to_string_pretty(results).unwrap();
    std::fs::write("rustbench_results.json", json).unwrap();
    log::info!("Results saved to rustbench_results.json");
}