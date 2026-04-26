use crate::vulkan_abstraction;

/// GPU-facing PBR material. Layout matches the `material_t` struct in the
/// shaders (see `common.glsl`) — fields are ordered and packed to satisfy
/// std430 alignment expectations.
///
/// Public fields use raw types rather than `Option<_>` / enums so the layout
/// stays predictable: texture indices are encoded as `u32` where
/// `Material::NULL_TEXTURE_INDEX` (= `u32::MAX`) means "no texture bound"
/// and any other value indexes the texture slot table. For convenience, use
/// `Material::default()` as a starting point and override individual fields
/// with struct-update syntax:
///
/// ```ignore
/// let mat = Material {
///     base_color_value: [0.85, 0.25, 0.25, 1.0],
///     roughness_factor: 0.6,
///     base_color_texture_index: my_texture_handle.0,
///     ..Material::default()
/// };
/// ```
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Material {
    pub base_color_value: [f32; 4],
    pub base_color_texture_index: u32,

    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture_index: u32,

    pub normal_texture_index: u32,
    pub occlusion_texture_index: u32,

    pub _padding: [f32; 2],

    /// rgb = emissive color, a = emissive strength
    pub emissive_factor: [f32; 4],
    pub emissive_texture_index: u32,

    pub alpha_mode: u32,
    pub alpha_cutoff: f32,

    pub transmission_factor: f32,
    pub ior: f32,

    pub _end_padding: [u32; 3],
}

impl Material {
    /// Sentinel stored in a texture index field to indicate "no texture —
    /// fall back to the scalar factor". Matches `null_texture` in the shader.
    pub const NULL_TEXTURE_INDEX: u32 = u32::MAX;
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color_value: [1.0, 1.0, 1.0, 1.0],
            base_color_texture_index: Self::NULL_TEXTURE_INDEX,

            metallic_factor: 0.0,
            roughness_factor: 0.5,
            metallic_roughness_texture_index: Self::NULL_TEXTURE_INDEX,

            normal_texture_index: Self::NULL_TEXTURE_INDEX,
            occlusion_texture_index: Self::NULL_TEXTURE_INDEX,

            _padding: [0.0; 2],

            emissive_factor: [0.0, 0.0, 0.0, 0.0],
            emissive_texture_index: Self::NULL_TEXTURE_INDEX,

            alpha_mode: 0,
            alpha_cutoff: 0.5,

            transmission_factor: 0.0,
            ior: 1.5,

            _end_padding: [0; 3],
        }
    }
}

impl From<&vulkan_abstraction::gltf::Material> for Material {
    fn from(material: &vulkan_abstraction::gltf::Material) -> Self {
        let to_texture_index = |i: Option<usize>| -> u32 {
            match i {
                Some(i) => i as u32,
                None => Self::NULL_TEXTURE_INDEX,
            }
        };

        Self {
            base_color_value: material.pbr_metallic_roughness_properties.base_color_factor,
            base_color_texture_index: to_texture_index(material.pbr_metallic_roughness_properties.base_color_texture_index),

            metallic_factor: material.pbr_metallic_roughness_properties.metallic_factor,
            roughness_factor: material.pbr_metallic_roughness_properties.roughness_factor,
            metallic_roughness_texture_index: to_texture_index(
                material.pbr_metallic_roughness_properties.metallic_roughness_texture_index,
            ),

            normal_texture_index: to_texture_index(material.normal_texture_index),
            occlusion_texture_index: to_texture_index(material.occlusion_texture_index),

            emissive_factor: [
                material.emissive_factor[0],
                material.emissive_factor[1],
                material.emissive_factor[2],
                material.emissive_strength,
            ],
            emissive_texture_index: to_texture_index(material.emissive_texture_index),

            alpha_mode: 0,
            alpha_cutoff: 0.0,
            transmission_factor: material.transmission_factor,
            ior: material.ior,
            _end_padding: [0; 3],
            _padding: [0.0; 2],
        }
    }
}
