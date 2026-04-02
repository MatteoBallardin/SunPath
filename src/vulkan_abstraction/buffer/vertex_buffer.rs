use std::{ops::Deref, rc::Rc};

use ash::vk;

use crate::{error::*, vulkan_abstraction};

pub struct VertexBuffer<T> {
    buffer: vulkan_abstraction::GpuOnlyBuffer<T>,
    len: usize,
    stride: usize,
}

impl<T> VertexBuffer<T> {
    //build a vertex buffer with flags for usage in a blas
    pub fn new_for_blas_from_data(core: Rc<vulkan_abstraction::Core>, data: &[T]) -> SrResult<Self> where T : Copy {
        let usage_flags = vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::VERTEX_BUFFER
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;

        Ok(Self {
            buffer: vulkan_abstraction::GpuOnlyBuffer::new_from_data(
                core,
                data,
                usage_flags,
                "vertex buffer for BLAS usage",
            )?,
            len: data.len(),
            stride: std::mem::size_of::<T>(),
        })
    }

    //build a vertex buffer with flags for usage in a blas
    pub fn new_for_blas(core: Rc<vulkan_abstraction::Core>, len: usize) -> SrResult<Self> {
        let usage_flags = vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::VERTEX_BUFFER
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;

        Ok(Self {
            buffer: vulkan_abstraction::GpuOnlyBuffer::new(
                core,
                len,
                usage_flags,
                "vertex buffer for BLAS usage",
            )?,
            len,
            stride: std::mem::size_of::<T>(),
        })
    }

    #[allow(dead_code)]
    pub fn buffer(&self) -> &vulkan_abstraction::GpuOnlyBuffer<T> {
        &self.buffer
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn stride(&self) -> usize {
        self.stride
    }
}
impl<T> Deref for VertexBuffer<T> {
    type Target = vulkan_abstraction::GpuOnlyBuffer<T>;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
