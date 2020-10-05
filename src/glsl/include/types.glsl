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
