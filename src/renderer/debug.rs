use ash::{vk, extensions::ext};
use std::ffi;

pub struct Debug {
    loader: ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT
}

impl Debug {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        let loader = ext::DebugUtils::new(entry, instance);

        let messenger_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            pfn_user_callback: Some(vulkan_debug_utils_callback),
            ..Default::default()
        };

        let messenger = unsafe {
            loader.create_debug_utils_messenger(&messenger_info, None).unwrap()
        };

        Self {
            loader,
            messenger
        }
    }

    pub unsafe fn cleanup(&mut self) {
        self.loader.destroy_debug_utils_messenger(self.messenger, None);
    }
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut ffi::c_void,
) -> vk::Bool32 {
    let message = ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();

    println!("[Debug][{}][{}] {:?}", severity, ty, message);

    vk::FALSE
}