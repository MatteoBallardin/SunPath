#version 460

layout(local_size_x = 16, local_size_y = 16, local_size_z = 1) in;

layout(push_constant) uniform PushConstants {
    uint frame_count;
} pc;

layout(set = 0, binding = 0, rgba32f) uniform readonly image2D spatial_result;
layout(set = 0, binding = 1, rgba32f) uniform writeonly image2D output_image;
layout(set = 0, binding = 2, rg16f) uniform readonly image2D motion_vector_image;
layout(set = 0, binding = 3) uniform sampler2D history_samplers[2];
layout(set = 0, binding = 4, rgba32f) uniform image2D accumulation_images[2];

const float ACCUMULATION_FACTOR = 1.0;

vec3 get_historical_color(uint history_idx, vec2 uv, vec3 current_color) {
    if (pc.frame_count == 0) return current_color;
    return texture(history_samplers[history_idx], uv).rgb;
}

vec3 ACESFilm(vec3 x) {
    float a = 2.51, b = 0.03, c = 2.43, d = 0.59, e = 0.14;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), 0.0, 1.0);
}

vec3 perform_temporal_accumulation(vec3 current_color, sampler2D history_sampler, vec2 uv, vec2 motion_vector, uint frame_count) {
    vec2 prev_uv = uv - motion_vector;
    bool is_off_screen = any(lessThan(prev_uv, vec2(0.0))) || any(greaterThan(prev_uv, vec2(1.0)));

    if (is_off_screen) return current_color;

    vec3 history_color = texture(history_sampler, prev_uv).rgb;
    float blend_factor = (frame_count == 0) ? 1.0 : (1.0 / float(frame_count + 1));
    blend_factor = max(blend_factor, 0.4);
    return mix(history_color, current_color, blend_factor);
}

void main() {
    ivec2 size = imageSize(output_image);
    ivec2 pixel_coords = ivec2(gl_GlobalInvocationID.xy);
    if (pixel_coords.x >= size.x || pixel_coords.y >= size.y) return;

    vec2 uv = (vec2(pixel_coords) + 0.5) / vec2(size);
    vec2 motion_vector = imageLoad(motion_vector_image, pixel_coords).rg;
    vec3 current_color = imageLoad(spatial_result, pixel_coords).rgb;

    uint history_idx = pc.frame_count % 2;
    uint accum_idx   = (pc.frame_count + 1) % 2;

    vec3 accumulated_color = perform_temporal_accumulation(
    current_color,
    history_samplers[history_idx],
    uv,
    motion_vector,
    pc.frame_count
    );

    // Final mix factor from original logic
    accumulated_color = mix(get_historical_color(history_idx, uv, current_color), accumulated_color, ACCUMULATION_FACTOR);

    // Store raw accumulation for next frame
    imageStore(accumulation_images[accum_idx], pixel_coords, vec4(accumulated_color, 1.0));

    // Tone mapping and output
    float EXPOSURE = 3.0;
    vec3 final_color = ACESFilm(accumulated_color * EXPOSURE);
    imageStore(output_image, pixel_coords, vec4(final_color, 1.0));
}