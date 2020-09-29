#version 450

layout(origin_upper_left) in vec4 gl_FragCoord;

layout(set = 0, binding = 0, rgba32f) uniform image2D output_image;

layout(location = 0) out vec4 output_color;

void main() {
    output_color = vec4(imageLoad(output_image, ivec2(gl_FragCoord.xy)).xyz, 1.0);
}