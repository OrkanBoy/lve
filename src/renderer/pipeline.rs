use super::Device;
use super::Shader;

use ash::vk;

pub struct Pipeline {
    pub graphics: vk::Pipeline,
    pub layout: vk::PipelineLayout
}

impl Pipeline {
    pub fn new(device: &Device, extent: vk::Extent2D, render_pass: vk::RenderPass) -> Self {
        //entry_name not shader creation local because p_name of shader modules hold reference
        let entry_name = std::ffi::CString::new("main").unwrap();
        
        let vert_shader = Shader::new(
            &device.logical, 
            vk_shader_macros::include_glsl!("./shaders/foo.vert"),
            vk::ShaderStageFlags::VERTEX, 
            &entry_name);
        
        let frag_shader = Shader::new(
            &device.logical, 
            vk_shader_macros::include_glsl!("./shaders/foo.frag"),
            vk::ShaderStageFlags::FRAGMENT, 
            &entry_name);

        let (graphics, layout) = Self::new_graphics(
            &device.logical,
            render_pass, 
            extent,
            &[vert_shader.stage_info, frag_shader.stage_info]);

        unsafe {
            vert_shader.cleanup(&device.logical);
            frag_shader.cleanup(&device.logical);
        }

        Self {
            graphics,
            layout
        }
    }

    pub fn new_graphics(
        logical_device: &ash::Device,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        shader_stages: &[vk::PipelineShaderStageCreateInfo])
    -> (vk::Pipeline, vk::PipelineLayout) {
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder();
        
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);


        let viewports = [
            vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 0.0
            }
        ];

        let scissors = [
            vk::Rect2D {
                offset: vk::Offset2D {
                    x: 0,
                    y: 0,
                },
                extent
            }
        ];
        
        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .line_width(1.0)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .cull_mode(vk::CullModeFlags::NONE)
            .polygon_mode(vk::PolygonMode::FILL);

        let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachments = [
            vk::PipelineColorBlendAttachmentState::builder()
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .alpha_blend_op(vk::BlendOp::ADD)
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .build()
        ];

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(&color_blend_attachments);

        let layout_info = vk::PipelineLayoutCreateInfo::builder();
        let layout = unsafe {
            logical_device.create_pipeline_layout(&layout_info, None).unwrap()
        };

        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampler_info)
            .color_blend_state(&color_blend_info)
            .layout(layout)
            .render_pass(render_pass)
            .subpass(0);
        
        let pipeline = unsafe {
            logical_device.create_graphics_pipelines(vk::PipelineCache::null(), &[info.build()], None).unwrap()
        }[0];

        (pipeline, layout)
    }

    pub unsafe fn cleanup(&mut self, logical_device: &ash::Device) {
        logical_device.destroy_pipeline(self.graphics, None);
        logical_device.destroy_pipeline_layout(self.layout, None);
    }
}