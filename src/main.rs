mod renderer;
use ash::vk;
use winit::event::{Event, WindowEvent};

fn main() {
    let mut renderer = renderer::Renderer::new();

    let event_loop = renderer.window.event_loop().unwrap();

    let graphics_queue = renderer.device.graphics_family.queues[0];

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = winit::event_loop::ControlFlow::Exit;
            },
            Event::RedrawRequested(_) => {
                // acquiring next image:
                renderer.swapchain.current_image = (renderer.swapchain.current_image + 1) % renderer.swapchain.image_count as usize;

                let (image_index, renew_swapchain) = unsafe {
                    renderer.swapchain.loader.acquire_next_image(
                        renderer.swapchain.swapchain,
                        u64::MAX,
                        renderer.swapchain.image_available_semaphores[renderer.swapchain.current_image],
                        vk::Fence::null(),
                    ).unwrap()
                };
                let image_index = image_index as usize;

                // fences:
                unsafe {
                    let fences = [renderer.swapchain.start_draw_fences[renderer.swapchain.current_image]];

                    renderer.device.logical_device.wait_for_fences(
                        &fences,
                        true,
                        u64::MAX,
                    ).unwrap();

                    renderer.device.logical_device.reset_fences(
                        &fences,
                    ).unwrap();
                };

                // submit:
                let semaphores_available = [renderer.swapchain.image_available_semaphores[renderer.swapchain.current_image]];
                let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                let semaphores_finished = [renderer.swapchain.render_finished_semaphores[renderer.swapchain.current_image]];
                let command_buffers = [renderer.command_buffers[image_index]];

                let submit_info = [
                    vk::SubmitInfo::builder()
                        .wait_semaphores(&semaphores_available)
                        .wait_dst_stage_mask(&waiting_stages)
                        .command_buffers(&command_buffers)
                        .signal_semaphores(&semaphores_finished)
                        .build()
                ];

                unsafe {
                    renderer.device.logical_device.queue_submit(
                        graphics_queue,
                        &submit_info,
                        renderer.swapchain.start_draw_fences[renderer.swapchain.current_image],
                    ).unwrap();
                };

                // present:
                let swapchains = [renderer.swapchain.swapchain];
                let indices = [image_index as u32];

                let present_info = vk::PresentInfoKHR::builder()
                    .wait_semaphores(&semaphores_finished)
                    .swapchains(&swapchains)
                    .image_indices(&indices);

                unsafe {
                    renderer.swapchain.loader
                        .queue_present(graphics_queue, &present_info)
                        .unwrap();
                };
            },
            _ => {}
        }
    });
}
