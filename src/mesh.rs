//! Public, backend-agnostic mesh input used by `Renderer::create_mesh`.
//!
//! Meshes are uploaded through the same `VertexBuffer` / `IndexBuffer` / BLAS
//! pipeline as glTF-loaded geometry; the `From<&MeshData> for Vec<Vertex>`
//! impl packs the caller's attribute arrays into sunray's internal
//! `gltf::Vertex` layout.

use crate::vulkan_abstraction::gltf::Vertex;

/// Raw mesh data ready for upload. `positions` and `indices` are required.
/// Per-vertex arrays, when present, must be the same length as `positions`.
#[derive(Clone, Default, Debug)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub tangents: Option<Vec<[f32; 4]>>,
    pub base_color_uvs: Option<Vec<[f32; 2]>>,
    pub metallic_roughness_uvs: Option<Vec<[f32; 2]>>,
    pub normal_uvs: Option<Vec<[f32; 2]>>,
    pub occlusion_uvs: Option<Vec<[f32; 2]>>,
    pub emissive_uvs: Option<Vec<[f32; 2]>>,
}

/// Opaque handle for a mesh registered in sunray. Obtained from
/// `Renderer::create_mesh`; passed to `Renderer::create_entity` and
/// `Renderer::destroy_mesh`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MeshHandle(pub u64);

impl From<&MeshData> for Vec<Vertex> {
    fn from(mesh: &MeshData) -> Self {
        let n = mesh.positions.len();
        let uv = |o: &Option<Vec<[f32; 2]>>, i: usize| {
            o.as_ref().and_then(|v| v.get(i).copied()).unwrap_or([0.0, 0.0])
        };
        (0..n)
            .map(|i| Vertex {
                position: mesh.positions[i],
                _padding0: [0.0; 1],
                normal: mesh.normals.get(i).copied().unwrap_or([0.0, 0.0, 0.0]),
                _padding1: [0.0; 1],
                tangent: mesh
                    .tangents
                    .as_ref()
                    .and_then(|v| v.get(i).copied())
                    .unwrap_or([0.0; 4]),
                base_color_tex_coord: uv(&mesh.base_color_uvs, i),
                metallic_roughness_tex_coord: uv(&mesh.metallic_roughness_uvs, i),
                normal_tex_coord: uv(&mesh.normal_uvs, i),
                occlusion_tex: uv(&mesh.occlusion_uvs, i),
                emissive_tex: uv(&mesh.emissive_uvs, i),
                _padding3: [0.0; 2],
            })
            .collect()
    }
}
