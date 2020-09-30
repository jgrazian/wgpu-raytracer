void set_face_normal(inout HitRec rec, Ray r, vec3 outward_normal) {
    rec.front_face = dot(r.direction, outward_normal) < 0;
    rec.normal = rec.front_face ? outward_normal :-outward_normal;
}

bool hit_sphere(Sphere s, Ray r, float t_min, float t_max, inout HitRec rec) {
    vec3 oc = r.origin - s.center;
    float a = dot(r.direction, r.direction);
    float half_b = dot(oc, r.direction);
    float c = dot(oc, oc) - s.radius*s.radius;
    float discriminant = half_b*half_b - a*c;

    if (discriminant > 0.0) {
        float root = sqrt(discriminant);
        float temp = (-half_b - root) / a;
        if (temp < t_max && temp > t_min) {
            rec.t = temp;
            rec.point = ray_at(r, rec.t);
            vec3 outward_normal = (rec.point - s.center) / s.radius;
            set_face_normal(rec, r, outward_normal);
            rec.mat_ptr = s.mat_ptr;
            return true;
        }

        temp = (-half_b + root) / a;
        if (temp < t_max && temp > t_min) {
            rec.t = temp;
            rec.point = ray_at(r, rec.t);
            vec3 outward_normal = (rec.point - s.center) / s.radius;
            set_face_normal(rec, r, outward_normal);
            rec.mat_ptr = s.mat_ptr;
            return true;
        }
    }
    return false;
}

bool hit_box(BVHNode b, Ray r) {
    vec3 inv_dir = 1 / r.direction;

    vec3 tbot = inv_dir * (b.min - r.origin);
    vec3 ttop = inv_dir * (b.max - r.origin);
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

    uint cur_index = 0;
    BVHNode cur_node;

    while (cur_index < bvh.len) {
        cur_node = bvh.data[cur_index];
        if (hit_box(cur_node, r)) {
            cur_index = cur_index + 1;
            if (cur_node.type == 1) {
                if (hit_sphere(spheres.data[cur_node.ptr], r, t_min, closest_so_far, temp_rec)) {
                    hit_anything = true;
                    closest_so_far = temp_rec.t;
                    rec = temp_rec;
                } 
            }
        } else {
            if (cur_node.type == 0) {
                cur_index = cur_node.ptr;
            } else {
                cur_index = cur_index + 1;
            }  
        }
    }

    return hit_anything;
}
