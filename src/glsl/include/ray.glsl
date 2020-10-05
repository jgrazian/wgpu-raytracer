struct Ray {
    vec3 origin;
    vec3 direction;
};

vec3 ray_at(Ray r, float t) {
    return (r.origin + t*r.direction);
}
