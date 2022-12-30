use ash::{vk, extensions::khr};
use super::Device;
use super::Window;

pub struct Swapchain {
    pub loader: khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,

    pub image_views: Vec<vk::ImageView>,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub extent: vk::Extent2D,

    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub start_draw_fences: Vec<vk::Fence>,
    pub image_count: usize,
    pub current_image: usize
}

impl Swapchain {
    pub fn new(instance: &ash::Instance, device: &Device, window: &Window, render_pass: vk::RenderPass) -> Self {
        let capabilities = window.surface_capabilities(device.physical);

        let (loader, swapchain) = Self::new_swapchain(instance, device, window, &capabilities);

        let images = unsafe {
            loader.get_swapchain_images(swapchain).unwrap()
        };
        let image_views = Self::new_image_views(&images, &device.logical);
        let image_count = image_views.len();

        let (image_available_semaphores,
            render_finished_semaphores,
            start_draw_fences) = Self::new_syncs(image_count, &device.logical);

        let extent = capabilities.current_extent;

        let framebuffers = Self::new_framebuffers(&image_views, &device.logical, extent, render_pass);
        
        Self {
            loader,
            swapchain,
            image_views,
            framebuffers,
            extent,
            image_available_semaphores,
            render_finished_semaphores,
            start_draw_fences,
            image_count,
            current_image: 0
        }
    }

    fn new_swapchain(instance: &ash::Instance,
        device: &Device, 
        window: &Window,
        capabilities: &vk::SurfaceCapabilitiesKHR)
    -> (khr::Swapchain, vk::SwapchainKHR) {

        let queue_family_indices = [device.graphics_family.index];
        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(window.surface)
            .min_image_count(3.max(capabilities.min_image_count).min(capabilities.max_image_count))
            .image_format(window.format.format)
            .image_color_space(window.format.color_space)
            .image_extent(capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO);

        let loader = khr::Swapchain::new(instance, &device.logical);
        let swapchain = unsafe {
            loader.create_swapchain(&info, None).unwrap()
        };
        (loader, swapchain)
    }

    fn new_image_views(images: &Vec<vk::Image>, logical: &ash::Device) -> Vec<vk::ImageView> {
        let mut image_views = Vec::with_capacity(images.len());
        for image in images {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::B8G8R8A8_UNORM)
                .subresource_range(*subresource_range);

            image_views.push(unsafe {
                logical.create_image_view(&info, None).unwrap()
            });
        }

        image_views
    }

    fn new_framebuffers(image_views: &Vec<vk::ImageView>, logical: &ash::Device, extent: vk::Extent2D, render_pass: vk::RenderPass) -> Vec<vk::Framebuffer> {
        let mut framebuffers = Vec::with_capacity(image_views.len());
        for &image_view in image_views {
            let attachments = [image_view];
            let info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);

            framebuffers.push(unsafe {
                logical.create_framebuffer(&info, None).unwrap()
            });
        }
        framebuffers
    }

    fn new_syncs(image_count: usize, logical: &ash::Device) -> 
        (Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>) {
        
        let semaphore_info = vk::SemaphoreCreateInfo::builder();

        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available_semaphores = Vec::with_capacity(image_count);
        let mut render_finished_semaphores = Vec::with_capacity(image_count);
        let mut start_draw_fences = Vec::with_capacity(image_count);

        for _ in 0..image_count {
            image_available_semaphores.push(unsafe {
                logical.create_semaphore(&semaphore_info, None).unwrap()
            });
            render_finished_semaphores.push(unsafe {
                logical.create_semaphore(&semaphore_info, None).unwrap()
            });
            start_draw_fences.push(unsafe {
                logical.create_fence(&fence_info, None).unwrap()
            });
        }
        (image_available_semaphores,
        render_finished_semaphores,
        start_draw_fences)
    }
    
    pub unsafe fn cleanup(&mut self, logical: &ash::Device) {
        for i in 0..self.image_count {
            logical.destroy_semaphore(self.image_available_semaphores[i], None);
            logical.destroy_semaphore(self.render_finished_semaphores[i], None);
            logical.destroy_fence(self.start_draw_fences[i], None);
            
            logical.destroy_framebuffer(self.framebuffers[i], None);
            logical.destroy_image_view(self.image_views[i], None);
        }
        self.loader.destroy_swapchain(self.swapchain, None);
    } 
}