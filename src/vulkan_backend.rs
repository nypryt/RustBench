use std::sync::Arc;
use ash::{vk, Entry, Instance, Device};
use winit::window::Window;
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};
use crate::{BenchmarkResult, GpuBackend};

pub struct VulkanBenchmark {
    entry: Entry,
    instance: Instance,
    device: Device,
    physical_device: vk::PhysicalDevice,
    queue: vk::Queue,
    queue_family_index: u32,
    device_name: String,
}

impl VulkanBenchmark {
    pub fn new() -> Result<Self, String> {
        let entry = unsafe { Entry::load().map_err(|e| e.to_string())? };

        let event_loop = winit::event_loop::EventLoop::new().map_err(|e| e.to_string())?;
        let window = winit::window::WindowBuilder::new()
            .with_title("RustBench Vulkan")
            .with_visible(false)
            .build(&event_loop)
            .map_err(|e| e.to_string())?;

        let instance = Self::create_instance(&entry, &window)?;
        let physical_device = Self::pick_physical_device(&instance)?;
        let (device, queue, queue_family_index) = Self::create_logical_device(&instance, physical_device)?;
        let device_name = Self::get_device_name(&instance, physical_device);

        Ok(Self {
            entry,
            instance,
            device,
            physical_device,
            queue,
            queue_family_index,
            device_name,
        })
    }

    fn create_instance(entry: &Entry, window: &Window) -> Result<Instance, String> {
        let app_info = vk::ApplicationInfo::default()
            .application_name(c"RustBench")
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(c"RustBench")
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_3);

        let mut extensions = ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
            .map_err(|e| e.to_string())?
            .to_vec();
        extensions.push(ash::ext::debug_utils::NAME.as_ptr());

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions);

        unsafe { entry.create_instance(&create_info, None).map_err(|e| e.to_string()) }
    }

    fn pick_physical_device(instance: &Instance) -> Result<vk::PhysicalDevice, String> {
        let devices = unsafe { instance.enumerate_physical_devices().map_err(|e| e.to_string())? };
        devices.into_iter()
            .find(|&d| {
                let props = unsafe { instance.get_physical_device_properties(d) };
                props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU ||
                props.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU
            })
            .ok_or("No suitable GPU found".into())
    }

    fn create_logical_device(instance: &Instance, physical_device: vk::PhysicalDevice) -> Result<(Device, vk::Queue, u32), String> {
        let queue_props = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let queue_family_index = queue_props.iter().position(|q| q.queue_flags.contains(vk::QueueFlags::COMPUTE))
            .ok_or("No compute queue family")? as u32;

        let queue_priorities = [1.0f32];
        let queue_create_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities);

        let features = vk::PhysicalDeviceFeatures::default()
            .shader_storage_buffer_array_dynamic_indexing(true);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_create_info))
            .enabled_features(&features);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None).map_err(|e| e.to_string())? };
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok((device, queue, queue_family_index))
    }

    fn get_device_name(instance: &Instance, physical_device: vk::PhysicalDevice) -> String {
        let props = unsafe { instance.get_physical_device_properties(physical_device) };
        unsafe { std::ffi::CStr::from_ptr(props.device_name.as_ptr()) }
            .to_string_lossy()
            .to_string()
    }

    fn create_compute_pipeline(&self, shader_spirv: &[u32]) -> Result<(vk::Pipeline, vk::PipelineLayout), String> {
        let shader_module = unsafe {
            let create_info = vk::ShaderModuleCreateInfo::default().code(shader_spirv);
            self.device.create_shader_module(&create_info, None).map_err(|e| e.to_string())?
        };

        let layout = unsafe {
            let create_info = vk::PipelineLayoutCreateInfo::default();
            self.device.create_pipeline_layout(&create_info, None).map_err(|e| e.to_string())?
        };

        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module)
            .name(c"main");

        let pipeline = unsafe {
            let create_info = vk::ComputePipelineCreateInfo::default()
                .stage(stage)
                .layout(layout);
            self.device.create_compute_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&create_info), None)
                .map_err(|e| e.1.to_string())?[0]
        };

        unsafe { self.device.destroy_shader_module(shader_module, None); }
        Ok((pipeline, layout))
    }

    fn run_compute(&self, pipeline: vk::Pipeline, layout: vk::PipelineLayout, work_groups: [u32; 3]) -> f64 {
        let cmd_pool = unsafe {
            let create_info = vk::CommandPoolCreateInfo::default()
                .queue_family_index(self.queue_family_index)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
            self.device.create_command_pool(&create_info, None).unwrap()
        };

        let cmd_buffer = unsafe {
            let alloc_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(cmd_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            self.device.allocate_command_buffers(&alloc_info).unwrap()[0]
        };

        unsafe {
            self.device.begin_command_buffer(cmd_buffer, &vk::CommandBufferBeginInfo::default()).unwrap();
            self.device.cmd_bind_pipeline(cmd_buffer, vk::PipelineBindPoint::COMPUTE, pipeline);
            self.device.cmd_dispatch(cmd_buffer, work_groups[0], work_groups[1], work_groups[2]);
            self.device.end_command_buffer(cmd_buffer).unwrap();

            let submit_info = vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&cmd_buffer));
            let fence = self.device.create_fence(&vk::FenceCreateInfo::default(), None).unwrap();

            let start = std::time::Instant::now();
            self.device.queue_submit(self.queue, std::slice::from_ref(&submit_info), fence).unwrap();
            self.device.wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX).unwrap();
            let duration = start.elapsed().as_secs_f64() * 1000.0;

            self.device.destroy_fence(fence, None);
            self.device.free_command_buffers(cmd_pool, &[cmd_buffer]);
            self.device.destroy_command_pool(cmd_pool, None);
            duration
        }
    }
}

impl GpuBackend for VulkanBenchmark {
    fn name(&self) -> &str {
        "Vulkan"
    }

    fn device_name(&self) -> String {
        self.device_name.clone()
    }

    fn run_compute_test(&self, name: &str, iterations: u64) -> BenchmarkResult {
        let (shader_spirv, work_groups) = match name {
            "Compute Shader (Simple)" => (SIMPLE_COMPUTE_SPIRV, [iterations as u32 / 256, 1, 1]),
            "Compute Shader (Complex)" => (COMPLEX_COMPUTE_SPIRV, [iterations as u32 / 64, 1, 1]),
            "Matrix Multiply" => (MATRIX_MULTIPLY_SPIRV, [16, 16, 1]),
            "Particle Simulation" => (PARTICLE_SIM_SPIRV, [iterations as u32 / 256, 1, 1]),
            _ => (SIMPLE_COMPUTE_SPIRV, [iterations as u32 / 256, 1, 1]),
        };

        let (pipeline, layout) = self.create_compute_pipeline(shader_spirv).unwrap();
        let duration = self.run_compute(pipeline, layout, work_groups);

        let score = match name {
            "Compute Shader (Simple)" => (iterations as f64 / duration) / 1_000_000.0,
            "Compute Shader (Complex)" => (iterations as f64 / duration) / 100_000.0,
            "Matrix Multiply" => 1000.0 / duration,
            "Particle Simulation" => (iterations as f64 / duration) / 1_000_000.0,
            _ => 0.0,
        };

        unsafe {
            self.device.destroy_pipeline(pipeline, None);
            self.device.destroy_pipeline_layout(layout, None);
        }

        BenchmarkResult {
            name: name.into(),
            backend: "Vulkan".into(),
            device: self.device_name(),
            duration_ms: duration,
            score,
            iterations,
        }
    }
}

impl Drop for VulkanBenchmark {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

// Pre-compiled SPIR-V shaders (minimal examples - in practice use shaderc to compile at build time)
const SIMPLE_COMPUTE_SPIRV: &[u32] = &[
    0x07230203, 0x00010000, 0x000a0004, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000001,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x0000000b, 0x00000000, 0x00000001, 0x00000000, 0x00000004, 0x00000000, 0x00000000, 0x00000000,
];

const COMPLEX_COMPUTE_SPIRV: &[u32] = &[
    0x07230203, 0x00010000, 0x000a0004, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000001,
];

const MATRIX_MULTIPLY_SPIRV: &[u32] = &[
    0x07230203, 0x00010000, 0x000a0004, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000001,
];

const PARTICLE_SIM_SPIRV: &[u32] = &[
    0x07230203, 0x00010000, 0x000a0004, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000001,
];