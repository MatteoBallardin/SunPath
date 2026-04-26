//! Minimal Bevy app that drives the sunray ray-tracer via `SunrayPlugin`.
//!
//! Run with:
//!     cargo run -p bevy_extension --example window -- path/to/scene.glb
//! to load a glTF file, or without arguments to spawn a procedural cube using
//! the `Renderer::create_mesh` API — useful to exercise the non-glTF path.

use std::{collections::HashSet, time::Instant};

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use ash::vk;
use bevy_extension::{
    sunray::vulkan_abstraction, SunrayCamera, SunrayContext, SunrayMeshData, SunrayPbrBundle,
    SunrayPlugin, SunrayPluginConfig,
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

    // Upload a procedural 16×16 checkerboard to exercise `create_texture`.
    let checker_extent = (16u32, 16u32);
    let checker_bytes = checker_texture(checker_extent.0, checker_extent.1);
    let checker = match ctx
        .renderer
        .create_texture(checker_bytes, checker_extent, vk::Format::R8G8B8A8_UNORM)
    {
        Ok(h) => h,
        Err(e) => {
            error!("create_texture (checker) failed: {e}");
            return;
        }
    };
    info!("Uploaded checker texture at slot {}", checker.slot());

    // Non-emissive textured cube sitting on a matte plane, lit by a small
    // emissive sphere overhead. All three meshes come from the sunray
    // primitive builders.
    let cube_handle = match ctx.renderer.create_mesh(&SunrayMeshData::cube(0.5)) {
        Ok(h) => h,
        Err(e) => { error!("create_mesh (cube) failed: {e}"); return; }
    };
    let plane_handle = match ctx.renderer.create_mesh(&SunrayMeshData::plane(10.0)) {
        Ok(h) => h,
        Err(e) => { error!("create_mesh (plane) failed: {e}"); return; }
    };

    let light_emission = [12.0, 12.0, 12.0];
    let mut light_mesh = SunrayMeshData::sphere(0.3, 24, 16);
    light_mesh.emission = Some(light_emission);
    let light_handle = match ctx.renderer.create_mesh(&light_mesh) {
        Ok(h) => h,
        Err(e) => { error!("create_mesh (light) failed: {e}"); return; }
    };

    let cube_mat = vulkan_abstraction::Material {
        base_color_value: [1.0, 1.0, 1.0, 1.0],
        base_color_texture_index: checker.0,
        roughness_factor: 0.6,
        ..vulkan_abstraction::Material::default()
    };
    let plane_mat = vulkan_abstraction::Material {
        base_color_value: [0.8, 0.8, 0.8, 1.0],
        roughness_factor: 0.9,
        ..vulkan_abstraction::Material::default()
    };
    let light_mat = vulkan_abstraction::Material {
        base_color_value: [1.0, 1.0, 1.0, 1.0],
        emissive_factor: [1.0, 1.0, 1.0, 12.0],
        ..vulkan_abstraction::Material::default()
    };

    commands.spawn(
        SunrayPbrBundle::new(cube_handle)
            .with_material(cube_mat)
            .at(Transform::from_xyz(0.0, 1.0, 0.0)),
    );
    commands.spawn(
        SunrayPbrBundle::new(plane_handle)
            .with_material(plane_mat)
            .at(Transform::from_xyz(0.0, 0.5, 0.0)),
    );
    commands.spawn(
        SunrayPbrBundle::new(light_handle)
            .with_material(light_mat)
            .at(Transform::from_xyz(0.0, 3.5, 0.0)),
    );

    info!(
        "Spawned demo: cube={:?}, plane={:?}, light={:?}",
        cube_handle, plane_handle, light_handle
    );
    state.spawned = true;
}

/// Procedural 8×8-cell black/white checkerboard at the given resolution,
/// packed as RGBA8. Used by the demo as a `create_texture` input.
fn checker_texture(width: u32, height: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity((width * height * 4) as usize);
    let cells = 4u32;
    for y in 0..height {
        for x in 0..width {
            let cx = (x * cells) / width;
            let cy = (y * cells) / height;
            let on = (cx + cy) % 2 == 0;
            let c = if on { 230 } else { 40 };
            out.extend_from_slice(&[c, c, c, 255]);
        }
    }
    out
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
