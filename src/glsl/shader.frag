#version 450

layout(origin_upper_left) in vec4 gl_FragCoord;

layout(set = 0, binding = 0, rgba32f) uniform image2D output_image;

layout(set = 0, binding = 1, std140) uniform Globals {
    uint num_frames;
};

layout(location = 0) out vec4 output_color;

void main() {
    vec3 tmp_color = imageLoad(output_image, ivec2(gl_FragCoord.xy)).xyz;
    tmp_color = sqrt(tmp_color / num_frames);
    output_color = vec4(tmp_color, 1.0);
}