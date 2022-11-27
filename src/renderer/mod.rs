pub mod debug;
pub mod device;
pub mod window;
pub mod swapchain;
pub mod pipeline;
pub mod shader;

use debug::Debug;
use device::Device;
use window::Window;
use swapchain::Swapchain;
use pipeline::Pipeline;
use shader::Shader;

use ash::{vk, extensions::*};
use std::{ffi};

pub struct Renderer {
    pub instance: ash::Instance,
    pub debug: Debug,
    pub device: Device,
    pub window: Window,
    pub render_pass: vk::RenderPass,
    pub swapchain: Swapchain,
    pub pipeline: Pipeline,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>
}

impl Renderer {
    pub fn new() -> Self {
        let entry = ash::Entry::linked();
        let (event_loop, window) = Window::new_window();

        let mut extension_names = vec![
            ext::DebugUtils::name().as_ptr(),
            khr::Surface::name().as_ptr()];
        for window_extension in ash_window::enumerate_required_extensions(&window).unwrap() {
            extension_names.push(window_extension.as_ptr());
        }
        
        let layer_names = vec!["VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8];

        let instance = Self::new_instance(&entry, &extension_names, &layer_names);

        let debug = Debug::new(&entry, &instance);

        let device = Device::new(&instance, &layer_names);

        let window = Window::new(event_loop, window, device.physical_device, &entry, &instance);

        let render_pass = Self::new_render_pass(&device, &window);

        let swapchain = Swapchain::new(&instance, &device, &window, render_pass);

        let pipeline = Pipeline::new(&device, swapchain.extent, render_pass);

        let command_pool = Self::new_command_pool(&device);

        let command_buffers = Self::new_command_buffers(&device, &swapchain, &pipeline, command_pool, render_pass);

        Self {
            instance,
            debug,
            device, 
            window,
            render_pass,
            swapchain, 
            pipeline,
            command_pool,
            command_buffers
        }
    }

    fn new_instance(entry: &ash::Entry, extension_names: &Vec<*const i8>, layer_names: &Vec<*const i8>) -> ash::Instance {
        let app_name = ffi::CString::new("Ash App").unwrap();
        let engine_name = ffi::CString::new("Ash Engine").unwrap();

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .engine_name(&engine_name)
            .application_version(vk::make_api_version(0, 0, 0, 1))
            .engine_version(vk::make_api_version(0, 0, 0, 1))
            .api_version(vk::API_VERSION_1_3);

        let info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(extension_names)
            .enabled_layer_names(layer_names);

        unsafe {
            entry.create_instance(&info, None).unwrap()
        }
    }

    fn new_render_pass(device: &Device, window: &Window) -> vk::RenderPass {
        let attachments = [
            vk::AttachmentDescription::builder()
                .format(window.format.format)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .samples(vk::SampleCountFlags::TYPE_1)
                .build()
        ];

        let color_attachment_references = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let subpasses = [
            vk::SubpassDescription::builder()
                .color_attachments(&color_attachment_references)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .build()
        ];

        let subpass_dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_subpass(0)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .build()
        ];

        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        unsafe {
            device.logical_device.create_render_pass(&info, None).unwrap()
        }
    }

    fn new_command_pool(device: &Device) -> vk::CommandPool {
        let info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(device.graphics_family.index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        unsafe {
            device.logical_device.create_command_pool(&info, None).unwrap()
        }
    }

    fn new_command_buffers(
        device: &Device, 
        swapchain: &Swapchain,
        pipeline: &Pipeline,
        pool: vk::CommandPool, 
        render_pass: vk::RenderPass)
    -> Vec<vk::CommandBuffer> {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .command_buffer_count(swapchain.image_count as u32);

        let command_buffers = unsafe {
            device.logical_device.allocate_command_buffers(&alloc_info).unwrap()
        };

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let begin_info = vk::CommandBufferBeginInfo::builder();

            unsafe {
                device.logical_device.begin_command_buffer(command_buffer, &begin_info).unwrap();
            }

            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    }
                },
            ];

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(render_pass)
                .framebuffer(swapchain.framebuffers[i])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: swapchain.extent,
                })
                .clear_values(&clear_values);

            unsafe {
                device.logical_device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE);

                device.logical_device.cmd_bind_pipeline(
                    command_buffer, 
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.pipeline
                );

                device.logical_device.cmd_draw(command_buffer, 3, 1, 0, 0);

                device.logical_device.cmd_end_render_pass(command_buffer);

                device.logical_device.end_command_buffer(command_buffer).unwrap();
            }
        }
        command_buffers
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device.logical_device.device_wait_idle().unwrap();


            self.device.logical_device.destroy_command_pool(self.command_pool, None);

            self.device.logical_device.destroy_render_pass(self.render_pass, None);

            self.pipeline.cleanup(&self.device.logical_device);

            self.swapchain.cleanup(&self.device.logical_device);

            self.window.cleanup();

            self.device.cleanup();

            self.debug.cleanup();

            self.instance.destroy_instance(None);
        }
    }
}