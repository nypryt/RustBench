use crate::BenchmarkResult;

pub trait GpuBackend {
    fn name(&self) -> &str;
    fn device_name(&self) -> String;
    fn run_compute_test(&self, name: &str, iterations: u64) -> BenchmarkResult;
}

pub struct GpuBenchmark<B: GpuBackend> {
    backend: B,
}

impl<B: GpuBackend> GpuBenchmark<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn run_all(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        results.push(self.backend.run_compute_test("Compute Shader (Simple)", 1_000_000));
        results.push(self.backend.run_compute_test("Compute Shader (Complex)", 100_000));
        results.push(self.backend.run_compute_test("Matrix Multiply", 10_000));
        results.push(self.backend.run_compute_test("Particle Simulation", 500_000));
        results
    }
}