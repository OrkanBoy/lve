use ash::{vk, extensions::khr};
use anyhow::Result;

pub struct Window {
    pub event_loop: Option<winit::event_loop::EventLoop<()>>,
    pub window: winit::window::Window,

    pub surface: vk::SurfaceKHR,
    pub surface_loader: khr::Surface,
    
    pub format: vk::SurfaceFormatKHR
}

impl Window {
    pub fn new_window() -> (winit::event_loop::EventLoop<()>, winit::window::Window) {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::Window::new(&event_loop).unwrap();

        (event_loop, window)
    }
    pub fn new(event_loop: winit::event_loop::EventLoop<()>,
        window: winit::window::Window,
        physical_device: vk::PhysicalDevice,
        entry: &ash::Entry,
        instance: &ash::Instance) -> Self {
        let surface = unsafe {
            ash_window::create_surface(entry, instance, &window, None).unwrap()
        };
        let surface_loader = khr::Surface::new(entry, instance);

        let format = unsafe {
            surface_loader.get_physical_device_surface_formats(physical_device, surface).unwrap()[0]
        };

        Self {
            event_loop: Some(event_loop),
            window,
            surface,
            surface_loader,
            format
        }
    }

    pub fn surface_capabilities(&self, physical_device: vk::PhysicalDevice) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            self.surface_loader.get_physical_device_surface_capabilities(physical_device, self.surface).unwrap()
        }
    }

    pub fn event_loop(&mut self) -> Result<winit::event_loop::EventLoop<()>> {
        match self.event_loop.take() {
            None => anyhow::bail!("EventLoop was acquired before"),
            Some(el) => Ok(el)
        }
    }

    pub unsafe fn cleanup(&mut self) {
        self.surface_loader.destroy_surface(self.surface, None);
    }
}