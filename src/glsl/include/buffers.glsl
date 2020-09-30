layout(local_size_x = 32, local_size_y = 32) in;

layout(set = 0, binding = 0, std140) uniform Globals {
    vec3 camera_origin;
    float aspect_ratio;
    vec2 viewport;
    vec2 window_size;
    float seed;
    uint num_frames;
} globals;

layout(set = 0, binding = 1, rgba32f) uniform image2D output_image;

layout(set = 0, binding = 2, std140) buffer Spheres {
    uint len;
    Sphere data[MAX_SPHERES];
} spheres;

layout(set = 0, binding = 3, std140) uniform Materials {
    uint len;
    Material data[MAX_MATERIALS];
} materials;

layout(set = 0, binding = 4, std140) buffer BVH {
    uint len;
    BVHNode data[MAX_NODES];
} bvh;
