# RustBench

Cross-platform CPU/GPU compute benchmark written in Rust with support for:
- **OpenGL** (cross-platform compute shaders)
- **Vulkan** (cross-platform compute pipelines)
- **Metal** (Apple Silicon & Intel Macs)

## Features

- CPU benchmarks: integer math, floating point, memory bandwidth, parallel scaling (Rayon)
- GPU benchmarks: compute shaders, matrix multiply, particle simulation
- Multiple backend support with automatic detection
- JSON results output
- Headless operation (no visible window required)

## Building

```bash
# Default (OpenGL + Vulkan)
cargo build --release

# With Metal support (macOS only)
cargo build --release --features metal

# Specific backends
cargo build --release --features opengl
cargo build --release --features vulkan
```

## Running

```bash
# Run all benchmarks
./target/release/rustbench

# Results saved to rustbench_results.json
```

## Requirements

- **Windows**: Visual Studio Build Tools, Vulkan SDK (for Vulkan backend)
- **Linux**: mesa-vulkan-drivers, libglvnd, vulkan headers
- **macOS**: Xcode Command Line Tools (Metal built-in)

## Example Output

```
================================================================================
                              RUSTBENCH RESULTS
================================================================================
Test                           Backend      Device              Time (ms)      Score
--------------------------------------------------------------------------------
Integer Math                   CPU          AMD Ryzen 9 7950X        45.23     2210.5
Floating Point                 CPU          AMD Ryzen 9 7950X       123.45      405.2
Memory Bandwidth               CPU          AMD Ryzen 9 7950X       567.89     112.3
Parallel Scaling               CPU (Rayon)  AMD Ryzen 9 7950X        89.12     1122.1
Compute Shader (Simple)        OpenGL       NVIDIA RTX 4090         12.34    81037.2
Compute Shader (Complex)       OpenGL       NVIDIA RTX 4090        156.78     637.8
Matrix Multiply                OpenGL       NVIDIA RTX 4090          5.67    17636.4
Particle Simulation            OpenGL       NVIDIA RTX 4090         23.45    21321.5
Compute Shader (Simple)        Vulkan       NVIDIA RTX 4090         10.21    97943.2
Compute Shader (Complex)       Vulkan       NVIDIA RTX 4090        134.56     743.2
Matrix Multiply                Vulkan       NVIDIA RTX 4090          4.89    20449.9
Particle Simulation            Vulkan       NVIDIA RTX 4090         19.87    25163.4
================================================================================
```

## License

MIT