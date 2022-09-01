use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use wgpu::util::DeviceExt;

use cgmath::prelude::*;

use rand::prelude::*;
mod texture;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}
impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

#[derive(Clone, Copy)]
struct Cell {
    position: cgmath::Vector3<f32>,
    hp: i32,
    neighbors: u32,
}
impl Cell {
    fn new(position: cgmath::Vector3<f32>, hp: i32) -> Self {
        Self {
            position,
            hp,
            neighbors: 0,
        }
    }
    fn get_color(&self) -> [f32; 3] {
        if self.hp == STATE as i32 {
            [0.9, 0., 0.]
        } else {
            let intensity = (1. + self.hp as f32) / (STATE as f32 + 2.);
            [intensity, intensity, intensity]
        }
    }
    fn create_instance(&self) -> Instance {
        Instance {
            position: self.position,
            color: self.get_color(),
        }
    }
    fn get_alive(&self) -> bool {
        self.hp == STATE as i32
    }
    fn should_draw(&self) -> bool {
        self.hp >= 0
    }
    fn sync(&mut self) {
        self.hp = (self.hp == STATE as i32) as i32 * (self.hp - 1 + SURVIVAL[self.neighbors as usize] as i32) + // alive
            (self.hp < 0) as i32 * (SPAWN[self.neighbors as usize] as i32 * (STATE + 1) as i32 - 1) +  // dead
            (self.hp >= 0 && self.hp < STATE as i32) as i32 * (self.hp - 1); // dying
    }
}

#[derive(Clone, Copy)]
struct Instance {
    position: cgmath::Vector3<f32>,
    color: [f32; 3],
}
impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: cgmath::Matrix4::from_translation(self.position).into(),
            color: self.color,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 3],
}
impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    //  color
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        // A - top left
        position: [-0.5, 0.5, 0.],
    },
    Vertex {
        // B - bottom left
        position: [-0.5, -0.5, 0.],
    },
    Vertex {
        // C - bottom right
        position: [0.5, -0.5, 0.],
    },
    Vertex {
        // D - top right
        position: [0.5, 0.5, 0.],
    },
    Vertex {
        // E - top right - left face
        position: [0.5, 0.5, -1.],
    },
    Vertex {
        // F - bottom right - left face
        position: [0.5, -0.5, -1.],
    },
    Vertex {
        // G - top left - right face
        position: [-0.5, 0.5, -1.],
    },
    Vertex {
        // H - bottom left - right face
        position: [-0.5, -0.5, -1.],
    },
    Vertex {
        // G - top left - top face
        position: [-0.5, 0.5, -1.],
    },
    Vertex {
        // A - bottom left - top face
        position: [-0.5, 0.5, 0.],
    },
    Vertex {
        // D - bottom right - top face
        position: [0.5, 0.5, 0.],
    },
    Vertex {
        // E - top right - top face
        position: [0.5, 0.5, -1.],
    },
    Vertex {
        // F - top left - bottom face
        position: [0.5, -0.5, -1.],
    },
    Vertex {
        // C - bottom left - bottom face
        position: [0.5, -0.5, 0.],
    },
    Vertex {
        // B - bottom right - bottom face
        position: [-0.5, -0.5, 0.],
    },
    Vertex {
        // H - top right - bottom face
        position: [-0.5, -0.5, -1.],
    },
];
const INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, 3, 2, 5, 3, 5, 4, 6, 7, 1, 6, 1, 0, 4, 5, 7, 4, 7, 6, 8, 9, 10, 8, 10, 11,
    12, 13, 14, 12, 14, 15,
];

const INSTANCES_PER_ROW: u32 = 50;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    INSTANCES_PER_ROW as f32 * 0.5,
    INSTANCES_PER_ROW as f32 * 0.5,
    INSTANCES_PER_ROW as f32 * 0.5,
);

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

const STATE: u32 = 10;
const SURVIVAL: [bool; 27] = [
    false, false, true, false, false, false, true, false, false, true, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
];
const SPAWN: [bool; 27] = [
    false, false, false, false, true, false, true, false, true, true, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
];
const ALIVE_CHANCE_ON_START: f32 = 0.15;
const NEIGHBOR_OFFSETS: [(i32, i32, i32); 26] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
    (1, 1, 0),
    (-1, 1, 0),
    (1, -1, 0),
    (-1, -1, 0),
    (1, 0, 1),
    (-1, 0, 1),
    (1, 0, -1),
    (-1, 0, -1),
    (0, 1, 1),
    (0, -1, 1),
    (0, 1, -1),
    (0, -1, -1),
    (1, 1, 1),
    (-1, 1, 1),
    (1, -1, 1),
    (-1, -1, 1),
    (1, 1, -1),
    (-1, 1, -1),
    (1, -1, -1),
    (-1, -1, -1),
];

fn three_to_one(x: u32, y: u32, z: u32) -> usize {
    z as usize
        + y as usize * INSTANCES_PER_ROW as usize
        + x as usize * INSTANCES_PER_ROW as usize * INSTANCES_PER_ROW as usize
}
fn valid_idx(x: u32, y: u32, z: u32, offset: (i32, i32, i32)) -> bool {
    x as i32 + offset.0 >= 0
        && x as i32 + offset.0 < INSTANCES_PER_ROW as i32
        && y as i32 + offset.1 >= 0
        && y as i32 + offset.1 < INSTANCES_PER_ROW as i32
        && z as i32 + offset.2 >= 0
        && z as i32 + offset.2 < INSTANCES_PER_ROW as i32
}

struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}
impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return proj * view;
    }
}

struct CameraStaging {
    camera: Camera,
    rotation: cgmath::Deg<f32>,
}
impl CameraStaging {
    fn new(camera: Camera) -> Self {
        Self {
            camera,
            rotation: cgmath::Deg(0.0),
        }
    }
    fn update_camera(&self, camera_uniform: &mut CameraUniform) {
        camera_uniform.view_proj = (OPENGL_TO_WGPU_MATRIX
            * self.camera.build_view_projection_matrix()
            * cgmath::Matrix4::from_angle_z(self.rotation))
        .into();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

struct CameraController {
    speed: f32,
    forward_down: bool,
    backward_down: bool,
    left_down: bool,
    right_down: bool,
}
impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            forward_down: false,
            backward_down: false,
            left_down: false,
            right_down: false,
        }
    }
    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.forward_down = pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.backward_down = pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.left_down = pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.right_down = pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
    fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if self.forward_down && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.backward_down {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.right_down {
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.left_down {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    camera_staging: CameraStaging,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    clear_color: wgpu::Color,

    depth_texture: texture::Texture,
    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,

    // diffuse_bind_group: wgpu::BindGroup,
    last_frame: Option<std::time::Instant>,

    cells: Vec<Cell>,
}
impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        // NOTE: could be none, see: https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        // let diffuse_bytes = include_bytes!("minecraft-grass.png");
        // let diffuse_texture =
        //     texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "minecraft-grass.png")
        //         .unwrap();

        // let texture_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         entries: &[
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 0,
        //                 visibility: wgpu::ShaderStages::FRAGMENT,
        //                 ty: wgpu::BindingType::Texture {
        //                     multisampled: false,
        //                     view_dimension: wgpu::TextureViewDimension::D2,
        //                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //                 },
        //                 count: None,
        //             },
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 1,
        //                 visibility: wgpu::ShaderStages::FRAGMENT,
        //                 ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //                 count: None,
        //             },
        //         ],
        //         label: Some("texture_bind_group_layout"),
        //     });

        // let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &texture_bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
        //         },
        //     ],
        //     label: Some("diffuse_bind_group"),
        // });

        let camera = Camera {
            eye: (0., 50., 50.).into(),
            target: (0., 0., 0.).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: size.width as f32 / size.height as f32,
            fovy: 45.,
            znear: 0.01,
            zfar: 300.,
        };
        let camera_controller = CameraController::new(0.2);

        let mut camera_uniform = CameraUniform::new();
        let camera_staging = CameraStaging::new(camera);
        camera_staging.update_camera(&mut camera_uniform);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        let mut rng = rand::thread_rng();
        let mut instances: Vec<Instance> = Vec::new();
        let mut cells: Vec<Cell> = Vec::new();
        for x in 0..INSTANCES_PER_ROW {
            for y in 0..INSTANCES_PER_ROW {
                for z in 0..INSTANCES_PER_ROW {
                    let mut cell = Cell::new(
                        cgmath::Vector3 {
                            x: x as f32,
                            y: y as f32,
                            z: z as f32,
                        } - INSTANCE_DISPLACEMENT,
                        -1,
                    );
                    if x >= INSTANCES_PER_ROW / 3
                        && x <= INSTANCES_PER_ROW * 2 / 3
                        && y >= INSTANCES_PER_ROW / 3
                        && y <= INSTANCES_PER_ROW * 2 / 3
                        && z >= INSTANCES_PER_ROW / 3
                        && z <= INSTANCES_PER_ROW * 2 / 3
                        && ALIVE_CHANCE_ON_START < rng.gen()
                    {
                        cell.hp = STATE as i32;
                    }
                    cells.push(cell);
                    instances.push(cell.create_instance());
                }
            }
        }
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };
        let mut last_frame = None;
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {}
            else {
                last_frame = Some(std::time::Instant::now());
            }
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            camera_staging,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            clear_color,
            depth_texture,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            instances,
            instance_buffer,
            // diffuse_bind_group,
            last_frame,
            cells,
        }
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x / self.size.width as f64,
                    g: position.y / self.size.height as f64,
                    b: 1. - (position.x / self.size.width as f64),
                    a: 1.0,
                };
            }
            // WindowEvent::KeyboardInput {
            //     input:
            //         KeyboardInput {
            //             state,
            //             virtual_keycode: Some(VirtualKeyCode::Space),
            //             ..
            //         },
            //     ..
            // } => self.space_down = *state == ElementState::Pressed,
            _ => {}
        }
        self.camera_controller.process_events(event)
    }
    fn update(&mut self) {
        self.count_neighbors();
        self.sync_cells();

        let instance_data = self.get_instance_data();
        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );

        self.camera_controller
            .update_camera(&mut self.camera_staging.camera);
        self.camera_staging.update_camera(&mut self.camera_uniform);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as u32)
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {}
            else {
                let elapsed_seconds = self.last_frame.unwrap().elapsed().as_secs_f64();
                let fps = 1. / elapsed_seconds;
                println!("FPS: {:.0}", fps);
                self.last_frame = Some(std::time::Instant::now());
            }
        }

        Ok(())
    }

    fn get_instance_data(&mut self) -> Vec<InstanceRaw> {
        self.instances.clear();
        for cell in self.cells.iter() {
            if cell.should_draw() {
                self.instances.push(cell.create_instance());
            }
        }
        // for _i in self.instances.len()..self.cells.len() {
        //     self.instances.push(Instance {
        //         position: cgmath::Vector3 { x: , y: (), z: () }
        //     });
        // }
        self.instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>()
    }
    fn count_neighbors(&mut self) {
        for x in 0..INSTANCES_PER_ROW {
            for y in 0..INSTANCES_PER_ROW {
                for z in 0..INSTANCES_PER_ROW {
                    let one_idx = three_to_one(x, y, z);
                    self.cells[one_idx].neighbors = 0;
                    for offset in NEIGHBOR_OFFSETS.iter() {
                        if valid_idx(x, y, z, *offset) {
                            if self.cells[three_to_one(
                                (x as i32 + offset.0) as u32,
                                (y as i32 + offset.1) as u32,
                                (z as i32 + offset.2) as u32,
                            )]
                            .get_alive()
                            {
                                self.cells[one_idx].neighbors += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    fn sync_cells(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.sync();
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 450));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("body")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't add canvas to doc");
    }

    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    state.resize(state.size)
                }
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
            }
        }
        Event::RedrawEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
