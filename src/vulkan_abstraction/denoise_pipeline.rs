use std::{ffi::CStr, rc::Rc};
use ash::vk;
use crate::error::SrResult;
use crate::vulkan_abstraction::{self, Core};

const SHADER_ENTRY_POINT: &CStr = c"main";

pub struct DenoisePipeline {
    core: Rc<Core>,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
}

impl DenoisePipeline {
    pub fn new(
        core: Rc<Core>,
        descriptor_set_layout: &vulkan_abstraction::DenoiseDescriptorSetLayout
    ) -> SrResult<Self> {
        let device = core.device().inner();

        // --- 1. Shader Loading Helper (Your Syntax) ---
        let make_shader_stage_create_info =
            |stage: vk::ShaderStageFlags, spirv: &[u8]| -> SrResult<vk::PipelineShaderStageCreateInfo> {

                let spirv_u32 = bytemuck::cast_slice(spirv);

                let module_create_info = vk::ShaderModuleCreateInfo::default()
                    .flags(vk::ShaderModuleCreateFlags::empty())
                    .code(spirv_u32);

                let module = unsafe { device.create_shader_module(&module_create_info, None) }?;

                let stage_create_info = vk::PipelineShaderStageCreateInfo::default()
                    .name(SHADER_ENTRY_POINT)
                    .module(module)
                    .stage(stage);

                Ok(stage_create_info)
            };

        // --- 2. Load Denoise Shader ---
        let denoise_stage_create_info = make_shader_stage_create_info(
            vk::ShaderStageFlags::COMPUTE,
            include_bytes_align_as!(u32, concat!(env!("OUT_DIR"), "/denoise.spirv")),
        )?;

        // --- 3. Descriptor Set Layout ---
        let set_layouts = [descriptor_set_layout.inner()];

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts);

        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

        // --- 5. Create Compute Pipeline ---
        let pipeline_info = vk::ComputePipelineCreateInfo::default()
            .stage(denoise_stage_create_info)
            .layout(pipeline_layout);

        let pipelines = unsafe {
            device.create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|(_, err)| err)?
        };
        let pipeline = pipelines[0];

        // --- 6. Cleanup Shader Module ---
        unsafe {
            device.destroy_shader_module(denoise_stage_create_info.module, None);
        }

        Ok(Self {
            core,
            pipeline,
            pipeline_layout,
            descriptor_set_layout: descriptor_set_layout.inner(),       //TODO this could be redundant
        })
    }

    // Getters for usage in the command buffer
    pub fn inner(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }

    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
}

impl Drop for DenoisePipeline {
    fn drop(&mut self) {
        let device = self.core.device().inner();
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);

        }
    }
}