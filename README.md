# RustBench

Cross-platform CPU/GPU compute benchmark written in Rust with support for:
- **OpenGL** (cross-platform compute shaders)
- **Vulkan** (cross-platform compute pipelines)
- **Metal** (Apple Silicon & Intel Macs)

## Features

- **CPU Benchmarks**: Integer math, floating point, memory bandwidth, parallel scaling (Rayon)
- **GPU Benchmarks**: Compute shaders, matrix multiply, particle simulation
- **Multiple backend support** with automatic detection
- **JSON results output** for easy parsing and comparison
- **Headless operation** (no visible window required)
- **Cross-platform**: Windows, Linux, macOS

## Building

```bash
# Default (OpenGL + Vulkan)
cargo build --release

# With Metal support (macOS only)
cargo build --release --features metal_backend

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

Pre-built binaries are available in the `BUILDS/` directory.

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

## Project Structure

```
RustBench/
├── src/
│   ├── main.rs              # Entry point, result handling
│   ├── cpu.rs               # CPU benchmarks (Rayon parallel)
│   ├── gpu.rs               # GPU backend trait
│   ├── opengl_backend.rs    # OpenGL compute shaders
│   ├── vulkan_backend.rs    # Vulkan compute pipelines
│   └── metal_backend.rs     # Metal compute (macOS)
├── BUILDS/                  # Pre-built release binaries
├── Cargo.toml
└── README.md
```

## Benchmark Details

### CPU Tests
| Test | Description |
|------|-------------|
| Integer Math | 100M iterations of LCG-style operations |
| Floating Point | 50M iterations of sin/cos/tan/sqrt |
| Memory Bandwidth | 64MB buffer, 10 passes (GB/s) |
| Parallel Scaling | 10M elements, Rayon parallel map/reduce |

### GPU Tests (per backend)
| Test | Workload |
|------|----------|
| Compute Shader (Simple) | 1M threads, basic ALU ops |
| Compute Shader (Complex) | 100K threads, 100 math ops each |
| Matrix Multiply | 16x16 thread blocks, 4x4 matrices |
| Particle Simulation | 500K particles, gravity + bounce |

## JSON Output Format

```json
[
  {
    "name": "Integer Math",
    "backend": "CPU",
    "device": "AMD Ryzen 9 7950X",
    "duration_ms": 45.23,
    "score": 2210.5,
    "iterations": 100000000
  }
]
```

## License

MIT