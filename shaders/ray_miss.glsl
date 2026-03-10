#version 460
#extension GL_EXT_ray_tracing : require
#include <shaders/common.glsl>

layout(location = 0) rayPayloadInEXT ray_payload_t payload;

void main() {
    payload.dist = -1.0;
    payload.emission = vec3(0.0, 0.0, 0.0);
}