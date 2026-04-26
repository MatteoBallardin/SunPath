# SunPath (Sunray V2)

Rust hardware real time path-tracing library

This project was developed by [Riccardo-Finello](https://github.com/riccardoFinelloUniTn) supervised by Professor [Marco Patrignani](https://squera.github.io/) for the bachelor thesis at the University of Trento, Italy
<br>
It's based on the [sunray](https://github.com/kalsifer-742/sunray) project developed by [kalsifer-742](https://github.com/kalsifer-742) and [circled-square](https://github.com/circled-square)
## Contribution

If you wish to contribute to the project you may check our issues, or if you found a bug or missing feature feel free to create one. 
You may also contact us at the e-mail addresses linked to our GitHub accounts.

If you're studying at University of Trento and are looking for a thesis subject you can ask Professor Marco Patrignani 
to be your supervisor to work on this project and we will be available if you need help or clarifications.

Thesis proposals:
- Library Integration https://github.com/kalsifer-742/sunray/issues/52

## Comparison

|                                                                      | Active project | Non-trivial | Real-time | Fully ray-traced | Hybrid |  GPU  | HW RT | Compute | SIMD  |  BVH  | Mesh  | Materials | Denoise | Rust  | Crate |  Engine   |                         Notes |
|:---------------------------------------------------------------------| :------------: | :---------: | :-------: | :--------------: |:------:| :---: | :---: | :-----: | :---: | :---: | :---: | :-------: |:-------:| :---: | :---: | :-------: | ----------------------------: |
| [Kajiya](https://github.com/EmbarkStudios/kajiya)                    |       ❌        |      ✅      |     ✅     |        ✅         |   ✅    |   ✅   |   ✅   |    ✅    |   ❌   |   ?   |   ✅   |     ✅     |    ✅    |   ✅   |   ❌   |     ❌     |                               |
| [Cycles](https://projects.blender.org/blender/cycles)                |       ✅        |      ✅      |     ❌     |        ✅         |   ❌    |   ✅   |   ✅   |    ✅    |   ✅   |   ✅   |   ✅   |     ✅     |    ✅    |   ❌   |  N/A  | ✅ Blender |                               |
| [manta-ray](https://github.com/ange-yaghi/manta-ray)                 |       ❌        |      ✅      |     ❌     |        ✅         |   ❌    |   ✅   |   ❌   |    ✅    |   ✅   |   ✅   |   ✅   |     ✅     |    ✅    |   ❌   |  N/A  | ✅ Blender |                               |
| [luxcore](https://luxcorerender.org/)                                |       ✅        |      ✅      |     ❌     |        ?         |   ?    |   ✅   |   ❌   |    ✅    |   ?   |   ?   |   ✅   |     ✅     |    ?    |   ❌   |  N/A  | ✅ Blender |                               |
| [akari_render](https://github.com/shiinamiyuki/akari_render)         |       ❌        |      ✅      |     ?     |        ?         |   ?    |   ✅   |   ❌   |    ✅    |   ?   |   ?   |   ✅   |     ✅     |    ?    |   ✅   |   ❌   | ✅ Blender |    Rebuild blender to install |
| [KaminariOS/rustracer](https://github.com/KaminariOS/rustracer)      |       ❌        |      ✅      |     ❌     |        ✅         |   ❌    |   ✅   |   ✅   |    ❌    |   ❌   |   ❌   |   ✅   |     ✅     |    ❌    |   ✅   |   ❌   |     ❌     |                      uses Nix |
| [RayTracingInVulkan](https://github.com/GPSnoopy/RayTracingInVulkan) |       ✅        |      ✅      |     ✅     |        ✅         |   ❌    |   ✅   |   ✅   |    ❌    |   ?   |   ✅   |   ✅   |  partial  |    ❌    |   ❌   |  N/A  |     ❌     |                               |
| [referencePT](https://github.com/boksajak/referencePT)               |       ❌        |      ✅      |     ?     |        ?         |   ?    |   ✅   |   ✅   |    ❌    |   ❌   |   ?   |   ✅   |     ✅     |    ?    |   ❌   |  N/A  |     ❌     |                               |
| [gbrt](https://github.com/giulianbiolo/gbrt)                         |       ❌        |      ❌      |     ❌     |        ❌         |   ❌    |   ❌   |   ❌   |    ❌    |   ✅   |   ✅   |   ✅   |     ❌     |    ❌    |   ✅   |   ❌   |     ❌     |                               |
| [Godot4-RayTracing](https://github.com/bitegw/Godot4-Raytracing)     |       ❌        |      ❌      |     ✅     |        ✅         |   ❌    |   ✅   |   ❌   |    ✅    |   ❌   |   ❌   |   ❌   |  partial  |    ❌    |   ❌   |  N/A  |  ✅ Godot  |                               |
| [Raytracing_Godot4](https://github.com/nekotogd/Raytracing_Godot4)   |       ❌        |      ❌      |     ✅     |        ✅         |   ❌    |   ✅   |   ❌   |    ✅    |   ❌   |   ❌   |   ❌   |     ❌     |    ❌    |   ❌   |  N/A  |  ✅ Godot  |                               |
| [bevyray](https://github.com/GrandmasterB42/bevyray)                 |       ✅        |      ❌      |     ✅     |        ❌         |   ✅    |   ✅   |   ❌   |    ❌    |   ❌   |   ✅   |   ❌   |  partial  |    ❌    |   ✅   |   ❌   |  ✅ Bevy   | raytracing in fragment shader |
| [hanamaru-renderer](https://github.com/gam0022/hanamaru-renderer)    |       ❌        |      ❌      |     ❌     |        ?         |   ?    |   ❌   |   ❌   |    ❌    |   ?   |   ✅   |   ✅   |     ✅     |    ✅    |   ✅   |   ❌   |     ❌     |          docs are in japanese |
| [rtwlib](https://crates.io/crates/rtwlib)                            |       ✅        |      ❌      |     ❌     |        ✅         |   ❌    |   ❌   |   ❌   |    ❌    |   ❌   |   ❌   |   ❌   |     ❌     |    ❌    |   ✅   |   ✅   |     ❌     |                               |
| [rustic-zen](https://crates.io/crates/rustic-zen)                    |       ❌        |      ❌      |     ✅     |        ?         |   ?    |   ❌   |   ❌   |    ❌    |   ?   |   ?   |   ?   |     ?     |    ?    |   ✅   |   ✅   |     ❌     |                            2D |
| [andros21/rustracer](https://crates.io/crates/rustracer)             |       ❌        |      ❌      |     ❌     |        ✅         |   ❌    |   ❌   |   ❌   |    ❌    |   ❌   |   ❌   |   ❌   |     ❌     |    ❌    |   ✅   |   ✅   |     ❌     |                               |
|                                                                      |                |             |           |                  |        |       |       |         |       |       |       |           |         |       |       |           |                               |
| [sunray](https://github.com/Kalsifer-742/sunray)                     |       ✅        |      ✅      |     ✅     |        ✅         |   ❌    |   ✅   |   ✅   |    ❌    |   ❌   |   ✅   |   ✅   |  partial  |    ❌    |   ✅   |   ✅   |     ❌     |                               |
| [sunPath](https://github.com/Kalsifer-742/sunray)                    |       ✅        |      ✅      |     ✅     |        ✅         |    ✅    |   ✅   |   ✅   |    ❌    |   ❌   |   ✅   |   ✅   |  partial  |    ✅     |   ✅   |   ✅   |     ❌     |                               |

## Resources

### General

- [Nvidia tutorial on vulkan KHR raytracing](https://nvpro-samples.github.io/vk_raytracing_tutorial_KHR/)
- [SaschaWillems basic ray tracing tutorial (C++)](https://github.com/SaschaWillems/Vulkan/blob/master/examples/raytracingbasic/raytracingbasic.cpp)
- [SaschaWillems vulkan tutorials (C++)](https://github.com/SaschaWillems/Vulkan)
- [Khronos vulkan samples (C++)](https://github.com/KhronosGroup/Vulkan-Samples/tree/main)
- [Ray Tracing in One Weekend - series](https://raytracing.github.io/)
- #### Other projects
  - [hatoo/ash-raytracing-example (Rust)](https://github.com/hatoo/ash-raytracing-example)
  - [adrien-ben/vulkan-examples-rs (Rust)](https://github.com/adrien-ben/vulkan-examples-rs)

### Rendering
- [Ray Tracing Gems II](https://developer.nvidia.com/ray-tracing-gems-ii)
- [pbrt](https://pbrt.org/)
  - [book](https://pbr-book.org/)
- [PBR for materials](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.pdf)
  - page 197 - appendix B: BRDF Implementation
- #### Shaders
  - https://www.gsn-lib.org/docs/nodes/raytracing.php
  - ##### Languages
    - [shader languages comparisons](https://alain.xyz/blog/a-review-of-shader-languages)
    - [slang](https://shader-slang.org/)
  - ##### Compilation
    - https://github.com/google/shaderc-rs

### Acceleration structure
- see [this nvidia blog](https://developer.nvidia.com/blog/best-practices-using-nvidia-rtx-ray-tracing/) for best practices for acceleration structures (and hit shading)

### glTF
- [2.0 reference guide pdf](https://www.khronos.org/files/gltf20-reference-guide.pdf)
- [2.0 spec](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.pdf)
- [khronos tutorials](https://github.com/KhronosGroup/glTF-Tutorials/tree/main)
- https://www.gltfeditor.com/
- #### Extensions
  - [KHR_lights_punctual](https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_lights_punctual/README.md)
- #### Models
  - [Khronos sample assets](https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main)
  - [Lantern](https://github.com/KhronosGroup/glTF-Sample-Assets/blob/main/Models/Lantern/README.md)

### Performance
- https://zeux.io/2020/02/27/writing-an-efficient-vulkan-renderer/
  - #### Syncronization
    - https://themaister.net/blog/2019/08/14/yet-another-blog-explaining-vulkan-synchronization/
    - https://xanderbert.github.io/2025/04/13/VulkanMemoryBarriers.html
    - https://cpp-rendering.io/barriers-vulkan-not-difficult/
    - [gpuopne - vulkan barriers explained](https://gpuopen.com/learn/vulkan-barriers-explained/)
    - [khr blog on image layout](https://www.khronos.org/blog/so-long-image-layouts-simplifying-vulkan-synchronisation)
  - #### Memory allocation
    - https://blog.io7m.com/2023/11/11/vulkan-memory-allocation.xhtml
    - https://github.com/Traverse-Research/gpu-allocator
    - https://github.com/gwihlidal/vk-mem-rs
    - https://docs.vulkan.org/guide/latest/memory_allocation.html
  - #### Queues
    - https://gpuopen.com/learn/concurrent-execution-asynchronous-queues/

### Miscelleaneus
- [graphics APIs](https://github.com/Vincent-Therrien/gpu-arena)
- [Semantic Versioning 2.0.0](https://semver.org/)
- #### Coordinate Systems
  - [nalgebra computer-graphics recipes](https://nalgebra.rs/docs/user_guide/cg_recipes)
  - https://learnopengl.com/Getting-started/Coordinate-Systems
- #### Rust
  - https://doc.rust-lang.org/book/
  - https://doc.rust-lang.org/rust-by-example/
- #### Vulkan
  - [docs](https://docs.vulkan.org/guide/latest/index.html)
  - [tutorial](https://docs.vulkan.org/tutorial/latest/00_Introduction.html)
  - [unofficaila tutorial](https://vulkan-tutorial.com/)
  - [paminerva tutorial](https://paminerva.github.io/docs/LearnVulkan/LearnVulkan)




Files
- bevy_extension/src/lib.rs:1 — SunrayPlugin, SunrayPluginConfig, SunrayCamera/SunrayEntity components, SunrayContext NonSend resource, four systems.
- bevy_extension/src/surface.rs:1 — ash + raw-window-handle 0.6 surface creation (the example's helpers were pinned to rwh 0.5).
- bevy_extension/src/swapchain.rs:1 — swapchain wrapper ported from examples/window/swapchain.rs.
- bevy_extension/examples/window.rs:1 — Bevy-driven fly camera demo: cargo run -p bevy_extension --example window -- path/to/scene.glb.
- bevy_extension/src/main.rs removed so the crate is pure library.

System wiring (lib.rs:63)
- Update: try_init_sunray — exclusive &mut World system, waits until a PrimaryWindow has a RawHandleWrapper + non-zero size, then builds SunrayContext and inserts it as NonSend.
- Last.chain(): handle_resize → sync_camera → sync_entity_transforms → render_frame. All take Option<NonSendMut<SunrayContext>> so they no-op before init. sync_entity_transforms uses Changed<SunrayEntity> + a single rebuild_tlas() per frame.

Fixes caught during build
- World::get_non_send / insert_non_send (the _resource variants are deprecated in this Bevy).
- CreateSurfaceFn is dyn Fn(...) with implicit 'static, so the closure needs move (handles are Copy).

Known loose ends to decide on next
- SunrayCamera is a standalone struct — not hooked up to bevy::transform::Transform. If you want Bevy-native camera transforms, I can swap it to read GlobalTransform instead.
- initial_scene entity IDs are exposed via SunrayContext.initial_scene_entities but not auto-wrapped in ECS entities. A follow-up system could spawn Bevy entities carrying SunrayEntity(id) so you can drive per-entity transforms from ECS.
- No runtime load_gltf / destroy_entity API yet — currently only initial_scene and per-entity transform sync. Easy to add a Commands-style wrapper when needed.

Both done. Summary of changes:

Example crash fix (examples/window.rs:26)
- Defaults to examples/assets/Room.glb when no CLI arg — sunray's build_image_dependent_data asserts textures are padded to 1024, which only happens after a glTF load.

Transform integration (lib.rs)
- SunrayCamera is now just { fov_y_degrees: f32 } — position/orientation read from GlobalTransform (sync_camera uses gt.translation() + gt.forward()).
- SunrayEntity is now SunrayEntity(EntityId) — a marker. Transforms come from the entity's GlobalTransform via a new mat4_to_na helper (glam and nalgebra are both column-major so from_column_slice(&m.to_cols_array()) is zero-loss).
- Example driven entirely through Transform::look_at / transform.translation.

Auto-spawn (lib.rs:144)
- New SunrayPluginConfig::auto_spawn_initial_scene (default true). After the renderer loads the initial glTF, try_init_sunray spawns one Bevy entity per sunray entity with (SunrayEntity(id), Transform::default()).
- sync_entity_transforms uses Ref<GlobalTransform> and skips frames where is_added() is true, so the glTF-loaded transforms remain authoritative until the user actively mutates Bevy Transform.

Known tradeoff on auto-spawn: on frame 1, Bevy Transform reads as identity while sunray still renders each entity at its glTF-loaded pose. As soon as the user mutates a Transform, Bevy wins and sunray is overwritten. A fully bidirectional sync would need a
public sunray API to read per-entity transforms — worth adding if you want ECS to reflect the glTF layout from frame 1. Say the word and I'll add Renderer::get_entity_transform upstream and wire it through.

Builds clean (cargo check -p bevy_extension --example window passes).

✻ Cogitated for 5m 41s

────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
❯                                                                                                                                                                                                                                                                   
────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
⏵⏵ accept edits on (shift+tab to cycle)                                                                                                                                                                                                           ⧉ In window.rs

        --> src\vulkan_abstraction\buffer\arena_core.rs:124:12
         |
     … +57 lines (ctrl+o to expand)

● Update(bevy_extension\examples\window.rs)
⎿  Added 1 line, removed 1 line
7  use std::{collections::HashSet, time::Instant};
8  
9  use bevy::prelude::*;
10 -use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, Window};                                                                                                                                                                          
10 +use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};                                                                                                                                                                                  
11  use bevy_extension::{SunrayCamera, SunrayPlugin, SunrayPluginConfig};
12  
13  #[derive(Resource, Default)]

● Both done. Summary of changes:

Example crash fix (examples/window.rs:26)
- Defaults to examples/assets/Room.glb when no CLI arg — sunray's build_image_dependent_data asserts textures are padded to 1024, which only happens after a glTF load.

Transform integration (lib.rs)
- SunrayCamera is now just { fov_y_degrees: f32 } — position/orientation read from GlobalTransform (sync_camera uses gt.translation() + gt.forward()).
- SunrayEntity is now SunrayEntity(EntityId) — a marker. Transforms come from the entity's GlobalTransform via a new mat4_to_na helper (glam and nalgebra are both column-major so from_column_slice(&m.to_cols_array()) is zero-loss).
- Example driven entirely through Transform::look_at / transform.translation.

Auto-spawn (lib.rs:144)
- New SunrayPluginConfig::auto_spawn_initial_scene (default true). After the renderer loads the initial glTF, try_init_sunray spawns one Bevy entity per sunray entity with (SunrayEntity(id), Transform::default()).
- sync_entity_transforms uses Ref<GlobalTransform> and skips frames where is_added() is true, so the glTF-loaded transforms remain authoritative until the user actively mutates Bevy Transform.

Known tradeoff on auto-spawn: on frame 1, Bevy Transform reads as identity while sunray still renders each entity at its glTF-loaded pose. As soon as the user mutates a Transform, Bevy wins and sunray is overwritten. A fully bidirectional sync would need a
public sunray API to read per-entity transforms — worth adding if you want ECS to reflect the glTF layout from frame 1. Say the word and I'll add Renderer::get_entity_transform upstream and wire it through.

Builds clean (cargo check -p bevy_extension --example window passes).

✻ Cogitated for 5m 41s

────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
❯ What is there to implement next  
────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
⏵⏵ accept edits on (shift+tab to cycle)                                                                                                                                                                                                           ⧉ In window.rs
Example crash fix (examples/window.rs:26)
- Defaults to examples/assets/Room.glb when no CLI arg — sunray's build_image_dependent_data asserts textures are padded to 1024, which only happens after a glTF load.

Transform integration (lib.rs)
- SunrayCamera is now just { fov_y_degrees: f32 } — position/orientation read from GlobalTransform (sync_camera uses gt.translation() + gt.forward()).
- SunrayEntity is now SunrayEntity(EntityId) — a marker. Transforms come from the entity's GlobalTransform via a new mat4_to_na helper (glam and nalgebra are both column-major so from_column_slice(&m.to_cols_array()) is zero-loss).
- Example driven entirely through Transform::look_at / transform.translation.

Auto-spawn (lib.rs:144)
- New SunrayPluginConfig::auto_spawn_initial_scene (default true). After the renderer loads the initial glTF, try_init_sunray spawns one Bevy entity per sunray entity with (SunrayEntity(id), Transform::default()).
- sync_entity_transforms uses Ref<GlobalTransform> and skips frames where is_added() is true, so the glTF-loaded transforms remain authoritative until the user actively mutates Bevy Transform.

Known tradeoff on auto-spawn: on frame 1, Bevy Transform reads as identity while sunray still renders each entity at its glTF-loaded pose. As soon as the user mutates a Transform, Bevy wins and sunray is overwritten. A fully bidirectional sync would need a
public sunray API to read per-entity transforms — worth adding if you want ECS to reflect the glTF layout from frame 1. Say the word and I'll add Renderer::get_entity_transform upstream and wire it through.

Builds clean (cargo check -p bevy_extension --example window passes).

✻ Cogitated for 5m 41s

❯ What is next to implement?                                                                                                                                                                                                                                        
⎿  ⧉ Selected 1 lines from ..\..\..\..\..\..\mnt\c\Users\MatGa\Documents\GitHub\Bosh\SunPathForked\bevy_extension\src\lib.rs in RustRover

● Four natural next steps, roughly in priority order:

1. Swapchain recreation on OUT_OF_DATE_KHR (correctness bug). Right now render_frame just warns when acquire_next_image returns out-of-date; it should call rebuild_for_size(ctx, ctx.current_extent) and skip the rest of the frame. Without this,               
   minimize/restore and some resize paths leave you stuck on a dead swapchain. — ~10 lines.
2. Graceful shutdown. Your last run showed VUID-vkDestroyDevice-device-05137 validation errors on exit (descriptor set/pool not destroyed). Add an AppExit observer (or Plugin::cleanup) that calls device_wait_idle() and drops SunrayContext before bevy tears  
   down the window. This won't fix the underlying sunray leak but will at least make shutdown ordered.
3. Bidirectional Transform sync (the tradeoff we flagged). Add Renderer::get_entity_transform(id) -> Option<Matrix4> upstream in sunray, then in try_init_sunray spawn each Bevy entity with the real glTF pose instead of Transform::default(). Also lets
   Added<SunrayEntity> become authoritative instead of a skip-first-frame hack.
4. Runtime scene API. A small command-style wrapper: SunrayCommands::load_gltf(path) -> Vec<Entity>, despawn_in_sunray(entity). Either via an exclusive system reading a Commands-ish queue resource, or a dedicated event type. Needed to move past "one glTF at
   startup."

My recommendation: do #1 + #2 together in one pass (both are small, both are correctness, and they touch the same systems), then evaluate whether you want #3 or #4 next based on your demo goals. If you're building toward dynamic scenes, #4; if you want clean
ECS↔sunray parity, #3.

Want me to take #1 + #2 now?

────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
❯                                                                                          
────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
⏵⏵ accept edits on (shift+tab to cycle)



- Defaults to examples/assets/Room.glb when no CLI arg — sunray's build_image_dependent_data asserts textures are padded to 1024, which only happens after a glTF load.

Transform integration (lib.rs)
- SunrayCamera is now just { fov_y_degrees: f32 } — position/orientation read from GlobalTransform (sync_camera uses gt.translation() + gt.forward()).
- SunrayEntity is now SunrayEntity(EntityId) — a marker. Transforms come from the entity's GlobalTransform via a new mat4_to_na helper (glam and nalgebra are both column-major so from_column_slice(&m.to_cols_array()) is zero-loss).
- Example driven entirely through Transform::look_at / transform.translation.

Auto-spawn (lib.rs:144)
- New SunrayPluginConfig::auto_spawn_initial_scene (default true). After the renderer loads the initial glTF, try_init_sunray spawns one Bevy entity per sunray entity with (SunrayEntity(id), Transform::default()).
- sync_entity_transforms uses Ref<GlobalTransform> and skips frames where is_added() is true, so the glTF-loaded transforms remain authoritative until the user actively mutates Bevy Transform.

Known tradeoff on auto-spawn: on frame 1, Bevy Transform reads as identity while sunray still renders each entity at its glTF-loaded pose. As soon as the user mutates a Transform, Bevy wins and sunray is overwritten. A fully bidirectional sync would need a
public sunray API to read per-entity transforms — worth adding if you want ECS to reflect the glTF layout from frame 1. Say the word and I'll add Renderer::get_entity_transform upstream and wire it through.

Builds clean (cargo check -p bevy_extension --example window passes).

✻ Cogitated for 5m 41s

❯ What is next to implement?                                                                                                                                                                                                                                        
⎿  ⧉ Selected 1 lines from ..\..\..\..\..\..\mnt\c\Users\MatGa\Documents\GitHub\Bosh\SunPathForked\bevy_extension\src\lib.rs in RustRover

● Four natural next steps, roughly in priority order:

1. Swapchain recreation on OUT_OF_DATE_KHR (correctness bug). Right now render_frame just warns when acquire_next_image returns out-of-date; it should call rebuild_for_size(ctx, ctx.current_extent) and skip the rest of the frame. Without this,
   minimize/restore and some resize paths leave you stuck on a dead swapchain. — ~10 lines.
2. Graceful shutdown. Your last run showed VUID-vkDestroyDevice-device-05137 validation errors on exit (descriptor set/pool not destroyed). Add an AppExit observer (or Plugin::cleanup) that calls device_wait_idle() and drops SunrayContext before bevy tears
   down the window. This won't fix the underlying sunray leak but will at least make shutdown ordered.
3. Bidirectional Transform sync (the tradeoff we flagged). Add Renderer::get_entity_transform(id) -> Option<Matrix4> upstream in sunray, then in try_init_sunray spawn each Bevy entity with the real glTF pose instead of Transform::default(). Also lets
   Added<SunrayEntity> become authoritative instead of a skip-first-frame hack.
4. Runtime scene API. A small command-style wrapper: SunrayCommands::load_gltf(path) -> Vec<Entity>, despawn_in_sunray(entity). Either via an exclusive system reading a Commands-ish queue resource, or a dedicated event type. Needed to move past "one glTF at
   startup."

My recommendation: do #1 + #2 together in one pass (both are small, both are correctness, and they touch the same systems), then evaluate whether you want #3 or #4 next based on your demo goals. If you're building toward dynamic scenes, #4; if you want clean
ECS↔sunray parity, #3.

Want me to take #1 + #2 now?


Builds clean (cargo check -p bevy_extension --example window).

✻ Cooked for 5m 13s

✻ Cooked for 5m 13s

❯ Next steps,like handling data transfers for the meshes ecc..

● Let me take a quick look at how sunray builds a BLAS today so I can give you a concrete recommendation rather than hand-waving.

Searched for 3 patterns, read 1 file (ctrl+o to expand)

● Good — now I have a concrete picture. The honest answer: "Bevy mesh data → sunray" is the biggest architectural lift left, because today sunray's scene loading is glTF-file-centric and all of BLAS/texture/material upload happens inside gltf::Gltf::new → scene.load_into_gpu → resource_manager.load_scene. There's no public API to build a BLAS from
raw vertex/index buffers — so the plugin currently can't consume a Bevy Handle<Mesh>, a bevy_image::Image, or a StandardMaterial.

Here's a staged roadmap, easiest → hardest. Each stage is independently shippable.

Stage 1 — Mesh upload plumbing in sunray (the enabler, ~1 day)                                
Add a public Renderer::create_mesh(positions: &[Vec3], normals: &[Vec3], uvs: &[Vec2], indices: &[u32], material: Material) -> SrResult<MeshHandle> and Renderer::destroy_mesh(MeshHandle). Under the hood it does what the glTF loader does per-primitive: allocate VertexBuffer/IndexBuffer, build a BLAS, register the material. Nothing in sunray is
refactored — we just lift an internal pathway to pub. Without this, nothing below is possible.

Stage 2 — Handle<Mesh> → sunray BLAS bridge in the plugin (~½ day)
- New SunrayMesh(MeshHandle) component.
- extract_meshes system reads AssetEvents<Mesh>: on LoadState::Loaded, pull the mesh's ATTRIBUTE_POSITION/NORMAL/UV_0 + indices() → call Renderer::create_mesh → stash the sunray handle in a HashMap<AssetId<Mesh>, MeshHandle> resource.
- On AssetEvent::Unused, call destroy_mesh.
- Entity spawn flow: commands.spawn((SunrayMeshOf(handle), Transform::...)) → another system creates the actual sunray entity using the stored MeshHandle.

Stage 3 — Materials and textures (~1 day)
Same pattern for Handle<Image> → sunray texture and Handle<StandardMaterial> → sunray material. The hard part is mapping PBR parameters (StandardMaterial has its own metallic/roughness/emissive fields); sunray's Material is KHR_glTF-shaped, so the mapping is mostly 1:1 but you'll need to handle base_color_texture: Option<Handle<Image>> by resolving
to the sunray texture id.

Stage 4 — Lifecycle/staging/streaming (~open-ended)                 
Right now load_gltf is synchronous — it blocks the frame during staging buffer copies and calls device_wait_idle. For a real asset pipeline you'd want upload batching, a frame-budget limit, and potentially running extract/prepare in a dedicated sub-app (mirrors what bevy_render does with its RenderApp). This is where "handling data transfers" gets
genuinely hard — not the copy itself, but doing it without stalling.

My recommendation: start with Stage 1 — it's the unblocker and purely additive (no refactor of existing sunray code). Once that lands I can immediately wire up Stage 2 and you'll have a working cmds.spawn((Mesh3d(handle), Transform::...)) flow. Then reassess whether you want textures/materials (Stage 3) or async upload (Stage 4) first, based on
whether your goal is "build Bevy scenes at runtime" (→ 3) or "stream large assets without hitches" (→ 4).

Main tradeoff to flag now: this path re-implements a slice of bevy_render on top of sunray. It's the right shape, but it's real work — roughly 2–3 days for Stages 1–3 to be solid. If instead you decide the glTF-path API (LoadSunrayScene) is enough for your actual use case, you can stop here and skip the whole thing. Worth deciding before you
invest.

Want me to start on Stage 1?     

Stage 1.5 (emissive plumbing in create_mesh)
- Added MeshData::emission: Option<[f32; 3]>. If set, ResourceManager::create_mesh builds an EmissiveTriangle per triangle (local-space positions + premultiplied emission), appends them to the BLAS emissive-triangles arena, and records the slot range on the
  BLAS so NEE picks them up. add_blas_emissive_triangles now returns the allocated slot range instead of ().
- ResourceManager::new_empty seeds rebuild_emissive_indirection() so binding 9 is never VK_NULL_HANDLE even before the first entity exists.
- Renderer::create_entity also rebuilds the indirection after TLAS rebuild, so adding emissive meshes keeps the indirection correct.

Stage 3 (public texture upload)
- New sunray::TextureHandle(pub u32) with .slot() convenience for plugging into gltf::Material texture-index fields.
- ResourceManager::create_texture(data, extent, format) uploads an Image + Sampler, allocates a slot from the top of the 1024-slot table downward (so scene textures and user textures don't collide), wires them into textures[slot].
- ResourceManager::destroy_texture(handle) reverts the slot to the fallback texture. Slots are not recycled (would collide with any still-bound materials).
- set_textures now re-applies user textures after its scene-fill, so load_scene no longer wipes them.
- Renderer::create_texture(Vec<u8>, (w, h), vk::Format) / destroy_texture(handle) — thin wrappers that device_wait_idle, forward to ResourceManager, and clear_image_dependent_data() so the descriptor sets are rebuilt next frame.
- bevy_extension re-exports SunrayTextureHandle.

Demo updates
- Uploads a 16×16 RGBA8 checkerboard via Renderer::create_texture, binds the returned slot to the red cube's base_color_texture_index.
- cube_mesh now also emits per-face UVs (standard 0,0→1,0→1,1→0,1 across the four verts) so textures map correctly.
- Red cube is non-emissive; a smaller bright white cube at y=3.5 registers MeshData.emission so its triangles act as an NEE area light. Over a second or two of accumulation the red cube should show a checker pattern lit by the white cube above.

Please run it and let me know what you see. If the checker looks washed out or the cube is still too dark, we can nudge emissive_strength on the light cube or change the base_color_factor/roughness on the red one.