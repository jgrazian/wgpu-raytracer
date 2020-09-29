#define FLT_MAX 3.402823466e+38
#define FLT_MIN 1.175494351e-38
#define M_TWO_PI 6.28318530718

#define MAX_SPHERES 2048
#define MAX_MATERIALS 2048
#define MAX_NODES 2048

float rand(inout float seed, vec2 pixel) {
    float result = fract(sin(seed / 100.0f * dot(pixel, vec2(12.9898f, 78.233f))) * 43758.5453f);
    seed += 1.0f;
    return result;
}

vec2 rand2(inout float seed, vec2 pixel) {
    return vec2(rand(seed, pixel), rand(seed, pixel));
}

vec3 rand3(inout float seed, vec2 pixel) {
    vec3 p = 2.0 * vec3(rand(seed, pixel), rand(seed, pixel), rand(seed, pixel)) - vec3(1.0);
    return normalize(p);
}

vec3 sample_sphere_uniform(vec2 s) {
    float phi = M_TWO_PI * s.x;
    float cos_theta = 1.0 - 2.0 * s.y;
    float sin_theta = sqrt(1.0 - cos_theta * cos_theta);

    return vec3(cos(phi) * sin_theta, cos_theta, sin(phi) * sin_theta);
}

vec3 random_in_hemisphere(vec3 normal, vec2 s) {
    vec3 in_unit_sphere = sample_sphere_uniform(s);
    if (dot(in_unit_sphere, normal) > 0.0) { // In the same hemisphere as the normal
        return in_unit_sphere;
    } else {
        return -in_unit_sphere;
    }
}

float schlick(float cosine, float ref_idx) {
    float r0 = (1-ref_idx) / (1+ref_idx);
    r0 = r0*r0;
    return r0 + (1-r0)*pow((1 - cosine), 5);
}
