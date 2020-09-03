#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub albedo: [f32; 3],
    pub type_flag: u32,
}
unsafe impl bytemuck::Pod for Material {}
unsafe impl bytemuck::Zeroable for Material {}

impl Material {
    pub fn new(albedo: [f32; 3], type_flag: u32) -> Self {
        Material { albedo, type_flag }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct MaterialBuffer {
    pub materials: Vec<Material>,
}

impl MaterialBuffer {
    pub fn to_buffer(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![];
        res.append(&mut bytemuck::cast_slice(&[self.materials.len() as u32]).to_vec());
        res.append(&mut bytemuck::cast_slice(&[0.0 as f32; 3]).to_vec());
        res.append(&mut bytemuck::cast_slice(self.materials.as_slice()).to_vec());

        res
    }

    pub fn len(&self) -> usize {
        std::mem::size_of::<Material>() * self.materials.len() + 16
    }
}
