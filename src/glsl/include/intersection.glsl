void set_face_normal(inout HitRec rec, Ray r, vec3 outward_normal) {
    rec.front_face = dot(r.direction, outward_normal) < 0;
    rec.normal = rec.front_face ? outward_normal :-outward_normal;
}

bool hit_sphere(Sphere s, Ray r, float t_min, float t_max, inout HitRec rec) {
    vec3 oc = r.origin - s.center;
    float half_b = dot(oc, r.direction);
    float c = dot(oc, oc) - s.radius*s.radius;
    float discriminant = half_b*half_b - c;

    if (discriminant < 0.0) {
        return false;
    }

    float root = sqrt(discriminant);
    float temp = -half_b - root;
    if (temp > t_max || temp < t_min) {
        temp = -half_b + root;
        if (temp > t_max || temp < t_min) {
            return false;
        }
    }
    
    rec.t = temp;
    rec.point = ray_at(r, rec.t);
    vec3 outward_normal = (rec.point - s.center) / s.radius;
    set_face_normal(rec, r, outward_normal);
    rec.mat_ptr = s.mat_ptr;
    return true;
}

bool hit_box(BVHNode b, Ray r, vec3 inv_dir) {
    vec3 tbot = inv_dir * (b.min.xyz - r.origin);
    vec3 ttop = inv_dir * (b.max.xyz - r.origin);
    vec3 tmin = min(ttop, tbot);
    vec3 tmax = max(ttop, tbot);
    vec2 t = max(tmin.xx, tmin.yz);
    float t0 = max(t.x, t.y);
    t = min(tmax.xx, tmax.yz);
    float t1 = min(t.x, t.y);

    return t1 > max(t0, 0.0);
}

bool hit_world(Ray r, float t_min, float t_max, inout HitRec rec) {
    HitRec temp_rec;
    bool hit_anything = false;
    float closest_so_far = t_max;
    const vec3 inv_dir = 1 / r.direction;

    BVHNode node;
    uint node_index = 0;

    while (node_index != 0xFFFFFFFF) {
        node.min = bvh.nodes[2*node_index + 0];
        node.max = bvh.nodes[2*node_index + 1];

        uint shape_type = floatBitsToUint(node.min.w);

        if (shape_type != 0xFFFFFFFF) { //Hit a shape
            Sphere s;
            s.center = node.min.xyz;
            s.radius = node.min.w;
            s.mat_ptr = floatBitsToUint(node.max.x);
            
            if (hit_sphere(s, r, t_min, closest_so_far, temp_rec)) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                rec = temp_rec;
            } 

        } else if (hit_box(node, r, inv_dir)) {
            node_index += 1;
			continue;
        }

        node_index = floatBitsToUint(node.max.w);
    }

    return hit_anything;
}
