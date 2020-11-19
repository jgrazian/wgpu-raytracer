Ray get_ray(Camera c, float s, float t, uvec3 x) {
    float theta = radians(c.vfov);
    float h = tan(theta / 2.0);
    float viewport_height = 2.0 * h;
    float viewport_width = c.aspect_ratio * viewport_height;

    vec3 vup = vec3(0.0, 1.0, 0.0);
    vec3 w = normalize(c.look_from - c.look_at);
    vec3 u = normalize(cross(vup, w));
    vec3 v = cross(w, u);
    vec3 horizontal = c.focus_dist * viewport_width * u;
    vec3 vertical = c.focus_dist * viewport_height * v;
    vec3 upper_left_corner = c.look_from - horizontal/2.0 + vertical/2.0 - c.focus_dist*w;

    vec2 rd = 0.5 * c.aperture * random_in_unit_disk(x);
    vec3 offset = u * rd.x + v * rd.y;

    vec3 ray_dir = normalize(upper_left_corner + s*horizontal - t*vertical - c.look_from - offset);
    return Ray(c.look_from + offset, ray_dir);

    // ------------------------------------
    // const vec3 look_from = c.look_from;
    // const vec3 look_at = c.look_at;
    // const vec3 vup = vec3(0.0, 1.0, 0.0);

    // vec3 w = normalize(look_from - look_at);
    // vec3 uu = normalize(cross(vup, w));
    // vec3 vv = cross(w, uu);
    // vec3 horizontal = globals.viewport.x * uu;
    // vec3 vertical = globals.viewport.y * vv;
    // vec3 upper_left_corner = look_from - horizontal/2.0 + vertical/2.0 - w;

    // vec3 ray_dir = normalize(upper_left_corner + s*horizontal - t*vertical - c.look_from);
    // return Ray(c.look_from, ray_dir);
}