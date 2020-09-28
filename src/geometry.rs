use crate::aabb::{Bounded, AABB};
use glam::Vec3;

pub trait Buffer {
    fn to_buffer(&self) -> Vec<u8>;
    fn buffer_size(&self) -> usize;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub mat_ptr: u32,
}
unsafe impl bytemuck::Pod for Sphere {}
unsafe impl bytemuck::Zeroable for Sphere {}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, mat_ptr: u32) -> Self {
        Sphere {
            center,
            radius,
            mat_ptr,
        }
    }
}

impl Bounded for Sphere {
    fn get_bounds(&self) -> AABB {
        AABB::from_bounds(
            self.center - Vec3::splat(self.radius),
            self.center + Vec3::splat(self.radius),
        )
    }
}

impl Buffer for Vec<Sphere> {
    fn to_buffer(&self) -> Vec<u8> {
        let mut flat: Vec<u8> = Vec::new();

        flat.extend_from_slice(bytemuck::cast_slice(&[self.len() as u32])); // 0
        flat.extend_from_slice(bytemuck::cast_slice(&[0 as u32; 3])); // 1, 2, 3
        for i in 0..self.len() {
            flat.extend_from_slice(bytemuck::cast_slice(&[self[i]])); // 0, 1, 2, 3, 4
            flat.extend_from_slice(bytemuck::cast_slice(&[0 as u32; 3])); // 5, 6, 7
        }

        flat
    }

    fn buffer_size(&self) -> usize {
        (std::mem::size_of::<Sphere>() + 12) * self.len() + 16
    }
}
