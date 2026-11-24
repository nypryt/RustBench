#[cfg(all(target_os = "macos", feature = "metal"))]
use metal::{Device as MetalDevice, MTLComputePipelineState, MTLCommandQueue, MTLBuffer};
#[cfg(all(target_os = "macos", feature = "metal"))]
use objc2_foundation::{NSString, NSUInteger};
#[cfg(all(target_os = "macos", feature = "metal"))]
use crate::{BenchmarkResult, GpuBackend};

#[cfg(all(target_os = "macos", feature = "metal"))]
pub struct MetalBenchmark {
    device: MetalDevice,
    command_queue: MTLCommandQueue,
    device_name: String,
}

#[cfg(all(target_os = "macos", feature = "metal"))]
impl MetalBenchmark {
    pub fn new() -> Result<Self, String> {
        let device = MetalDevice::system_default().ok_or("No Metal device found")?;
        let command_queue = device.new_command_queue();
        let device_name = device.name().to_string();
        Ok(Self { device, command_queue, device_name })
    }

    fn create_pipeline(&self, shader_source: &str) -> Result<MTLComputePipelineState, String> {
        let library = self.device.new_library_with_source(shader_source, &metal::CompileOptions::new())
            .map_err(|e| format!("Library creation failed: {:?}", e))?;
        let function = library.get_function("compute_main", None)
            .map_err(|e| format!("Function not found: {:?}", e))?;
        self.device.new_compute_pipeline_state_with_function(&function)
            .map_err(|e| format!("Pipeline creation failed: {:?}", e))
    }

    fn run_compute(&self, pipeline: &MTLComputePipelineState, threadgroups: MTLSize, threads_per_threadgroup: MTLSize) -> f64 {
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.compute_command_encoder();
        encoder.set_compute_pipeline_state(pipeline);
        encoder.dispatch_thread_groups(threadgroups, threads_per_threadgroup);
        encoder.end_encoding();

        let start = std::time::Instant::now();
        command_buffer.commit();
        command_buffer.wait_until_completed();
        start.elapsed().as_secs_f64() * 1000.0
    }
}

#[cfg(all(target_os = "macos", feature = "metal"))]
impl GpuBackend for MetalBenchmark {
    fn name(&self) -> &str {
        "Metal"
    }

    fn device_name(&self) -> String {
        self.device_name.clone()
    }

    fn run_compute_test(&self, name: &str, iterations: u64) -> BenchmarkResult {
        let (shader_src, threadgroups, threads_per_threadgroup) = match name {
            "Compute Shader (Simple)" => (SIMPLE_METAL, MTLSize::new(iterations as u64 / 256, 1, 1), MTLSize::new(256, 1, 1)),
            "Compute Shader (Complex)" => (COMPLEX_METAL, MTLSize::new(iterations as u64 / 64, 1, 1), MTLSize::new(64, 1, 1)),
            "Matrix Multiply" => (MATRIX_METAL, MTLSize::new(16, 16, 1), MTLSize::new(16, 16, 1)),
            "Particle Simulation" => (PARTICLE_METAL, MTLSize::new(iterations as u64 / 256, 1, 1), MTLSize::new(256, 1, 1)),
            _ => (SIMPLE_METAL, MTLSize::new(iterations as u64 / 256, 1, 1), MTLSize::new(256, 1, 1)),
        };

        let pipeline = self.create_pipeline(shader_src).unwrap();
        let duration = self.run_compute(&pipeline, threadgroups, threads_per_threadgroup);

        let score = match name {
            "Compute Shader (Simple)" => (iterations as f64 / duration) / 1_000_000.0,
            "Compute Shader (Complex)" => (iterations as f64 / duration) / 100_000.0,
            "Matrix Multiply" => 1000.0 / duration,
            "Particle Simulation" => (iterations as f64 / duration) / 1_000_000.0,
            _ => 0.0,
        };

        BenchmarkResult {
            name: name.into(),
            backend: "Metal".into(),
            device: self.device_name(),
            duration_ms: duration,
            score,
            iterations,
        }
    }
}

#[cfg(all(target_os = "macos", feature = "metal"))]
const SIMPLE_METAL: &str = r#"
#include <metal_stdlib>
using namespace metal;

kernel void compute_main(device uint* data [[buffer(0)]], uint tid [[thread_position_in_grid]]) {
    uint v = data[tid];
    v = v * 1664525u + 1013904223u;
    v = v ^ (v >> 16);
    v = v * 1664525u + 1013904223u;
    data[tid] = v;
}
"#;

#[cfg(all(target_os = "macos", feature = "metal"))]
const COMPLEX_METAL: &str = r#"
#include <metal_stdlib>
using namespace metal;

kernel void compute_main(device float* data [[buffer(0)]], uint tid [[thread_position_in_grid]]) {
    float x = data[tid];
    for (int i = 0; i < 100; i++) {
        x = sin(x) * cos(x) + tan(x) * 0.01;
        x = sqrt(fabs(x)) + 0.1;
    }
    data[tid] = x;
}
"#;

#[cfg(all(target_os = "macos", feature = "metal"))]
const MATRIX_METAL: &str = r#"
#include <metal_stdlib>
using namespace metal;

kernel void compute_main(device float4x4* a [[buffer(0)]], device float4x4* b [[buffer(1)]], device float4x4* c [[buffer(2)]], uint2 tid [[thread_position_in_grid]]) {
    float4x4 result = float4x4(0);
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            for (int k = 0; k < 4; k++) {
                result[i][j] += a[0][i][k] * b[0][k][j];
            }
        }
    }
    c[0] = result;
}
"#;

#[cfg(all(target_os = "macos", feature = "metal"))]
const PARTICLE_METAL: &str = r#"
#include <metal_stdlib>
using namespace metal;

kernel void compute_main(device float4* positions [[buffer(0)]], device float4* velocities [[buffer(1)]], constant float& dt [[buffer(2)]], uint tid [[thread_position_in_grid]]) {
    float4 pos = positions[tid];
    float4 vel = velocities[tid];
    vel.y -= 9.81 * dt;
    pos += vel * dt;
    if (pos.y < 0.0) { pos.y = 0.0; vel.y = -vel.y * 0.8; }
    positions[tid] = pos;
    velocities[tid] = vel;
}
"#;