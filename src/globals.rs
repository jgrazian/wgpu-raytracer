use glam::{Mat3, Vec2, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Globals {
    pub camera_pos: Vec3,
    pub aspect_ratio: f32,
    pub viewport: Vec2,
    pub window_size: Vec2,
    pub rng_seed: f32,
    pub num_frames: u32,
}
unsafe impl bytemuck::Pod for Globals {}
unsafe impl bytemuck::Zeroable for Globals {}

impl Globals {
    pub fn arcball_rotate(&mut self, p0: Vec2, p1: Vec2) {
        if p0 == p1 {
            return;
        }
        let looking_at = Vec3::new(0.0, 1.0, 0.0);

        let va = self.get_arcball_vector(p0);
        let vb = self.get_arcball_vector(p1);

        let axis = va.cross(vb).normalize();
        let angle = va.dot(vb).min(1.0).acos() * 1.0;
        let mat = Mat3::from_axis_angle(axis, angle);

        let cur_pos = (self.camera_pos - looking_at).normalize();
        let r = (self.camera_pos - looking_at).length();
        let cur_axis = cur_pos.cross(Vec3::unit_x()).normalize();
        let cur_angle = cur_pos.dot(Vec3::unit_x()).acos();
        let cur_mat = Mat3::from_axis_angle(cur_axis, -cur_angle);

        self.camera_pos = (mat * cur_mat * Vec3::unit_x()) * r + looking_at;
    }

    fn get_arcball_vector(&self, p0: Vec2) -> Vec3 {
        let mut p = Vec3::new(
            p0.x() / self.window_size.x() * 2.0 - 1.0,
            p0.y() / self.window_size.y() * 2.0 - 1.0,
            0.0,
        );
        p.set_y(-p.y());
        let r = p.x() * p.x() + p.y() * p.y();
        if r <= 1.0 {
            p.set_z((1.0 - r).sqrt()); // Pythagoras
        } else {
            p = p.normalize(); // nearest point
        }
        p
    }

    pub fn arcball_zoom(&mut self, delta: f32) {
        let r = (Vec3::new(0.0, 1.0, 0.0) - self.camera_pos).normalize();
        self.camera_pos = self.camera_pos + r * delta * 0.1;
    }
}

#[cfg(test)]
mod tests {
    use super::Globals;
    use glam::{Vec2, Vec3};

    #[test]
    fn test_get_arcball_vector() {
        let p = Vec2::new(0.1, 0.0);
    }
}
