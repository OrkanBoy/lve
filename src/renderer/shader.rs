use ash::vk;
use std::ffi;

pub struct Shader {
    pub module: vk::ShaderModule,
    pub stage_info: vk::PipelineShaderStageCreateInfo
}

impl Shader {
    pub fn new(logical_device: &ash::Device, code: &[u32], stage: vk::ShaderStageFlags, entry_name: &ffi::CStr) -> Self {
        let module_info = vk::ShaderModuleCreateInfo::builder()
            .code(code);

        let module = unsafe {
            logical_device.create_shader_module(&module_info, None).unwrap()
        };

        let stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(stage)    
            .module(module)
            .name(&entry_name)
            .build();
            
        Self {
            module,
            stage_info
        }
    }

    pub unsafe fn cleanup(&self, logical_device: &ash::Device) {
        logical_device.destroy_shader_module(self.module, None);
    }
}