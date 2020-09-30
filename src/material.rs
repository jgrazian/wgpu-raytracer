use crate::geometry::Buffer;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub albedo: [f32; 3],
    pub type_flag: u32,
    pub is_light: bool,
}
unsafe impl bytemuck::Pod for Material {}
unsafe impl bytemuck::Zeroable for Material {}

impl Material {
    pub fn new(albedo: [f32; 3], type_flag: u32, is_light: bool) -> Self {
        Material {
            albedo,
            type_flag,
            is_light,
        }
    }
}

impl Buffer for Vec<Material> {
    fn to_buffer(&self) -> Vec<u8> {
        let mut flat: Vec<u8> = vec![];
        flat.extend_from_slice(&mut bytemuck::cast_slice(&[self.len() as u32]));
        flat.extend_from_slice(&mut bytemuck::cast_slice(&[0.0 as u32; 3]));
        for i in 0..self.len() {
            //flat.extend_from_slice(bytemuck::cast_slice(&[self[i]])); // 0, 1, 2, 3, 4
            flat.extend_from_slice(bytemuck::cast_slice(&[self[i].albedo]));
            flat.extend_from_slice(bytemuck::cast_slice(&[self[i].type_flag]));
            flat.extend_from_slice(bytemuck::cast_slice(&[self[i].is_light as u32]));
            flat.extend_from_slice(bytemuck::cast_slice(&[0 as u32; 3])); // 5, 6, 7
        }

        flat
    }

    fn buffer_size(&self) -> usize {
        (std::mem::size_of::<Material>() + 12) * self.len() + 16
    }
}
