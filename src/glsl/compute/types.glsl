struct Material {
    vec3 albedo;
    uint type;
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
    vec3 min;
    uint type;
    vec3 max;
    uint ptr;
};

struct Ray {
    vec3 origin;
    vec3 direction;
};
