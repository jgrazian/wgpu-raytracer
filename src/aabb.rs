use glam::Vec3;

pub trait Bounded {
    fn get_bounds(&self) -> AABB;
}

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

#[derive(Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl AABB {
    pub fn from_bounds(min: Vec3, max: Vec3) -> Self {
        AABB { min, max }
    }

    pub fn union(a: &Self, b: &Self) -> Self {
        AABB {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        }
    }

    pub fn largest_axis(&self) -> Axis {
        let size = self.max - self.min;
        if size.x() >= size.y() && size.x() >= size.z() {
            return Axis::X;
        } else if size.y() >= size.x() && size.y() >= size.z() {
            return Axis::Y;
        } else {
            return Axis::Z;
        }
    }

    pub fn get_min(&self) -> Vec3 {
        self.min
    }

    pub fn get_max(&self) -> Vec3 {
        self.max
    }

    pub fn size_x(&self) -> f32 {
        self.max.x() - self.min.x()
    }

    pub fn size_y(&self) -> f32 {
        self.max.y() - self.min.y()
    }

    pub fn size_z(&self) -> f32 {
        self.max.z() - self.min.z()
    }

    pub fn min_x(&self) -> f32 {
        self.min.x()
    }

    pub fn min_y(&self) -> f32 {
        self.min.y()
    }

    pub fn min_z(&self) -> f32 {
        self.min.z()
    }

    pub fn max_x(&self) -> f32 {
        self.max.x()
    }

    pub fn max_y(&self) -> f32 {
        self.max.y()
    }

    pub fn max_z(&self) -> f32 {
        self.max.z()
    }
}
