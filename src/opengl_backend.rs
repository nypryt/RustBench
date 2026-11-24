use std::rc::Rc;
use glow::{Context, HasContext};
use winit::window::Window;
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};
use crate::{BenchmarkResult, GpuBackend};

pub struct OpenGlBenchmark {
    gl: Rc<Context>,
    device_name: String,
}

impl OpenGlBenchmark {
    pub fn new() -> Result<Self, String> {
        let event_loop = winit::event_loop::EventLoop::new().map_err(|e| e.to_string())?;
        let window = winit::window::WindowBuilder::new()
            .with_title("RustBench OpenGL")
            .with_visible(false)
            .build(&event_loop)
            .map_err(|e| e.to_string())?;

        let window_handle = window.window_handle().map_err(|e| e.to_string())?.as_raw();
        let display_handle = window.display_handle().map_err(|e| e.to_string())?.as_raw();

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|s| {
                let s = std::ffi::CString::new(s.as_bytes()).unwrap();
                winit::platform::windows::WindowExtWindows::get_proc_address(&window, s.as_ptr()) as *const _
            })
        };

        let device_name = Self::get_device_name(&gl);

        Ok(Self {
            gl: Rc::new(gl),
            device_name,
        })
    }

    fn get_device_name(gl: &Context) -> String {
        unsafe {
            let renderer = gl.get_parameter_string(glow::RENDERER);
            let vendor = gl.get_parameter_string(glow::VENDOR);
            format!("{} {}", vendor.unwrap_or_default(), renderer.unwrap_or_default())
        }
    }

    fn create_compute_shader(&self, src: &str) -> Result<glow::NativeProgram, String> {
        unsafe {
            let program = self.gl.create_program().map_err(|e| e.to_string())?;
            let shader = self.gl.create_shader(glow::COMPUTE_SHADER).map_err(|e| e.to_string())?;
            self.gl.shader_source(shader, src);
            self.gl.compile_shader(shader);
            if !self.gl.get_shader_compile_status(shader) {
                let log = self.gl.get_shader_info_log(shader);
                self.gl.delete_shader(shader);
                return Err(format!("Compute shader compile error: {}", log));
            }
            self.gl.attach_shader(program, shader);
            self.gl.link_program(program);
            if !self.gl.get_program_link_status(program) {
                let log = self.gl.get_program_info_log(program);
                self.gl.delete_program(program);
                return Err(format!("Program link error: {}", log));
            }
            self.gl.delete_shader(shader);
            Ok(program)
        }
    }

    fn run_shader(&self, program: glow::NativeProgram, work_groups: [u32; 3]) -> f64 {
        unsafe {
            self.gl.use_program(Some(program));
            let start = std::time::Instant::now();
            self.gl.dispatch_compute(work_groups[0], work_groups[1], work_groups[2]);
            self.gl.memory_barrier(glow::SHADER_STORAGE_BARRIER_BIT);
            self.gl.finish();
            start.elapsed().as_secs_f64() * 1000.0
        }
    }
}

impl GpuBackend for OpenGlBenchmark {
    fn name(&self) -> &str {
        "OpenGL"
    }

    fn device_name(&self) -> String {
        self.device_name.clone()
    }

    fn run_compute_test(&self, name: &str, iterations: u64) -> BenchmarkResult {
        let (shader_src, work_groups) = match name {
            "Compute Shader (Simple)" => (SIMPLE_COMPUTE, [iterations as u32 / 256, 1, 1]),
            "Compute Shader (Complex)" => (COMPLEX_COMPUTE, [iterations as u32 / 64, 1, 1]),
            "Matrix Multiply" => (MATRIX_MULTIPLY, [16, 16, 1]),
            "Particle Simulation" => (PARTICLE_SIM, [iterations as u32 / 256, 1, 1]),
            _ => (SIMPLE_COMPUTE, [iterations as u32 / 256, 1, 1]),
        };

        let program = self.create_compute_shader(shader_src).unwrap();
        let duration = self.run_shader(program, work_groups);

        let score = match name {
            "Compute Shader (Simple)" => (iterations as f64 / duration) / 1_000_000.0,
            "Compute Shader (Complex)" => (iterations as f64 / duration) / 100_000.0,
            "Matrix Multiply" => 1000.0 / duration,
            "Particle Simulation" => (iterations as f64 / duration) / 1_000_000.0,
            _ => 0.0,
        };

        unsafe {
            self.gl.delete_program(program);
        }

        BenchmarkResult {
            name: name.into(),
            backend: "OpenGL".into(),
            device: self.device_name(),
            duration_ms: duration,
            score,
            iterations,
        }
    }
}

const SIMPLE_COMPUTE: &str = r#"
#version 450
layout(local_size_x = 256) in;
layout(std430, binding = 0) buffer Data { uint values[]; };
void main() {
    uint idx = gl_GlobalInvocationID.x;
    uint v = values[idx];
    v = v * 1664525u + 1013904223u;
    v = v ^ (v >> 16);
    v = v * 1664525u + 1013904223u;
    values[idx] = v;
}
"#;

const COMPLEX_COMPUTE: &str = r#"
#version 450
layout(local_size_x = 64) in;
layout(std430, binding = 0) buffer Data { float values[]; };
void main() {
    uint idx = gl_GlobalInvocationID.x;
    float x = values[idx];
    for (int i = 0; i < 100; i++) {
        x = sin(x) * cos(x) + tan(x) * 0.01;
        x = sqrt(abs(x)) + 0.1;
    }
    values[idx] = x;
}
"#;

const MATRIX_MULTIPLY: &str = r#"
#version 450
layout(local_size_x = 16, local_size_y = 16) in;
layout(std430, binding = 0) buffer A { mat4 mats_a[]; };
layout(std430, binding = 1) buffer B { mat4 mats_b[]; };
layout(std430, binding = 2) buffer C { mat4 mats_c[]; };
void main() {
    ivec2 idx = ivec2(gl_GlobalInvocationID.xy);
    mat4 a = mats_a[0];
    mat4 b = mats_b[0];
    mat4 result = mat4(0.0);
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            for (int k = 0; k < 4; k++) {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    mats_c[0] = result;
}
"#;

const PARTICLE_SIM: &str = r#"
#version 450
layout(local_size_x = 256) in;
layout(std430, binding = 0) buffer Particles { vec4 positions[]; vec4 velocities[]; };
uniform float dt;
void main() {
    uint idx = gl_GlobalInvocationID.x;
    vec4 pos = positions[idx];
    vec4 vel = velocities[idx];
    vel.y -= 9.81 * dt;
    pos += vel * dt;
    if (pos.y < 0.0) { pos.y = 0.0; vel.y = -vel.y * 0.8; }
    positions[idx] = pos;
    velocities[idx] = vel;
}
"#;