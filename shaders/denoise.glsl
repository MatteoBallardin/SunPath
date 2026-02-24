#version 460
layout(local_size_x = 16, local_size_y = 16) in;

layout(set = 0, binding = 0, rgba32f) uniform image2D inputImage;
layout(set = 0, binding = 1, rgba32f) uniform image2D outputImage;

layout (local_size_x = 16, local_size_y = 16) in;


void main() {
    ivec2 size = imageSize(outputImage);
    ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
    if (uv.x >= size.x || uv.y >= size.y) return;

    vec3 centerColor = imageLoad(inputImage, uv).rgb;

    vec3 minColor = vec3(10000.0);
    vec3 maxColor = vec3(-10000.0);
    vec3 avgColor = vec3(0.0);

    int count = 0;
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            // Skip the center pixel for the stats
            if (x == 0 && y == 0) continue;

            ivec2 neighborPos = uv + ivec2(x, y);

            // Bounds check
            if (neighborPos.x < 0 || neighborPos.y < 0 ||
            neighborPos.x >= size.x || neighborPos.y >= size.y) continue;

            vec3 c = imageLoad(inputImage, neighborPos).rgb;

            minColor = min(minColor, c);
            maxColor = max(maxColor, c);
            avgColor += c;
            count++;
        }
    }

    avgColor /= float(count);


    vec3 clampedColor = clamp(centerColor, minColor, maxColor);

    float mixFactor = 0.2;
    vec3 finalColor = mix(clampedColor, avgColor, mixFactor);

    //temporarly disabled. Use finalcolor instead of centercolor

    imageStore(outputImage, uv, vec4(centerColor, 1.0));
}