use cgmath::{Basis3, InnerSpace, Matrix3, Quaternion, Rad, Rotation, Rotation3, Vector3};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Globals {
    pub camera_pos: Vector3<f32>,
    pub aspect_ratio: f32,
    pub viewport: [f32; 2],
    pub window_size: [f32; 2],
}
unsafe impl bytemuck::Pod for Globals {}
unsafe impl bytemuck::Zeroable for Globals {}

impl Globals {
    pub fn arcball_rotate(&mut self, p0: [f32; 2], p1: [f32; 2]) {
        let pivot = Vector3::<f32> {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        let va = self.get_arcball_vector(p0);
        let vb = self.get_arcball_vector(p1);
        let q = Quaternion::from_arc(va, vb, None);

        let angle = va.dot(vb).min(1.0).acos() * 2.0;
        let axis = InnerSpace::normalize(va.cross(vb));
        let q2 = Matrix3::from_axis_angle(axis, Rad::<f32>(angle));

        let dir = self.camera_pos - pivot;

        self.camera_pos = q2 * dir + pivot;
    }

    fn get_arcball_vector(&self, p0: [f32; 2]) -> Vector3<f32> {
        let mut p = Vector3::<f32> {
            x: p0[0] / self.window_size[0] * 2.0 - 1.0,
            y: 1.0 * (p0[1] / self.window_size[1] * 2.0 - 1.0),
            z: 0.0,
        };

        let r = p.x * p.x + p.y * p.y;
        if r <= 1.0 {
            p.z = (1.0 - r).sqrt(); // Pythagoras
        } else {
            p = InnerSpace::normalize(p); // nearest point
        }
        return p;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub center: [f32; 3],
    pub radius: f32,
}
unsafe impl bytemuck::Pod for Sphere {}
unsafe impl bytemuck::Zeroable for Sphere {}

pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl ComputePipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        // Load shader
        let cs = include_bytes!("shader.comp.spv");
        let cs_module = device
            .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(cs.iter())).unwrap());

        // Bind Groups
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute"),
            bindings: &[
                // Globals
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                // Output image
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                        format: wgpu::TextureFormat::Rgba32Float,
                        readonly: false,
                    },
                },
                // Spheres
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &pipeline_layout,
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &cs_module,
                entry_point: "main",
            },
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}
