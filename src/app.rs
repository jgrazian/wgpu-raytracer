use cgmath::Vector3;
use winit::{event::WindowEvent, window::Window};

use crate::pipelines::*;

struct MouseState {
    state: winit::event::ElementState,
    position: [f32; 2],
}

pub struct State {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    globals: compute::Globals,

    globals_buffer: wgpu::Buffer,
    output_texture: wgpu::TextureView,
    spheres_buffer: wgpu::Buffer,

    compute_pipeline: compute::ComputePipeline,
    render_pipeline: render::RenderPipeline,

    size: winit::dpi::PhysicalSize<u32>,
    mouse_state: MouseState,
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
        let globals = compute::Globals {
            camera_pos: Vector3::<f32> {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            viewport: [ar * viewport_height, viewport_height],

            window_size: [size.width as f32, size.height as f32],
            aspect_ratio: ar,
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

        let spheres = [
            compute::Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 0.5,
            },
            compute::Sphere {
                center: [0.0, -100.5, -1.0],
                radius: 100.0,
            },
        ];
        let spheres_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[spheres]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let mouse_state = MouseState {
            state: winit::event::ElementState::Released,
            position: [0.0, 0.0],
        };

        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            globals,
            globals_buffer,
            output_texture,
            spheres_buffer,
            compute_pipeline,
            render_pipeline,
            size,
            mouse_state,
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
        self.globals = compute::Globals {
            camera_pos: Vector3::<f32> {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            viewport: [ar * viewport_height, viewport_height],
            window_size: [new_size.width as f32, new_size.height as f32],
            aspect_ratio: ar,
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
                    let dx = position.x as f32 - self.mouse_state.position[0];
                    let dy = position.y as f32 - self.mouse_state.position[1];

                    let p0 = [position.x as f32, position.y as f32];
                    let p1 = [self.mouse_state.position[0], self.mouse_state.position[1]];
                    self.globals.arcball_rotate(p0, p1);
                }

                self.mouse_state.position = [position.x as f32, position.y as f32];
            }
            _ => return false,
        }

        true
    }

    pub fn update(&mut self) {
        //unimplemented!()
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
            let globals_size = std::mem::size_of::<compute::Globals>();
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
                        range: 0..std::mem::size_of::<compute::Globals>() as u64,
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
                        range: 0..(std::mem::size_of::<compute::Sphere>() * 2) as u64,
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

        self.queue.submit(&[encoder.finish()]);
    }
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
