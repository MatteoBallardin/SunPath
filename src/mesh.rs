//! Public, backend-agnostic mesh input used by `Renderer::create_mesh`.
//!
//! Meshes are uploaded through the same `VertexBuffer` / `IndexBuffer` / BLAS
//! pipeline as glTF-loaded geometry; the `From<&MeshData> for Vec<Vertex>`
//! impl packs the caller's attribute arrays into sunray's internal
//! `gltf::Vertex` layout.

use crate::vulkan_abstraction::gltf::Vertex;

/// Raw mesh data ready for upload. `positions` and `indices` are required.
/// Per-vertex arrays, when present, must be the same length as `positions`.
///
/// If `emission` is `Some`, the mesh's triangles are registered for
/// Next-Event-Estimation sampling with the given (already multiplied) RGB
/// emission radiance — that makes the mesh act as an area light and
/// illuminate other geometry. Leaving `emission` as `None` still renders a
/// correct direct hit on an emissive material (the closest-hit shader reads
/// `emissive_factor * emissive_strength` from the material), but the mesh
/// won't light anything else.
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
    pub emission: Option<[f32; 3]>,
}

/// Opaque handle for a mesh registered in sunray. Obtained from
/// `Renderer::create_mesh`; passed to `Renderer::create_entity` and
/// `Renderer::destroy_mesh`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MeshHandle(pub u64);

/// Handle for a texture uploaded via `Renderer::create_texture`. The inner
/// value is the shader-side descriptor slot; assign it to a
/// `gltf::Material` texture index field via `.slot()` to make a material
/// sample from it (e.g. `base_color_texture_index = Some(handle.slot())`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextureHandle(pub u32);

impl TextureHandle {
    /// Returns the texture slot index usable as a `gltf::Material` texture
    /// index (`Option<usize>`).
    pub fn slot(&self) -> usize {
        self.0 as usize
    }
}

impl MeshData {
    /// Flat-shaded axis-aligned cube centered at the origin with edge
    /// length `2 * half_extent`. Each face gets its own four vertices so
    /// normals are constant per-face, and standard 0..1 UVs are emitted on
    /// `base_color_uvs` so the mesh can sample a texture out of the box.
    pub fn cube(half_extent: f32) -> Self {
        let h = half_extent;
        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            ([-1.0, 0.0, 0.0], [[-h, -h, -h], [-h, -h,  h], [-h,  h,  h], [-h,  h, -h]]),
            ([ 1.0, 0.0, 0.0], [[ h, -h,  h], [ h, -h, -h], [ h,  h, -h], [ h,  h,  h]]),
            ([0.0, -1.0, 0.0], [[-h, -h, -h], [ h, -h, -h], [ h, -h,  h], [-h, -h,  h]]),
            ([0.0,  1.0, 0.0], [[-h,  h,  h], [ h,  h,  h], [ h,  h, -h], [-h,  h, -h]]),
            ([0.0, 0.0, -1.0], [[ h, -h, -h], [-h, -h, -h], [-h,  h, -h], [ h,  h, -h]]),
            ([0.0, 0.0,  1.0], [[-h, -h,  h], [ h, -h,  h], [ h,  h,  h], [-h,  h,  h]]),
        ];
        const FACE_UVS: [[f32; 2]; 4] =
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        let mut positions = Vec::with_capacity(24);
        let mut normals = Vec::with_capacity(24);
        let mut uvs = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);
        for (face_idx, (normal, verts)) in faces.iter().enumerate() {
            let base = (face_idx * 4) as u32;
            for (vi, v) in verts.iter().enumerate() {
                positions.push(*v);
                normals.push(*normal);
                uvs.push(FACE_UVS[vi]);
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        Self {
            positions,
            normals,
            indices,
            base_color_uvs: Some(uvs),
            ..Default::default()
        }
    }

    /// Axis-aligned plane on the XZ plane centered at the origin, facing
    /// +Y, with edge length `size`. Emits UVs that map the whole 0..1
    /// square onto the plane.
    pub fn plane(size: f32) -> Self {
        let h = size * 0.5;
        Self {
            positions: vec![
                [-h, 0.0, -h],
                [ h, 0.0, -h],
                [ h, 0.0,  h],
                [-h, 0.0,  h],
            ],
            normals: vec![[0.0, 1.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            base_color_uvs: Some(vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
            ]),
            ..Default::default()
        }
    }

    /// UV sphere of the given radius, with `latitudes` horizontal rings
    /// and `longitudes` vertical slices. Normals are smooth (per-vertex
    /// unit radial). `latitudes` and `longitudes` are clamped to a minimum
    /// of 3 each; larger values produce a rounder sphere at the cost of
    /// more triangles.
    pub fn sphere(radius: f32, longitudes: u32, latitudes: u32) -> Self {
        use std::f32::consts::{PI, TAU};
        let lon = longitudes.max(3);
        let lat = latitudes.max(3);

        let vert_count = ((lat + 1) * (lon + 1)) as usize;
        let mut positions = Vec::with_capacity(vert_count);
        let mut normals = Vec::with_capacity(vert_count);
        let mut uvs = Vec::with_capacity(vert_count);

        for i in 0..=lat {
            let v = i as f32 / lat as f32;
            let theta = v * PI;
            let sin_t = theta.sin();
            let cos_t = theta.cos();
            for j in 0..=lon {
                let u = j as f32 / lon as f32;
                let phi = u * TAU;
                let dir = [sin_t * phi.cos(), cos_t, sin_t * phi.sin()];
                positions.push([dir[0] * radius, dir[1] * radius, dir[2] * radius]);
                normals.push(dir);
                uvs.push([u, v]);
            }
        }

        let row = lon + 1;
        let mut indices = Vec::with_capacity((lat * lon * 6) as usize);
        for i in 0..lat {
            for j in 0..lon {
                let a = i * row + j;
                let b = a + 1;
                let c = (i + 1) * row + j;
                let d = c + 1;
                indices.extend_from_slice(&[a, c, d, a, d, b]);
            }
        }

        Self {
            positions,
            normals,
            indices,
            base_color_uvs: Some(uvs),
            ..Default::default()
        }
    }
}

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
