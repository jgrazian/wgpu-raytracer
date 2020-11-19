struct Camera {
    vec3 look_from;
    float vfov;
    vec3 look_at;
    float aspect_ratio;
    float aperture;
    float focus_dist;
};

struct Material {
    vec3 albedo;
    uint type;
    bool is_light;
};

struct HitRec {
    vec3 point;
    vec3 normal;
    float t;
    bool front_face;
    uint mat_ptr;
};

struct Sphere {
    vec3 center;
    float radius;
    uint mat_ptr;
};

struct BVHNode {
    vec4 min;
    vec4 max;
};
