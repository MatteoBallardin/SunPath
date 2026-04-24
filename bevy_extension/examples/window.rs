//! Minimal Bevy app that drives the sunray ray-tracer via `SunrayPlugin`.
//!
//! Run with:
//!     cargo run -p bevy_extension --example window -- path/to/scene.glb
//! or, without a scene argument, the app opens the bundled demo scene.

use std::{collections::HashSet, time::Instant};

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy_extension::{SunrayCamera, SunrayPlugin, SunrayPluginConfig};

#[derive(Resource, Default)]
struct CameraState {
    yaw: f32,
    pitch: f32,
    keys_down: HashSet<KeyCode>,
    mouse_captured: bool,
    last_frame: Option<Instant>,
}

fn main() {
    // Sunray requires at least one glTF scene before `build_image_dependent_data`
    // can succeed (it pads textures up to NUMBER_OF_SAMPLERS = 1024).
    let scene = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/assets/Room.glb".to_string());

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SunrayPlugin {
            config: SunrayPluginConfig {
                initial_scene: Some(scene),
                ..Default::default()
            },
        })
        .insert_resource(CameraState {
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
            ..Default::default()
        })
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, (track_keys, track_mouse_buttons, fly_camera).chain())
        .run();
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
