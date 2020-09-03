use glam::Vec3;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub mat_ptr: u32,
    pub pad: [u32; 3],
}
unsafe impl bytemuck::Pod for Sphere {}
unsafe impl bytemuck::Zeroable for Sphere {}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, mat_ptr: u32) -> Self {
        Sphere {
            center,
            radius,
            mat_ptr,
            pad: [0; 3],
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct SphereBuffer {
    pub spheres: Vec<Sphere>,
}

impl SphereBuffer {
    pub fn to_buffer(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![];
        res.append(&mut bytemuck::cast_slice(&[self.spheres.len() as u32]).to_vec());
        res.append(&mut bytemuck::cast_slice(&[0.0 as f32; 3]).to_vec());
        res.append(&mut bytemuck::cast_slice(self.spheres.as_slice()).to_vec());

        res
    }

    pub fn len(&self) -> usize {
        std::mem::size_of::<Sphere>() * self.spheres.len() + 16
    }
}
