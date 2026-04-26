use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use crate::vulkan_abstraction::{ArenaBuffer, Buffer, EntityGpuData, HostAccessibleBuffer, Material, MatricesBufferContents, BLAS};
use crate::{error::SrResult, vulkan_abstraction, CameraMatrices, MeshHandle, TextureHandle, MAX_TLAS_INSTANCES};
use ash::vk;
use log::info;
use rand::{RngExt};

const ARENA_CAPACITY: vk::DeviceSize = 4096;

pub(crate) struct ResourceManager { //TODO ring buffer for cameras and instances_buffer
    // TODo ring buffering this would be needed for cpu stuff too if they were ever uses outside of gpu data build
    // Camera
    matrices_uniform_buffer: vulkan_abstraction::UniformBuffer<vulkan_abstraction::MatricesBufferContents>,

    entities: vulkan_abstraction::ArenaKeyMappedBuffer<vulkan_abstraction::EntityGpuData>,
    // GPU-side: dedicated transform buffer (stride = 48 bytes = VkTransformMatrixKHR), indexed by the same arena slot.
    // Binding 12 reads this as entity_transform_t[slot] in shaders.
    transforms: vulkan_abstraction::ArenaKeyMappedBuffer<vk::TransformMatrixKHR>,
    // CPU-side metadata per entity (blas_index, transform — needed for TLAS rebuild & emissive indirection)
    entity_data: BTreeMap<u64, vulkan_abstraction::Entity>,


    // Acceleration structures
    blases: BTreeMap<u64 ,vulkan_abstraction::BLAS>,
    tlas: vulkan_abstraction::TLAS,

    instances_buffer: vulkan_abstraction::StagingBuffer<vk::AccelerationStructureInstanceKHR>,

    /// instance index to entity this is needed to get O(1) reverse search on blas instance removal
    instance_to_entity: BTreeMap<u64, u64 >,


    // Emissive lighting — local-space triangles stored per-BLAS (arena ring buffer)
    blas_emissive_triangles: vulkan_abstraction::ArenaGpuBuffer<vulkan_abstraction::gltf::EmissiveTriangle>,
    // Dense indirection buffer for NEE sampling: (blas_tri_index, entity_arena_slot) pairs
    emissive_indirection_gpu: vulkan_abstraction::GpuOnlyBuffer,


    // Textures
    textures: Vec<(vk::Sampler, vk::ImageView)>,

    // Samplers loaded from scene
    samplers: Vec<vulkan_abstraction::Sampler>,

    // Owned images with unique IDs (includes scene images)
    images: BTreeMap<u64, vulkan_abstraction::Image>,

    // User-created texture state. Slots are allocated from the top of the
    // 1024-slot table downwards so they don't collide with scene-loaded
    // textures (which fill from slot 0 upward via `set_textures`).
    user_texture_samplers: BTreeMap<u32, vulkan_abstraction::Sampler>,
    user_texture_images: BTreeMap<u32, u64>,
    next_user_texture_slot: u32,

    // Fallback and default textures/samplers
    fallback_texture_image: vulkan_abstraction::Image,
    fallback_texture_sampler: vulkan_abstraction::Sampler,
    default_sampler: vulkan_abstraction::Sampler,

    //these are action to be done at the start or end of frame together with queued free slots for arena buffers
    buffer_copies_queued : Vec<(vk::Buffer,vk::Buffer, vk::BufferCopy)>,

    core: Rc<vulkan_abstraction::Core>,
}

impl ResourceManager {
    pub const NUMBER_OF_SAMPLERS: usize = 1024;

    pub fn new_empty(core: Rc<vulkan_abstraction::Core>) -> SrResult<Self> {
        let matrices_uniform_buffer = vulkan_abstraction::UniformBuffer::new(Rc::clone(&core), 1 as vk::DeviceSize)?;

        let entities = vulkan_abstraction::ArenaKeyMappedBuffer::new(
            core.clone(),
            ARENA_CAPACITY,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
            "Entities GPU buffer",
        )?;

        let transforms = vulkan_abstraction::ArenaKeyMappedBuffer::new(
            core.clone(),
            ARENA_CAPACITY,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            "Entity transforms GPU buffer",
        )?;

        let mut instances_buffer = vulkan_abstraction::StagingBuffer::new(
            Rc::clone(&core),
            MAX_TLAS_INSTANCES as vk::DeviceSize,
            vk::BufferUsageFlags::STORAGE_BUFFER
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            "Cpu side instances of blases",
        )?;
        let tlas = vulkan_abstraction::TLAS::new(Rc::clone(&core), &[], &mut instances_buffer)?;

        let fallback_texture_image = {
            const RESOLUTION: u32 = 64;
            let image_data = crate::utils::iterate_image_extent(RESOLUTION, RESOLUTION)
                .map(|(x, y)| {
                    if (x + y).is_multiple_of(2) {
                        0xff000000u32
                    } else {
                        0xffff00ffu32
                    }
                })
                .map(u32::to_be_bytes)
                .flatten()
                .collect::<Vec<u8>>();

            vulkan_abstraction::Image::new_from_data(
                Rc::clone(&core),
                image_data,
                vk::Extent3D {
                    width: RESOLUTION,
                    height: RESOLUTION,
                    depth: 1,
                },
                vk::Format::R8G8B8A8_UNORM,
                vk::ImageTiling::OPTIMAL,
                gpu_allocator::MemoryLocation::GpuOnly,
                vk::ImageUsageFlags::SAMPLED,
                "fallback texture image",
            )?
        };
        let fallback_texture_sampler = vulkan_abstraction::Sampler::new(
            Rc::clone(&core),
            vk::Filter::NEAREST,
            vk::Filter::NEAREST,
            vk::SamplerAddressMode::REPEAT,
            vk::SamplerAddressMode::REPEAT,
            vk::SamplerAddressMode::REPEAT,
            vk::SamplerMipmapMode::LINEAR,
        )?;
        let default_sampler = vulkan_abstraction::Sampler::new(
            Rc::clone(&core),
            vk::Filter::LINEAR,
            vk::Filter::LINEAR,
            vk::SamplerAddressMode::CLAMP_TO_EDGE,
            vk::SamplerAddressMode::CLAMP_TO_EDGE,
            vk::SamplerAddressMode::CLAMP_TO_EDGE,
            vk::SamplerMipmapMode::LINEAR,
        )?;

        let mut manager = Self {
            matrices_uniform_buffer,

            entities,
            transforms,
            entity_data: BTreeMap::new(),

            blases: Default::default(),
            tlas,
            instances_buffer,

            instance_to_entity: Default::default(),
            blas_emissive_triangles: vulkan_abstraction::ArenaGpuBuffer::new(
                core.clone(),
                ARENA_CAPACITY,
                vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
                "blas emissive triangles",
            )?,

            emissive_indirection_gpu: vulkan_abstraction::Buffer::new_null(Rc::clone(&core)),
            textures: Vec::new(),

            samplers: Vec::new(),

            images: BTreeMap::new(),

            user_texture_samplers: BTreeMap::new(),
            user_texture_images: BTreeMap::new(),
            next_user_texture_slot: (Self::NUMBER_OF_SAMPLERS as u32) - 1,

            fallback_texture_image,
            fallback_texture_sampler,
            default_sampler,

            buffer_copies_queued: vec![],
            core,
        };

        // Pre-fill the texture slot table with fallback entries so
        // `build_image_dependent_data` can run without a glTF scene loaded —
        // otherwise the descriptor-set assert at
        // `raytracing_descriptor_set.rs:342` fires with `0 != 1024`.
        manager.set_textures(&[], &[], &[]);

        // Seed the emissive indirection buffer with a dummy entry so descriptor
        // writes never bind VK_NULL_HANDLE (validation error
        // VUID-VkDescriptorBufferInfo-buffer-02998). Later mutations through
        // `rebuild_emissive_indirection` replace this with real data.
        manager.rebuild_emissive_indirection()?;

        Ok(manager)
    }


    pub fn empty_out(self) -> SrResult<Self> {
        Self::new_empty(self.core)
    }


    pub fn start_of_frame(&mut self) -> SrResult<()> {
        let frame =
        self.entities.process_pending_frees();
        self.blas_emissive_triangles.process_pending_frees();
        self.transforms.process_pending_frees();



        if self.buffer_copies_queued.is_empty() {
            return Ok(());
        }

        let copies = std::mem::take(&mut self.buffer_copies_queued);


        let mut seen: HashMap<(vk::Buffer, vk::DeviceSize, vk::DeviceSize), usize> = HashMap::new();
        for (i, (_, dst, region)) in copies.iter().enumerate() {
            seen.insert((*dst, region.dst_offset, region.size), i);
        }
        let copies: Vec<_> = seen.values().map(|&i| copies[i]).collect();


        let device = self.core.device().inner();
        let graphics_queue = self.core.graphics_queue();
        let cmd_pool = self.core.graphics_cmd_pool();

        let cmd_buf = vulkan_abstraction::cmd_buffer::new_command_buffer(cmd_pool, device)?;
        let begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device.begin_command_buffer(cmd_buf, &begin_info)?;

            for (src, dst, region) in &copies {
                device.cmd_copy_buffer(cmd_buf, *src, *dst, std::slice::from_ref(region));
            }

            let unique_dsts: std::collections::HashSet<vk::Buffer> = copies.iter().map(|(_, dst, _)| *dst).collect();
            let barriers: Vec<vk::BufferMemoryBarrier2> = unique_dsts
                .into_iter()
                .map(|buf| {
                    vk::BufferMemoryBarrier2::default()
                        .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                        .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                        .dst_stage_mask(
                            vk::PipelineStageFlags2::RAY_TRACING_SHADER_KHR | vk::PipelineStageFlags2::COMPUTE_SHADER,
                        )
                        .dst_access_mask(vk::AccessFlags2::SHADER_READ )
                        .buffer(buf)
                        .offset(0)
                        .size(vk::WHOLE_SIZE)
                })
                .collect();

            let dependency_info = vk::DependencyInfo::default().buffer_memory_barriers(&barriers);
            device.cmd_pipeline_barrier2(cmd_buf, &dependency_info);

            device.end_command_buffer(cmd_buf)?;
        }

        graphics_queue.submit_sync(cmd_buf)?;
        unsafe { device.free_command_buffers(cmd_pool.inner(), &[cmd_buf]) };

        Ok(())
    }

    // ─── Camera ──────────────────────────────────────────────────────────────

    pub fn set_matrices(
        &mut self,
        CameraMatrices {
            view_inverse,
            proj_inverse,
            view_proj,
            prev_view_proj,
        }: CameraMatrices,
    ) -> SrResult<()> {
        let mem = self.matrices_uniform_buffer.map_mut()?;
        mem[0] = MatricesBufferContents {
            view_inverse,
            proj_inverse,
            view_proj,
            prev_view_proj,
        };
        Ok(())
    }

    // ─── Entity management ───────────────────────────────────────────────────

    /// Build the GPU data for an entity from its BLAS and material.
    fn build_entity_gpu_data(
        blas: &vulkan_abstraction::BLAS,
        material: &Material,
    ) -> EntityGpuData {
        EntityGpuData {
            vertex_buffer: blas.vertex_buffer().get_device_address(),
            index_buffer: blas.index_buffer().get_device_address(),
            material: *material,
        }
    }

    /// Create an entity with the given BLAS index, material, and transform.
    /// Returns the entity ID. After this call the TLAS must be rebuilt by the caller.
    pub fn create_entity(
        &mut self,
        blas_index: u64,
        material: &Material,
        transform: vk::TransformMatrixKHR,
    ) -> SrResult<vulkan_abstraction::EntityId> {
        let id = Self::generate(&self.entity_data);

        let gpu_data = Self::build_entity_gpu_data(&self.blases[&blas_index], material);
        let (slot, copy_region) = self.entities.insert(id, &gpu_data)?;
        self.copy_commands_queue(self.entities.inner_staging(), self.entities.inner(), copy_region);

        let (_, xform_copy) = self.transforms.insert(id, &transform)?;
        self.copy_commands_queue(self.transforms.inner_staging(), self.transforms.inner(), xform_copy);

        let entity = vulkan_abstraction::Entity {
            id: vulkan_abstraction::EntityId(id),
            blas_index,
            transform,
            material: *material,
            blas_instance_index: slot as u64,
        };
        self.instance_to_entity.insert(slot as u64, id);
        self.entity_data.insert(id, entity);

        Ok(vulkan_abstraction::EntityId(id))
    }

    /// Destroy an entity, freeing its arena slots. The TLAS must be rebuilt by the caller.
    pub fn destroy_entity(&mut self, id: vulkan_abstraction::EntityId) {
        if let Some(entity) = self.entity_data.remove(&id.0) {
            self.instance_to_entity.remove(&entity.blas_instance_index);
            self.entities.remove(id.0);
            self.transforms.remove(id.0);
        }
    }

    /// Update an entity's transform. The TLAS must be rebuilt or updated by the caller.
    pub fn set_entity_transform(&mut self, id: vulkan_abstraction::EntityId, transform: vk::TransformMatrixKHR) -> SrResult<()> {
        if let Some(entity) = self.entity_data.get_mut(&id.0) {
            entity.transform = transform;
            let (_, xform_copy) = self.transforms.insert(id.0, &transform)?;
            self.copy_commands_queue(self.transforms.inner_staging(), self.transforms.inner(), xform_copy);
        }
        Ok(())
    }

    /// Overwrite the material stored in an entity's `EntityGpuData` slot.
    /// Does not change the BLAS or transform and does not touch descriptor
    /// set bindings — the shader re-reads the entity buffer each trace.
    pub fn set_entity_material(
        &mut self,
        id: vulkan_abstraction::EntityId,
        material: &Material,
    ) -> SrResult<()> {
        let Some(entity) = self.entity_data.get_mut(&id.0) else {
            return Ok(());
        };
        entity.material = *material;
        let blas_index = entity.blas_index;
        let gpu_data = Self::build_entity_gpu_data(&self.blases[&blas_index], material);
        let (_, copy) = self.entities.insert(id.0, &gpu_data)?;
        self.copy_commands_queue(self.entities.inner_staging(), self.entities.inner(), copy);
        Ok(())
    }

    pub fn get_entity(&self, id: vulkan_abstraction::EntityId) -> Option<&vulkan_abstraction::Entity> {
        self.entity_data.get(&id.0)
    }

    /// Return the entity's current world transform as stored on the CPU side.
    /// Returns `None` if the entity has been destroyed or was never created.
    pub fn get_entity_transform(&self, id: vulkan_abstraction::EntityId) -> Option<vk::TransformMatrixKHR> {
        self.entity_data.get(&id.0).map(|e| e.transform)
    }

    pub fn entity_data(&self) -> &BTreeMap<u64, vulkan_abstraction::Entity> {
        &self.entity_data
    }

    // ─── Emissive triangles (per-BLAS, local-space) ──────────────────────────

    /// Append local-space emissive triangles for a BLAS into the arena ring buffer.
    /// Allocates slots and flushes the staging copies to GPU. Returns the
    /// `start..end` slot range the triangles were assigned — callers that
    /// need to build a BLAS referencing these triangles store the range in
    /// `BLAS::emissive_triangle_ranges`. The range is contiguous as long as
    /// the arena hasn't had per-slot frees in between (true for both the
    /// load_scene and create_mesh paths today).
    pub fn add_blas_emissive_triangles(
        &mut self,
        triangles: &[vulkan_abstraction::gltf::EmissiveTriangle],
    ) -> SrResult<std::ops::Range<u32>> {
        if triangles.is_empty() {
            return Ok(0..0);
        }

        let mut first: Option<u32> = None;
        let mut last: u32 = 0;
        for tri in triangles {
            let (slot, copy_region) = self.blas_emissive_triangles.allocate_and_update(tri)?;
            self.copy_commands_queue(
                self.blas_emissive_triangles.inner_staging(),
                self.blas_emissive_triangles.inner(),
                copy_region,
            );
            let slot = slot as u32;
            if first.is_none() {
                first = Some(slot);
            }
            last = slot;
        }

        Ok(first.unwrap()..last + 1)
    }

    /// Rebuild the dense emissive indirection buffer from all live entities and their BLASes' ranges.
    pub fn rebuild_emissive_indirection(&mut self) -> SrResult<()> {
        let mut entries = Vec::new();

        for (&entity_id, entity) in &self.entity_data {
            let blas = &self.blases[&entity.blas_index];
            let arena_slot = self.entities.get_slot(entity_id).unwrap_or(0);
            for range in blas.emissive_triangle_ranges() {
                for tri_idx in range.clone() {
                    entries.push(vulkan_abstraction::gltf::EmissiveIndirectionEntry {
                        blas_tri_index: tri_idx,
                        entity_id: arena_slot as u32, //TODO this is putting a u64 into a u32 this is collision at its finest
                    });
                }
            }
        }

        if entries.is_empty() {
            let dummy = [vulkan_abstraction::gltf::EmissiveIndirectionEntry {
                blas_tri_index: 0,
                entity_id: 0,
            }];
            self.emissive_indirection_gpu = vulkan_abstraction::GpuOnlyBuffer::new_from_data(
                Rc::clone(&self.core),
                &dummy,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                "emissive indirection dummy",
            )?;
        } else {
            self.emissive_indirection_gpu = vulkan_abstraction::GpuOnlyBuffer::new_from_data(
                Rc::clone(&self.core),
                &entries,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                "emissive indirection",
            )?;
        }

        Ok(())
    }

    // ─── Image storage ───────────────────────────────────────────────────────

    /// Take ownership of an image and return a unique ID for it.
    pub fn add_image(&mut self, image: vulkan_abstraction::Image) -> u64 {
        let id = Self::generate(&self.images);
        self.images.insert(id, image);
        id
    }

    /// Remove and destroy an image by its ID. No-op if the ID doesn't exist.
    pub fn remove_image(&mut self, id: u64) {
        self.images.remove(&id);
    }

    pub fn get_image(&self, id: u64) -> Option<&vulkan_abstraction::Image> {
        self.images.get(&id)
    }

    // ─── Acceleration structures ───────────────────────────────────────────

    pub fn tlas(&self) -> &vulkan_abstraction::TLAS {
        &self.tlas
    }

    pub fn blases(&self) -> &BTreeMap<u64, BLAS> {
        &self.blases
    }

    pub fn blases_mut(&mut self) -> &mut BTreeMap<u64, BLAS> {
        &mut self.blases
    }

    pub fn rebuild_tlas(&mut self) -> SrResult<()> {
        self.tlas.rebuild_from_entities(&self.entity_data, &self.blases, &mut self.instances_buffer)
    }

    pub fn update_tlas(&mut self) -> SrResult<()> {
        self.tlas.update_from_entities(&self.entity_data, &self.blases, &mut self.instances_buffer)
    }

    // ─── Textures ───────────────────────────────────────────────────────────

    pub fn default_sampler(&self) -> &vulkan_abstraction::Sampler {
        &self.default_sampler
    }



    pub fn set_textures(
        &mut self,
        images: &[vulkan_abstraction::Image],
        samplers: &[vulkan_abstraction::Sampler],
        textures: &[vulkan_abstraction::gltf::Texture],
    ) {
        self.textures.clear();
        self.textures.reserve_exact(Self::NUMBER_OF_SAMPLERS);

        for tex in textures {
            let sampler = match tex.sampler {
                Some(i) => &samplers[i],
                None => &self.default_sampler,
            };
            let image = &images[tex.source];
            self.textures.push((sampler.inner(), image.image_view()));
        }

        while self.textures.len() < Self::NUMBER_OF_SAMPLERS {
            self.textures.push((
                self.fallback_texture_sampler.inner(),
                self.fallback_texture_image.image_view(),
            ));
        }

        assert_eq!(self.textures.len(), Self::NUMBER_OF_SAMPLERS);

        // Re-apply any user-created textures that `clear` just wiped out.
        // User slots are allocated from the top of the table downward, so
        // they don't collide with scene-loaded textures occupying low slots.
        for (&slot, sampler) in &self.user_texture_samplers {
            if let Some(img_id) = self.user_texture_images.get(&slot) {
                if let Some(image) = self.images.get(img_id) {
                    self.textures[slot as usize] = (sampler.inner(), image.image_view());
                }
            }
        }
    }

    // ─── Standalone texture upload ───────────────────────────────────────────

    /// Upload a texture and register it in the descriptor slot table. Returns
    /// the slot index the caller stores on a material texture field.
    /// Descriptor sets referencing `get_textures()` need to be rebuilt after
    /// this call (the `Renderer` wrapper clears image-dependent data).
    pub fn create_texture(
        &mut self,
        data: Vec<u8>,
        extent: vk::Extent3D,
        format: vk::Format,
    ) -> SrResult<TextureHandle> {
        if self.user_texture_samplers.len() >= Self::NUMBER_OF_SAMPLERS {
            return Err(crate::error::SrError::new_custom(
                "texture slot table exhausted".to_string(),
            ));
        }

        let image = vulkan_abstraction::Image::new_from_data(
            Rc::clone(&self.core),
            data,
            extent,
            format,
            vk::ImageTiling::OPTIMAL,
            gpu_allocator::MemoryLocation::GpuOnly,
            vk::ImageUsageFlags::SAMPLED,
            "user texture",
        )?;
        let sampler = vulkan_abstraction::Sampler::new(
            Rc::clone(&self.core),
            vk::Filter::LINEAR,
            vk::Filter::LINEAR,
            vk::SamplerAddressMode::REPEAT,
            vk::SamplerAddressMode::REPEAT,
            vk::SamplerAddressMode::REPEAT,
            vk::SamplerMipmapMode::LINEAR,
        )?;

        let slot = self.next_user_texture_slot;
        let image_id = self.add_image(image);
        let image_view = self.images[&image_id].image_view();
        let sampler_inner = sampler.inner();

        // `textures` is lazily populated on the first `set_textures` call —
        // new_empty already invokes that with empty inputs, so the table is
        // always sized to NUMBER_OF_SAMPLERS here.
        self.textures[slot as usize] = (sampler_inner, image_view);
        self.user_texture_samplers.insert(slot, sampler);
        self.user_texture_images.insert(slot, image_id);

        self.next_user_texture_slot = self.next_user_texture_slot.saturating_sub(1);

        Ok(TextureHandle(slot))
    }

    /// Release a texture previously returned by `create_texture`. The slot
    /// reverts to the fallback pink-checker texture and is not recycled
    /// (recycling would risk collisions with materials still referencing it).
    pub fn destroy_texture(&mut self, handle: TextureHandle) {
        let slot = handle.0;
        if let Some(img_id) = self.user_texture_images.remove(&slot) {
            self.remove_image(img_id);
        }
        self.user_texture_samplers.remove(&slot);
        if (slot as usize) < self.textures.len() {
            self.textures[slot as usize] = (
                self.fallback_texture_sampler.inner(),
                self.fallback_texture_image.image_view(),
            );
        }
    }

    // ─── Standalone mesh upload (no glTF) ────────────────────────────────────

    /// Upload `mesh` as a new BLAS and return its handle. The caller pairs it
    /// with `create_entity` to place instances in the scene; no TLAS rebuild
    /// happens here. If `mesh.emission` is `Some`, the mesh's triangles are
    /// also registered for NEE sampling with the given local-space radiance.
    pub fn create_mesh(&mut self, mesh: &crate::MeshData) -> SrResult<MeshHandle> {
        let vertices: Vec<vulkan_abstraction::gltf::Vertex> = mesh.into();
        let vertex_buffer = vulkan_abstraction::VertexBuffer::new_for_blas_from_data(
            Rc::clone(&self.core),
            &vertices,
        )?;
        let index_buffer = vulkan_abstraction::IndexBuffer::new_for_blas_from_data(
            Rc::clone(&self.core),
            &mesh.indices,
        )?;

        let mut emissive_ranges = Vec::new();
        if let Some(emission) = mesh.emission {
            let tris = build_emissive_triangles(&mesh.positions, &mesh.indices, emission);
            if !tris.is_empty() {
                let range = self.add_blas_emissive_triangles(&tris)?;
                emissive_ranges.push(range);
            }
        }

        let blas = vulkan_abstraction::BLAS::new(
            Rc::clone(&self.core),
            vertex_buffer,
            index_buffer,
            emissive_ranges,
            false,
        )?;
        let id = Self::generate(&self.blases);
        self.blases.insert(id, blas);
        Ok(MeshHandle(id))
    }

    /// Drop the BLAS backing `handle`. The caller must ensure no live entity
    /// still references this mesh and that the GPU is idle.
    pub fn destroy_mesh(&mut self, handle: MeshHandle) {
        self.blases.remove(&handle.0);
    }

    // ─── Scene loading ───────────────────────────────────────────────────────

    pub fn load_scene(&mut self, scene: &crate::Scene, scene_data: crate::SceneData) -> SrResult<Vec<vulkan_abstraction::EntityId>> {
        let mut blases = vec![];
        let (blas_instances, blas_indices, materials, textures, samplers, images, emissive_triangles) =
            scene.load_into_gpu(&self.core, &mut blases, scene_data)?;

        // Collect entity creation data before consuming blas_instances
        let entity_creation_data: Vec<_> = blas_instances
            .iter()
            .zip(blas_indices.iter())
            .zip(materials.iter())
            .map(|((bi, &blas_idx), mat)| (blas_idx, mat.clone(), bi.transform))
            .collect();

        // Insert BLASes with stable IDs; build a remapping from scene index to manager ID.
        let blas_id_map: Vec<u64> = blases
            .into_iter()
            .map(|blas| {
                let id = Self::generate(&self.blases);
                self.blases.insert(id, blas);
                id
            })
            .collect();

        self.set_textures(&images, &samplers, &textures);
        self.add_blas_emissive_triangles(&emissive_triangles)?;

        // Create entities; this assigns arena slots that become gl_InstanceCustomIndexEXT.
        let mut entity_ids = Vec::with_capacity(entity_creation_data.len());
        for (scene_blas_idx, material, transform) in &entity_creation_data {
            let blas_id = blas_id_map[*scene_blas_idx];
            let gpu_material = Material::from(material);
            entity_ids.push(self.create_entity(blas_id, &gpu_material, *transform)?);
        }

        // Rebuild TLAS *after* entity slots are known so instance_custom_index matches.
        self.rebuild_tlas()?;

        self.rebuild_emissive_indirection()?;

        for image in images {
            self.add_image(image);
        }
        self.samplers = samplers;

        Ok(entity_ids)
    }

    /// Spawn a new instance that shares the same BLAS and material as `src` but has a different
    /// transform. The caller must rebuild the TLAS afterwards.
    pub fn clone_entity(&mut self, src: vulkan_abstraction::EntityId, transform: vk::TransformMatrixKHR) -> SrResult<vulkan_abstraction::EntityId> {
        let (blas_index, material) = self
            .entity_data
            .get(&src.0)
            .map(|e| (e.blas_index, e.material))
            .ok_or_else(|| crate::error::SrError::new_custom(format!("clone_entity: no entity {}", src.0)))?;

        let id = Self::generate(&self.entity_data);

        let gpu_data = EntityGpuData {
            vertex_buffer: self.blases[&blas_index].vertex_buffer().get_device_address(),
            index_buffer: self.blases[&blas_index].index_buffer().get_device_address(),
            material,
        };
        let (slot, copy_region) = self.entities.insert(id, &gpu_data)?;
        self.copy_commands_queue(self.entities.inner_staging(), self.entities.inner(), copy_region);

        let (_, xform_copy) = self.transforms.insert(id, &transform)?;
        self.copy_commands_queue(self.transforms.inner_staging(), self.transforms.inner(), xform_copy);

        let entity = vulkan_abstraction::Entity {
            id: vulkan_abstraction::EntityId(id),
            blas_index,
            transform,
            material,
            blas_instance_index: slot as u64,
        };
        self.instance_to_entity.insert(slot as u64, id);
        self.entity_data.insert(id, entity);

        Ok(vulkan_abstraction::EntityId(id))
    }

    // ─── Descriptor set accessors ────────────────────────────────────────────

    pub fn get_matrices_uniform_buffer(&self) -> vk::Buffer {
        self.matrices_uniform_buffer.inner()
    }

    pub fn get_meshes_info_storage_buffer(&self) -> vk::Buffer {
        self.entities.inner()
    }

    pub fn get_emissive_triangles_storage_buffer(&self) -> vk::Buffer {
        self.blas_emissive_triangles.inner()
    }

    pub fn get_emissive_indirection_buffer(&self) -> vk::Buffer {
        self.emissive_indirection_gpu.inner()
    }

    pub fn get_entity_transforms_buffer(&self) -> vk::Buffer {
        self.transforms.inner()
    }

    pub fn get_textures(&self) -> &[(vk::Sampler, vk::ImageView)] {
        &self.textures
    }

    // ─── Internal helpers ────────────────────────────────────────────────────

    fn copy_commands_queue(&mut self, src: vk::Buffer, dst: vk::Buffer, region: vk::BufferCopy) {
        self.buffer_copies_queued.push((src, dst, region));
    }

    pub(crate) fn generate<T>(map: &BTreeMap<u64, T>) -> u64 {
        loop {
            let mut rng = rand::rng();
            let key = rng.random::<u64>();
            if !map.contains_key(&key) {
                return key;
            }
        }
    }
}

/// Pack triangle corners + emission into `EmissiveTriangle`s, matching the
/// layout used by the glTF importer (see `gltf::mod.rs::process_mesh`).
/// Positions are kept in local (BLAS) space — the shader applies the entity
/// transform at sample time.
fn build_emissive_triangles(
    positions: &[[f32; 3]],
    indices: &[u32],
    emission: [f32; 3],
) -> Vec<vulkan_abstraction::gltf::EmissiveTriangle> {
    let emission = [emission[0], emission[1], emission[2], 0.0];
    indices
        .chunks_exact(3)
        .map(|chunk| {
            let p0 = positions[chunk[0] as usize];
            let p1 = positions[chunk[1] as usize];
            let p2 = positions[chunk[2] as usize];
            vulkan_abstraction::gltf::EmissiveTriangle {
                v0: [p0[0], p0[1], p0[2], 0.0],
                v1: [p1[0], p1[1], p1[2], 0.0],
                v2: [p2[0], p2[1], p2[2], 0.0],
                emission,
            }
        })
        .collect()
}