//! Minimal Bevy app that drives the sunray ray-tracer via `SunrayPlugin`.
//!
//! Run with:
//!     cargo run -p bevy_extension --example window -- path/to/scene.glb
//! to load a glTF file, or without arguments to spawn a procedural cube using
//! the `Renderer::create_mesh` API — useful to exercise the non-glTF path.

use std::{collections::HashSet, time::Instant};

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy_extension::{
    sunray::vulkan_abstraction, SunrayCamera, SunrayContext, SunrayMesh, SunrayMeshData,
    SunrayMaterial, SunrayPlugin, SunrayPluginConfig,
};

#[derive(Resource, Default)]
struct CameraState {
    yaw: f32,
    pitch: f32,
    keys_down: HashSet<KeyCode>,
    mouse_captured: bool,
    last_frame: Option<Instant>,
}

fn main() {
    let scene = std::env::args().nth(1);
    let demo_cube = scene.is_none();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SunrayPlugin {
            config: SunrayPluginConfig {
                initial_scene: scene,
                ..Default::default()
            },
        })
        .insert_resource(CameraState {
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
            ..Default::default()
        })
        .insert_resource(DemoCubeState {
            enabled: demo_cube,
            spawned: false,
        })
        .add_systems(Startup, spawn_camera)
        .add_systems(
            Update,
            (
                spawn_demo_cube,
                track_keys,
                track_mouse_buttons,
                fly_camera,
            )
                .chain(),
        )
        .run();
}

#[derive(Resource)]
struct DemoCubeState {
    enabled: bool,
    spawned: bool,
}

/// Builds a flat-shaded unit cube via `Renderer::create_mesh`, then spawns a
/// Bevy entity with `SunrayMesh` + `SunrayMaterial` so the plugin picks it up
/// on the next frame. Runs once, only when no glTF path was supplied.
fn spawn_demo_cube(
    mut state: ResMut<DemoCubeState>,
    ctx: Option<NonSendMut<SunrayContext>>,
    mut commands: Commands,
) {
    if !state.enabled || state.spawned {
        return;
    }
    let Some(mut ctx) = ctx else { return; };

    let mesh = cube_mesh();
    let handle = match ctx.renderer.create_mesh(&mesh) {
        Ok(h) => h,
        Err(e) => {
            error!("create_mesh failed: {e}");
            return;
        }
    };

    // Sunray's path tracer has no default sky / ambient light — the miss
    // shader returns zero emission. Give the cube a strong emissive factor so
    // direct hits are visible on screen. (Proper lit rendering of
    // non-emissive geometry additionally needs per-BLAS emissive triangle
    // registration, which `create_mesh` doesn't wire up yet.)
    let mut material = vulkan_abstraction::gltf::Material::default();
    material.pbr_metallic_roughness_properties.base_color_factor = [0.85, 0.25, 0.25, 1.0];
    material.pbr_metallic_roughness_properties.roughness_factor = 0.4;
    material.emissive_factor = [0.85, 0.25, 0.25];
    material.emissive_strength = 4.0;

    commands.spawn((
        SunrayMesh(handle),
        SunrayMaterial(material),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));

    info!("Spawned demo cube (MeshHandle = {:?})", handle);
    state.spawned = true;
}

/// Flat-shaded unit cube centered at the origin. Each face gets its own four
/// vertices so normals are constant per face.
fn cube_mesh() -> SunrayMeshData {
    const H: f32 = 0.5;
    let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
        ([-1.0, 0.0, 0.0], [[-H, -H, -H], [-H, -H,  H], [-H,  H,  H], [-H,  H, -H]]),
        ([ 1.0, 0.0, 0.0], [[ H, -H,  H], [ H, -H, -H], [ H,  H, -H], [ H,  H,  H]]),
        ([0.0, -1.0, 0.0], [[-H, -H, -H], [ H, -H, -H], [ H, -H,  H], [-H, -H,  H]]),
        ([0.0,  1.0, 0.0], [[-H,  H,  H], [ H,  H,  H], [ H,  H, -H], [-H,  H, -H]]),
        ([0.0, 0.0, -1.0], [[ H, -H, -H], [-H, -H, -H], [-H,  H, -H], [ H,  H, -H]]),
        ([0.0, 0.0,  1.0], [[-H, -H,  H], [ H, -H,  H], [ H,  H,  H], [-H,  H,  H]]),
    ];

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    for (face_idx, (normal, verts)) in faces.iter().enumerate() {
        let base = (face_idx * 4) as u32;
        for v in verts {
            positions.push(*v);
            normals.push(*normal);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    SunrayMeshData { positions, normals, indices, ..Default::default() }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        SunrayCamera { fov_y_degrees: 45.0 },
        Transform::from_xyz(0.0, 2.0, 10.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
    ));
}

fn track_keys(
    mut state: ResMut<CameraState>,
    mut keys: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut windows: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    use bevy::input::ButtonState;
    for ev in keys.read() {
        match ev.state {
            ButtonState::Pressed => {
                if ev.key_code == KeyCode::Escape {
                    state.mouse_captured = false;
                    if let Ok(mut cursor) = windows.single_mut() {
                        cursor.grab_mode = CursorGrabMode::None;
                        cursor.visible = true;
                    }
                } else {
                    state.keys_down.insert(ev.key_code);
                }
            }
            ButtonState::Released => {
                state.keys_down.remove(&ev.key_code);
            }
        }
    }
}

fn track_mouse_buttons(
    mut state: ResMut<CameraState>,
    mut buttons: MessageReader<bevy::input::mouse::MouseButtonInput>,
    mut windows: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    use bevy::input::{mouse::MouseButton, ButtonState};
    for ev in buttons.read() {
        if ev.state == ButtonState::Pressed && ev.button == MouseButton::Left {
            state.mouse_captured = true;
            if let Ok(mut cursor) = windows.single_mut() {
                cursor.grab_mode = CursorGrabMode::Confined;
                cursor.visible = false;
            }
        }
    }
}

fn fly_camera(
    mut state: ResMut<CameraState>,
    mut cams: Query<&mut Transform, With<SunrayCamera>>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
) {
    let now = Instant::now();
    let dt = state
        .last_frame
        .map(|t| now.duration_since(t).as_secs_f32())
        .unwrap_or(0.016);
    state.last_frame = Some(now);

    if state.mouse_captured {
        let sensitivity = 0.002f32;
        for ev in motion.read() {
            state.yaw += ev.delta.x * sensitivity;
            state.pitch -= ev.delta.y * sensitivity;
        }
        let limit = std::f32::consts::FRAC_PI_2 - 0.01;
        state.pitch = state.pitch.clamp(-limit, limit);
    } else {
        motion.clear();
    }

    let Ok(mut transform) = cams.single_mut() else { return; };

    // Build forward from yaw/pitch (yaw around +Y, pitch around +X).
    let forward = Vec3::new(
        state.yaw.cos() * state.pitch.cos(),
        state.pitch.sin(),
        state.yaw.sin() * state.pitch.cos(),
    )
    .normalize();
    let right = forward.cross(Vec3::Y).normalize();

    let speed = if state.keys_down.contains(&KeyCode::ShiftLeft) {
        9.0
    } else {
        3.0
    } * dt;

    if state.keys_down.contains(&KeyCode::KeyW) {
        transform.translation += forward * speed;
    }
    if state.keys_down.contains(&KeyCode::KeyS) {
        transform.translation -= forward * speed;
    }
    if state.keys_down.contains(&KeyCode::KeyD) {
        transform.translation += right * speed;
    }
    if state.keys_down.contains(&KeyCode::KeyA) {
        transform.translation -= right * speed;
    }
    if state.keys_down.contains(&KeyCode::Space) {
        transform.translation += Vec3::Y * speed;
    }
    if state.keys_down.contains(&KeyCode::ControlLeft) {
        transform.translation -= Vec3::Y * speed;
    }

    // Orient the transform so its -Z axis points along `forward`.
    let target = transform.translation + forward;
    transform.look_at(target, Vec3::Y);
}
