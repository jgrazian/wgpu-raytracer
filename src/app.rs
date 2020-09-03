use glam::{Vec2, Vec3};
use rand;
use winit::{event::WindowEvent, window::Window};

use crate::geometry;
use crate::globals;
use crate::material;
use crate::pipelines::*;

struct MouseState {
    state: winit::event::ElementState,
    position: Vec2,
}

pub struct State {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    globals: globals::Globals,
    spheres: geometry::SphereBuffer,
    materials: material::MaterialBuffer,

    globals_buffer: wgpu::Buffer,
    output_texture: wgpu::TextureView,
    spheres_buffer: wgpu::Buffer,
    material_buffer: wgpu::Buffer,

    compute_pipeline: compute::ComputePipeline,
    render_pipeline: render::RenderPipeline,

    size: winit::dpi::PhysicalSize<u32>,
    mouse_state: MouseState,
    dirty: bool,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // ---- Hardware ----
        // Create Surface
        let surface = wgpu::Surface::create(window);

        // Pick a gpu
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();
        println!("{}", adapter.get_info().name);

        // Request access to that GPU
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        // Create swap chain
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // ---- Pipelines ----
        let compute_pipeline = compute::ComputePipeline::new(&device);
        let render_pipeline = render::RenderPipeline::new(&device);

        // ---- Buffers ----
        let viewport_height = 2.0;
        let ar = size.width as f32 / size.height as f32;
        let globals = globals::Globals {
            camera_pos: Vec3::new(0.0, 0.0, 1.25),
            viewport: Vec2::new(ar * viewport_height, viewport_height),
            window_size: Vec2::new(size.width as f32, size.height as f32),
            aspect_ratio: ar,
            rng_seed: rand::random(),
            num_frames: 0,
        };
        let globals_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[globals]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsage::STORAGE,
        });
        let output_texture = output_texture.create_default_view();

        let mut spheres = geometry::SphereBuffer {
            spheres: vec![geometry::Sphere::new(Vec3::new(0.0, -100.0, 0.0), 100.0, 0)],
        };
        spheres.spheres.extend(make_sphereflake());
        let spheres_buffer = device.create_buffer_with_data(
            spheres.to_buffer().as_slice(),
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
        );

        let materials = material::MaterialBuffer {
            materials: vec![
                material::Material::new([1.0, 0.7, 0.0], 0),
                material::Material::new([0.1, 0.1, 0.1], 1),
            ],
        };
        let material_buffer = device.create_buffer_with_data(
            materials.to_buffer().as_slice(),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        // Misc
        let mouse_state = MouseState {
            state: winit::event::ElementState::Released,
            position: Vec2::new(0.0, 0.0),
        };

        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            globals,
            spheres,
            materials,
            globals_buffer,
            output_texture,
            spheres_buffer,
            material_buffer,
            compute_pipeline,
            render_pipeline,
            size,
            mouse_state,
            dirty: false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        // Update buffers
        let viewport_height = 2.0;
        let ar = new_size.width as f32 / new_size.height as f32;
        self.globals = globals::Globals {
            camera_pos: self.globals.camera_pos,
            viewport: Vec2::new(ar * viewport_height, viewport_height),
            window_size: Vec2::new(new_size.width as f32, new_size.height as f32),
            aspect_ratio: ar,
            rng_seed: rand::random(),
            num_frames: 0,
        };

        let output_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output texture"),
            size: wgpu::Extent3d {
                width: new_size.width,
                height: new_size.height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsage::STORAGE,
        });
        self.output_texture = output_texture.create_default_view();

        self.render();
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput { state, .. } => self.mouse_state.state = *state,
            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_state.state == winit::event::ElementState::Pressed {
                    self.dirty = true;
                    let p1 = Vec2::new(position.x as f32, position.y as f32);
                    self.globals.arcball_rotate(self.mouse_state.position, p1);
                }
                self.mouse_state.position = Vec2::new(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    self.dirty = true;
                    self.globals.arcball_zoom(*y);
                }
                _ => return false,
            },
            _ => return false,
        }

        true
    }

    pub fn update(&mut self) {
        self.globals.rng_seed = rand::random();
        if self.dirty {
            self.globals.num_frames = 0;
            self.dirty = false;
        }
    }

    pub fn render(&mut self) {
        let frame = self
            .swap_chain
            .get_next_texture()
            .expect("Timeout when acquiring next swap chain texture");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

        //Copy new data to GPU
        {
            let globals_size = std::mem::size_of::<globals::Globals>();
            let globals_buffer = self.device.create_buffer_with_data(
                bytemuck::cast_slice(&[self.globals]),
                wgpu::BufferUsage::COPY_SRC,
            );

            encoder.copy_buffer_to_buffer(
                &globals_buffer,
                0,
                &self.globals_buffer,
                0,
                globals_size as wgpu::BufferAddress,
            );
        }

        //Create bind groups
        let compute_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute bind group"),
            layout: &self.compute_pipeline.bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &self.globals_buffer,
                        range: 0..std::mem::size_of::<globals::Globals>() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.output_texture),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &self.spheres_buffer,
                        range: 0..self.spheres.len() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &self.material_buffer,
                        range: 0..self.materials.len() as u64,
                    },
                },
            ],
        });

        let render_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render bind group"),
            layout: &self.render_pipeline.bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.output_texture),
            }],
        });

        // Compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass();
            compute_pass.set_pipeline(&self.compute_pipeline.pipeline);
            compute_pass.set_bind_group(0, &compute_bind_group, &[]);
            compute_pass.dispatch((self.size.width + 31) / 32, (self.size.height + 32) / 32, 1);
        }

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 1.0,
                        b: 0.0,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline.pipeline);
            render_pass.set_bind_group(0, &render_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        self.globals.num_frames += 1;
        self.queue.submit(&[encoder.finish()]);
    }
}

fn make_sphereflake() -> Vec<geometry::Sphere> {
    let r = sphereflake(Vec3::unit_y(), Vec3::unit_y(), 1.0, 0);
    r
}

fn sphereflake(pos: Vec3, axis: Vec3, r: f32, depth: u32) -> Vec<geometry::Sphere> {
    let mut s = Vec::new();
    s.push(geometry::Sphere::new(pos, r, 1));

    if depth == 3 {
        return s;
    }

    let perp: Vec3;
    if axis.x() != 0.0 {
        perp = Vec3::new(-axis.y(), axis.x(), 0.0).normalize();
    } else if axis.y() != 0.0 {
        perp = Vec3::new(axis.y(), -axis.x(), 0.0).normalize();
    } else {
        perp = Vec3::new(axis.z(), 0.0, -axis.x()).normalize();
    };

    for i in 1..3 {
        let mat = glam::Mat3::from_axis_angle(perp, 0.785398 * i as f32);
        let a1 = mat * axis.normalize();
        let angle = 2.0 * 3.1415926 / (i * 3) as f32;
        for j in 0..i * 3 {
            let mat =
                glam::Mat3::from_axis_angle(axis, angle * j as f32 + (i as f32 - 1.0) * 0.523599);
            let new_axis = (mat * a1).normalize();
            let new_pos = pos + new_axis * r * 1.33;
            s.extend(sphereflake(new_pos, new_axis, 0.33 * r, depth + 1));
        }
    }

    s
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct Pixel {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

unsafe impl bytemuck::Zeroable for Pixel {}
unsafe impl bytemuck::Pod for Pixel {}
