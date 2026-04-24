use gltf::json::extensions::material::EmissiveStrength;

#[derive(Clone, Debug)]
pub struct PbrMetallicRoughnessProperties {
    pub base_color_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub base_color_texture_index: Option<usize>,
    pub metallic_roughness_texture_index: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Material {
    pub pbr_metallic_roughness_properties: PbrMetallicRoughnessProperties,
    pub normal_texture_index: Option<usize>,
    pub occlusion_texture_index: Option<usize>,
    pub emissive_factor: [f32; 3],
    pub emissive_strength: f32,
    pub emissive_texture_index: Option<usize>,
    pub alpha_mode: gltf::material::AlphaMode,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
    pub transmission_factor: f32,
    pub ior: f32,
}

impl Default for PbrMetallicRoughnessProperties {
    fn default() -> Self {
        Self {
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            base_color_texture_index: None,
            metallic_roughness_texture_index: None,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            pbr_metallic_roughness_properties: PbrMetallicRoughnessProperties::default(),
            normal_texture_index: None,
            occlusion_texture_index: None,
            emissive_factor: [0.0, 0.0, 0.0],
            emissive_strength: 0.0,
            emissive_texture_index: None,
            alpha_mode: gltf::material::AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
            transmission_factor: 0.0,
            ior: 1.5,
        }
    }
}
