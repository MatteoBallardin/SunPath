//! Bevy plugin exposing the sunray hardware ray-tracing renderer as a
//! replacement for `bevy_render`.
//!
//! The renderer holds `Rc<vulkan_abstraction::Core>` and is therefore `!Send`;
//! all its state lives in the [`SunrayContext`] `NonSend` resource and all
//! render systems run on the main thread, the same way `bevy_winit` handles
//! the winit `EventLoop`.
//!
//! Install after `WindowPlugin` + `WinitPlugin` (e.g. after `DefaultPlugins`,
//! built without the `bevy_render` feature):
//!
//! ```ignore
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(SunrayPlugin::default())
//!     .run();
//! ```

#![allow(clippy::too_many_arguments)]

pub mod surface;
pub mod swapchain;

use std::rc::Rc;

use ash::vk;
use bevy::math::Mat4;
use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;
use bevy::window::{PrimaryWindow, RawHandleWrapper, Window, WindowResized};
use nalgebra as na;

use sunray::{
    camera::Camera as SrCamera,
    error::{ErrorSource, SrResult},
    vulkan_abstraction, MeshHandle, Renderer, MAX_FRAMES_IN_FLIGHT,
};

pub use sunray;
pub use sunray::{MeshData as SunrayMeshData, MeshHandle as SunrayMeshHandle};

/// Plugin configuration. Stored as a Bevy `Resource` so it can be tweaked
/// between `add_plugins` and the first frame.
#[derive(Resource, Clone)]
pub struct SunrayPluginConfig {
    /// Optional glTF scene to load on renderer initialization. Sunray currently
    /// requires at least one loaded scene before rendering can start (it
    /// expects a fixed number of textures to be bound).
    pub initial_scene: Option<String>,
    /// Format of the raytracing output image.
    pub image_format: vk::Format,
    /// If `true`, spawn one Bevy entity per sunray entity produced by
    /// `initial_scene`, carrying `SunrayEntity(id)` + identity `Transform` /
    /// `GlobalTransform`. The glTF-original transforms remain in sunray; Bevy
    /// `Transform` starts at identity and is not pushed to sunray until the
    /// user mutates it (see [`sync_entity_transforms`]).
    pub auto_spawn_initial_scene: bool,
}

impl Default for SunrayPluginConfig {
    fn default() -> Self {
        Self {
            initial_scene: None,
            image_format: vk::Format::R8G8B8A8_SRGB,
            auto_spawn_initial_scene: true,
        }
    }
}

#[derive(Default)]
pub struct SunrayPlugin {
    pub config: SunrayPluginConfig,
}

impl Plugin for SunrayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .add_message::<LoadSunrayScene>()
            .add_message::<SunraySceneLoaded>()
            .add_message::<DespawnSunrayEntity>()
            // Scene commands run in Update so Bevy entities spawned as a side
            // effect are visible to the Last-chain render systems this frame.
            .add_systems(
                Update,
                (
                    try_init_sunray,
                    process_scene_events,
                    spawn_pending_sunray_entities,
                )
                    .chain(),
            )
            .add_systems(
                Last,
                (
                    handle_resize,
                    sync_camera,
                    sync_entity_transforms,
                    render_frame,
                )
                    .chain(),
            );
    }
}

/// Request a glTF load on the sunray renderer. A `SunraySceneLoaded` message
/// is emitted once loading completes, carrying the spawned Bevy entities.
#[derive(Message, Debug, Clone)]
pub struct LoadSunrayScene {
    pub path: String,
}

/// Emitted after a `LoadSunrayScene` is processed successfully.
#[derive(Message, Debug, Clone)]
pub struct SunraySceneLoaded {
    pub path: String,
    pub bevy_entities: Vec<Entity>,
}

/// Request removal of a sunray scene entity. The target Bevy entity must carry
/// a [`SunrayEntity`] component; on success both the sunray side entity and
/// the Bevy entity are despawned.
#[derive(Message, Debug, Clone, Copy)]
pub struct DespawnSunrayEntity {
    pub entity: Entity,
}

/// Component marking the active sunray camera. Position and orientation are
/// read from the entity's `GlobalTransform` (so Bevy's transform hierarchy
/// drives the camera). If multiple are present, the first encountered wins.
#[derive(Component, Clone, Copy, Debug)]
pub struct SunrayCamera {
    pub fov_y_degrees: f32,
}

impl Default for SunrayCamera {
    fn default() -> Self {
        Self { fov_y_degrees: 45.0 }
    }
}

/// Links a Bevy entity to a sunray scene entity. Mutate the entity's
/// `Transform`; its `GlobalTransform` will be pushed to sunray on change.
#[derive(Component, Clone, Copy, Debug)]
pub struct SunrayEntity(pub vulkan_abstraction::EntityId);

/// Reference to a mesh uploaded to sunray. Obtain a `MeshHandle` by calling
/// `ctx.renderer.create_mesh(&data)` from a system with
/// `NonSendMut<SunrayContext>` access. Pair this with a `SunrayMaterial` and
/// a `GlobalTransform` on a Bevy entity; the plugin will call `create_entity`
/// next frame and insert a `SunrayEntity` component alongside it.
#[derive(Component, Clone, Copy, Debug)]
pub struct SunrayMesh(pub MeshHandle);

/// PBR material for a `SunrayMesh`. Read once at `create_entity` time; later
/// mutations are ignored (material updates require reuploading the entity's
/// GPU data, which isn't wired yet).
#[derive(Component, Clone, Debug)]
pub struct SunrayMaterial(pub vulkan_abstraction::gltf::Material);

impl Default for SunrayMaterial {
    fn default() -> Self {
        Self(vulkan_abstraction::gltf::Material::default())
    }
}

/// `NonSend` resource that owns every `!Send` piece of Vulkan state. Built on
/// the first `Update` where a primary window with a valid `RawHandleWrapper`
/// is available.
///
/// Field order matters — Rust drops struct fields top-to-bottom, and the
/// Vulkan spec requires the `VkSwapchainKHR` to be destroyed *before* its
/// associated `VkSurfaceKHR` (VUID-vkDestroySurfaceKHR-surface-01266).
/// So: renderer first (pipelines / image-dependent cmd buffers) → per-frame
/// sync primitives → swapchain → surface last.
pub struct SunrayContext {
    pub renderer: Renderer,
    pub img_acquired_sems: Vec<vulkan_abstraction::Semaphore>,
    pub img_rendered_fences: Vec<vk::Fence>,
    pub ready_to_present_sems: Vec<vulkan_abstraction::Semaphore>,
    pub img_barrier_to_present_cmd_bufs: Vec<vulkan_abstraction::CmdBuffer>,
    pub swapchain: swapchain::Swapchain,
    pub surface: surface::Surface,
    pub frame_index: usize,
    pub current_extent: (u32, u32),
    /// Entity IDs produced by the `initial_scene` glTF load, preserved for
    /// users who set `auto_spawn_initial_scene = false` and want to attach
    /// `SunrayEntity` components to their own Bevy entities.
    pub initial_scene_entities: Vec<vulkan_abstraction::EntityId>,
}

impl Drop for SunrayContext {
    fn drop(&mut self) {
        // Wait for the GPU to finish all in-flight work before any child
        // Vulkan handle (swapchain images, command buffers, semaphores,
        // descriptor sets) is torn down by its own `Drop`. Without this, a
        // minimized-then-closed window produces VUID-vkDestroyDevice warnings
        // because the driver is still reading from resources we're about to
        // free.
        unsafe {
            if let Err(e) = self.renderer.core().device().inner().device_wait_idle() {
                error!("device_wait_idle on shutdown failed: {e:?}");
            }
        }
    }
}

fn try_init_sunray(world: &mut World) {
    if world.get_non_send::<SunrayContext>().is_some() {
        return;
    }
    let (size, handle) = {
        let mut q = world.query_filtered::<(&Window, &RawHandleWrapper), With<PrimaryWindow>>();
        let Some((window, handle)) = q.iter(world).next() else { return; };
        let size = (window.physical_width(), window.physical_height());
        if size.0 == 0 || size.1 == 0 {
            return;
        }
        (size, handle.clone())
    };

    let config = world.resource::<SunrayPluginConfig>().clone();

    let ctx = match build_context(handle, size, &config) {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("Sunray init failed: {e}");
            return;
        }
    };
    info!("Sunray renderer initialized at {}x{}", size.0, size.1);

    let scene_ids = ctx.initial_scene_entities.clone();
    // Read the glTF-original transforms back from sunray so spawned Bevy
    // entities start out with `Transform` matching what the renderer is
    // actually drawing — no "identity placeholder" frame.
    let initial_transforms: Vec<Transform> = if config.auto_spawn_initial_scene {
        scene_ids
            .iter()
            .map(|id| {
                ctx.renderer
                    .get_entity_transform(*id)
                    .map(na_mat4_to_transform)
                    .unwrap_or_default()
            })
            .collect()
    } else {
        Vec::new()
    };
    world.insert_non_send(ctx);

    if config.auto_spawn_initial_scene {
        for (id, transform) in scene_ids.into_iter().zip(initial_transforms) {
            world.spawn((SunrayEntity(id), transform));
        }
    }
}

fn process_scene_events(
    ctx: Option<NonSendMut<SunrayContext>>,
    mut load_events: MessageReader<LoadSunrayScene>,
    mut despawn_events: MessageReader<DespawnSunrayEntity>,
    mut loaded_events: MessageWriter<SunraySceneLoaded>,
    mut commands: Commands,
    sunray_entities: Query<&SunrayEntity>,
) {
    let Some(mut ctx) = ctx else {
        // Drain so these events don't pile up until the renderer is ready.
        load_events.clear();
        despawn_events.clear();
        return;
    };

    for ev in load_events.read() {
        match ctx.renderer.load_gltf(&ev.path) {
            Ok(ids) => {
                info!("Loaded {} entities from '{}'", ids.len(), ev.path);
                invalidate_stale_fences(&mut ctx);
                let bevy_entities: Vec<Entity> = ids
                    .into_iter()
                    .map(|id| {
                        let transform = ctx
                            .renderer
                            .get_entity_transform(id)
                            .map(na_mat4_to_transform)
                            .unwrap_or_default();
                        commands.spawn((SunrayEntity(id), transform)).id()
                    })
                    .collect();
                loaded_events.write(SunraySceneLoaded {
                    path: ev.path.clone(),
                    bevy_entities,
                });
            }
            Err(e) => error!("LoadSunrayScene('{}'): {e}", ev.path),
        }
    }

    for ev in despawn_events.read() {
        let Ok(&SunrayEntity(id)) = sunray_entities.get(ev.entity) else {
            continue;
        };
        if let Err(e) = ctx.renderer.destroy_entity(id) {
            error!("destroy_entity({:?}) failed: {e}", id);
            continue;
        }
        invalidate_stale_fences(&mut ctx);
        commands.entity(ev.entity).despawn();
    }
}

/// `load_gltf` / `destroy_entity` / `duplicate_entity` on the sunray renderer
/// all call `clear_image_dependent_data` internally, which drops the
/// `CmdBuffer`s whose `VkFence` handles we stashed in `img_rendered_fences`.
/// Null them out so next frame's `wait_fence` skips the stale handles.
fn invalidate_stale_fences(ctx: &mut SunrayContext) {
    for f in ctx.img_rendered_fences.iter_mut() {
        *f = vk::Fence::null();
    }
}

/// Turn Bevy entities carrying `SunrayMesh` + `SunrayMaterial` (but no
/// `SunrayEntity` yet) into live sunray entities. Reads the entity's current
/// `GlobalTransform` as the initial pose; subsequent transform changes are
/// picked up by `sync_entity_transforms`.
fn spawn_pending_sunray_entities(
    ctx: Option<NonSendMut<SunrayContext>>,
    pending: Query<
        (Entity, &SunrayMesh, &SunrayMaterial, &GlobalTransform),
        Without<SunrayEntity>,
    >,
    mut commands: Commands,
) {
    let Some(mut ctx) = ctx else { return; };
    let mut created_any = false;
    for (entity, mesh, material, gt) in &pending {
        let transform = mat4_to_na(gt.to_matrix());
        match ctx.renderer.create_entity(mesh.0, &material.0, transform) {
            Ok(id) => {
                commands.entity(entity).insert(SunrayEntity(id));
                created_any = true;
            }
            Err(e) => error!("create_entity for Bevy entity {entity:?} failed: {e}"),
        }
    }
    if created_any {
        invalidate_stale_fences(&mut ctx);
    }
}

fn build_context(
    handle: RawHandleWrapper,
    size: (u32, u32),
    config: &SunrayPluginConfig,
) -> SrResult<SunrayContext> {
    let display_handle = handle.get_display_handle();
    let window_handle = handle.get_window_handle();

    let instance_exts = surface::enumerate_required_extensions(display_handle)?;

    let create_surface = move |entry: &ash::Entry, instance: &ash::Instance| -> SrResult<vk::SurfaceKHR> {
        surface::create_surface(entry, instance, display_handle, window_handle)
    };

    let (mut renderer, surface_handle) =
        Renderer::new_with_surface(size, config.image_format, instance_exts, &create_surface)?;

    let initial_scene_entities = if let Some(path) = &config.initial_scene {
        match renderer.load_gltf(path) {
            Ok(ids) => {
                info!("Loaded {} entities from '{}'", ids.len(), path);
                ids
            }
            Err(e) => {
                error!("Failed to load initial scene '{}': {e}", path);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let surface = surface::Surface::new(
        renderer.core().entry(),
        renderer.core().instance(),
        surface_handle,
    );
    let swapchain =
        swapchain::Swapchain::new(Rc::clone(renderer.core()), surface.handle(), size)?;

    renderer.build_image_dependent_data(swapchain.images())?;

    let core = Rc::clone(renderer.core());

    let img_barrier_to_present_cmd_bufs = build_present_barriers(&core, swapchain.images())?;
    let img_acquired_sems = (0..MAX_FRAMES_IN_FLIGHT)
        .map(|_| vulkan_abstraction::Semaphore::new(Rc::clone(&core)))
        .collect::<Result<Vec<_>, _>>()?;
    let img_rendered_fences = vec![vk::Fence::null(); MAX_FRAMES_IN_FLIGHT];
    let ready_to_present_sems = swapchain
        .images()
        .iter()
        .map(|_| vulkan_abstraction::Semaphore::new(Rc::clone(&core)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SunrayContext {
        renderer,
        surface,
        swapchain,
        img_acquired_sems,
        img_rendered_fences,
        ready_to_present_sems,
        img_barrier_to_present_cmd_bufs,
        frame_index: 0,
        current_extent: size,
        initial_scene_entities,
    })
}

fn build_present_barriers(
    core: &Rc<vulkan_abstraction::Core>,
    images: &[vk::Image],
) -> SrResult<Vec<vulkan_abstraction::CmdBuffer>> {
    images
        .iter()
        .map(|image| -> SrResult<vulkan_abstraction::CmdBuffer> {
            let cmd_buf = vulkan_abstraction::CmdBuffer::new(Rc::clone(core))?;
            unsafe {
                let begin = vk::CommandBufferBeginInfo::default();
                core.device()
                    .inner()
                    .begin_command_buffer(cmd_buf.inner(), &begin)?;
                vulkan_abstraction::cmd_image_memory_barrier(
                    core,
                    cmd_buf.inner(),
                    *image,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::AccessFlags::empty(),
                    vk::ImageLayout::GENERAL,
                    vk::ImageLayout::PRESENT_SRC_KHR,
                );
                core.device().inner().end_command_buffer(cmd_buf.inner())?;
            }
            Ok(cmd_buf)
        })
        .collect()
}

fn handle_resize(
    ctx: Option<NonSendMut<SunrayContext>>,
    mut resized: MessageReader<WindowResized>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(mut ctx) = ctx else { return; };
    if resized.read().next().is_none() {
        return;
    }
    let Ok(window) = windows.single() else { return; };
    let new_size = (window.physical_width(), window.physical_height());
    if new_size.0 == 0 || new_size.1 == 0 || new_size == ctx.current_extent {
        return;
    }
    if let Err(e) = rebuild_for_size(&mut ctx, new_size) {
        error!("Sunray resize failed: {e}");
    }
}

fn rebuild_for_size(ctx: &mut SunrayContext, new_size: (u32, u32)) -> SrResult<()> {
    ctx.renderer.resize(new_size)?;
    ctx.renderer
        .core()
        .device()
        .update_surface_support_details(ctx.surface.handle(), ctx.surface.instance_fn());

    let actual = swapchain::Swapchain::get_extent(
        new_size,
        &ctx.renderer.core().device().surface_support_details(),
    );
    let cur = ctx.swapchain.extent();
    if (actual.width, actual.height) == (cur.width, cur.height) {
        ctx.current_extent = new_size;
        return Ok(());
    }

    for f in ctx.img_rendered_fences.iter_mut() {
        *f = vk::Fence::null();
    }

    ctx.swapchain.rebuild(ctx.surface.handle(), new_size)?;
    let core = Rc::clone(ctx.renderer.core());
    ctx.img_barrier_to_present_cmd_bufs = build_present_barriers(&core, ctx.swapchain.images())?;
    ctx.current_extent = new_size;
    Ok(())
}

fn sync_camera(
    ctx: Option<NonSendMut<SunrayContext>>,
    q: Query<(&SunrayCamera, &GlobalTransform)>,
) {
    let Some(mut ctx) = ctx else { return; };
    let Some((cam, gt)) = q.iter().next() else { return; };

    let pos = gt.translation();
    // In Bevy, `GlobalTransform::forward()` returns the entity's local -Z in
    // world space, which matches the convention used by Bevy's own Camera.
    let fwd = *gt.forward();

    let position = na::point![pos.x, pos.y, pos.z];
    let target = na::point![pos.x + fwd.x, pos.y + fwd.y, pos.z + fwd.z];
    let sr_camera = SrCamera::new(position, target, cam.fov_y_degrees);
    if let Err(e) = ctx.renderer.set_camera(sr_camera) {
        error!("set_camera failed: {e}");
    }
}

fn sync_entity_transforms(
    ctx: Option<NonSendMut<SunrayContext>>,
    q: Query<(&SunrayEntity, Ref<GlobalTransform>)>,
) {
    let Some(mut ctx) = ctx else { return; };
    let mut any = false;
    for (se, gt) in &q {
        // `Changed<T>` fires on `Added` too; skip the initial fire since the
        // spawn-time transform was read out of sunray in the first place, so
        // pushing it back is a no-op upload.
        if !gt.is_changed() || gt.is_added() {
            continue;
        }
        let mat = mat4_to_na(gt.to_matrix());
        if let Err(err) = ctx.renderer.set_entity_transform(se.0, mat) {
            error!("set_entity_transform failed: {err}");
        } else {
            any = true;
        }
    }
    if any {
        if let Err(e) = ctx.renderer.rebuild_tlas() {
            error!("rebuild_tlas failed: {e}");
        }
    }
}

fn render_frame(
    ctx: Option<NonSendMut<SunrayContext>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(mut ctx) = ctx else { return; };
    match draw_one(&mut ctx) {
        Ok(()) => {}
        Err(e) => {
            if matches!(
                e.get_source(),
                ErrorSource::Vulkan(vk::Result::ERROR_OUT_OF_DATE_KHR)
                    | ErrorSource::Vulkan(vk::Result::SUBOPTIMAL_KHR)
            ) {
                // The swapchain no longer matches the surface (resize, minimize
                // restore, DPI change). Rebuild against the current window size
                // and skip the rest of this frame — the next frame will see a
                // fresh swapchain.
                warn!("swapchain out of date, rebuilding");
                let Ok(window) = windows.single() else { return; };
                let size = (window.physical_width(), window.physical_height());
                if size.0 == 0 || size.1 == 0 {
                    return;
                }
                if let Err(e) = rebuild_for_size(&mut ctx, size) {
                    error!("rebuild after OUT_OF_DATE failed: {e}");
                }
            } else {
                error!("render_frame: {e}");
            }
        }
    }
}

fn draw_one(ctx: &mut SunrayContext) -> SrResult<()> {
    let frame_index = ctx.frame_index % MAX_FRAMES_IN_FLIGHT;
    let img_acquired_sem = ctx.img_acquired_sems[frame_index].inner();
    let img_rendered_fence = ctx.img_rendered_fences[frame_index];
    vulkan_abstraction::wait_fence(ctx.renderer.core().device(), img_rendered_fence)?;

    let (img_index, suboptimal) = unsafe {
        ctx.swapchain.device().acquire_next_image(
            ctx.swapchain.inner(),
            u64::MAX,
            img_acquired_sem,
            vk::Fence::null(),
        )
    }?;
    if suboptimal {
        warn!("Swapchain suboptimal for surface");
    }
    let img_index = img_index as usize;

    let swapchain_image = ctx.swapchain.images()[img_index];
    ctx.img_rendered_fences[frame_index] = ctx
        .renderer
        .render_to_image(swapchain_image, img_acquired_sem)?;

    let cmd_buf = &mut ctx.img_barrier_to_present_cmd_bufs[img_index];
    let done_fence = cmd_buf.fence_mut().submit()?;
    let cmd_buf_inner = cmd_buf.inner();
    let ready_sem = ctx.ready_to_present_sems[img_index].inner();

    ctx.renderer.core().graphics_queue().submit_async(
        cmd_buf_inner,
        &[],
        &[],
        &[ready_sem],
        done_fence,
    )?;

    let swapchains = [ctx.swapchain.inner()];
    let image_indices = [img_index as u32];
    let wait_semaphores = [ready_sem];
    let present_info = vk::PresentInfoKHR::default()
        .wait_semaphores(&wait_semaphores)
        .swapchains(&swapchains)
        .image_indices(&image_indices);

    unsafe {
        ctx.swapchain
            .device()
            .queue_present(ctx.renderer.core().graphics_queue().inner(), &present_info)
    }?;

    ctx.frame_index = ctx.frame_index.wrapping_add(1);
    Ok(())
}

/// glam (column-major) → nalgebra (column-major) — memory layouts match.
fn mat4_to_na(m: Mat4) -> na::Matrix4<f32> {
    na::Matrix4::from_column_slice(&m.to_cols_array())
}

/// nalgebra `Matrix4` → Bevy `Transform`. Indexes element-by-element to avoid
/// depending on nalgebra's internal storage layout.
fn na_mat4_to_transform(m: na::Matrix4<f32>) -> Transform {
    let mut cols = [0f32; 16];
    for c in 0..4 {
        for r in 0..4 {
            cols[c * 4 + r] = m[(r, c)];
        }
    }
    Transform::from_matrix(Mat4::from_cols_array(&cols))
}
