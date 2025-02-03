use bytemuck::{cast_slice, Pod, Zeroable};
use cgmath::{perspective, Deg, Matrix4, Point3, Quaternion, Rotation, Rotation3, SquareMatrix, Vector3};
use icosphere::Icosphere;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{include_wgsl, vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Extent3d, FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages, StencilState, StoreOp, SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, VertexBufferLayout, VertexState, VertexStepMode};
use wgpu::{Backends, Device, DeviceDescriptor, Instance, InstanceDescriptor, MemoryHints, PowerPreference, Queue, RequestAdapterOptions, Surface, WindowHandle};
use winit::dpi::PhysicalSize;

use crate::system::{PlanetInstance, System};

mod icosphere;

pub(super) struct Canvas {
    pub(super) device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    config_changed: bool,

    camera: Matrix4<f32>,

    depth_texture: Texture,
    bind_group: BindGroup,
    index_buffer: Buffer,
    vertex_buffer: Buffer,
    camera_buffer: Buffer,
    render: RenderPipeline,
    grid_render: RenderPipeline,
    index_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct Vertex {
    position: [f32; 4],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct Camera {
    projection: [[f32; 4]; 4],
    position: [f32; 4]
}

impl Canvas {
    pub(super) async fn new(window: impl WindowHandle + 'static, size: PhysicalSize<u32>) -> Canvas {
        #[cfg(not(target_os = "windows"))]
        let instance = Instance::new(Default::default());

        #[cfg(target_os = "windows")]
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::DX12,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface)
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            label: None,
            memory_hints: MemoryHints::Performance,
            required_features: Default::default(),
            required_limits: Default::default()
        }, None).await.unwrap();

        let config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1)).unwrap();

        let icosphere = Icosphere::new(3);
        let vertex_buffer = icosphere.vertex_buffer(&device);
        let index_buffer = icosphere.index_buffer(&device);
        let index_count = icosphere.index_count();
        
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: cast_slice(&[Camera::zeroed()])
        });

        let shader_code = include_wgsl!("shader.wgsl");
        let grid_shader_code = include_wgsl!("grid_shader.wgsl");
        let shader_module = device.create_shader_module(shader_code);
        let grid_shader_module = device.create_shader_module(grid_shader_code);

        let camera_bind_group_layout_entry = BindGroupLayoutEntry {
            binding: 0,
            count: None,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None
            }
        };

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[camera_bind_group_layout_entry]
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding()
            }]
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });

        let vertex_state = VertexState {
            buffers: &[Vertex::desc(), PlanetInstance::desc()],
            module: &shader_module, 
            entry_point: Some("vertex"),
            compilation_options: Default::default()
        };

        let grid_vertex_state = VertexState {
            buffers: &[],
            module: &grid_shader_module, 
            entry_point: Some("vertex"),
            compilation_options: Default::default()
        };

        let fragment_state = FragmentState {
            module: &shader_module,
            entry_point: Some("fragment"),
            compilation_options: Default::default(),
            targets: &[Some(ColorTargetState {
                format: config.format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL
            })]
        };

        let grid_fragment_state = FragmentState {
            module: &grid_shader_module,
            entry_point: Some("fragment"),
            compilation_options: Default::default(),
            targets: &[Some(ColorTargetState {
                format: config.format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL
            })]
        };

        let multisample_state = MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false
        };

        let primitive_state = PrimitiveState {
            conservative: false,
            cull_mode: None,
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill
        };

        let depth_stencil_state = DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default()
        };

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: vertex_state,
            fragment: Some(fragment_state),
            cache: None,
            depth_stencil: Some(depth_stencil_state.clone()),
            multisample: multisample_state,
            multiview: None,
            primitive: primitive_state,
        });

        let grid_render = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: grid_vertex_state,
            fragment: Some(grid_fragment_state),
            cache: None,
            depth_stencil: Some(depth_stencil_state),
            multisample: multisample_state,
            multiview: None,
            primitive: primitive_state,
        });

        let depth_texture = Self::create_depth_texture(&device, config.width, config.height);

        Canvas {
            device,
            queue,
            surface,
            config,
            depth_texture,
            camera: Matrix4::identity(),
            bind_group,
            config_changed: true,
            index_buffer,
            vertex_buffer,
            camera_buffer,
            render: render_pipeline,
            index_count,
            grid_render
        }
    }

    pub(super) fn update(&mut self, system: &mut System, width: u32, height: u32, yaw: f32, pitch: f32, zoom: f32) {
        if self.config_changed || self.config.width != width.max(1) || self.config.height != height.max(1) {
            self.config.width = width.max(1);
            self.config.height = height.max(1);
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Self::create_depth_texture(&self.device, self.config.width, self.config.height);
            self.config_changed = false;
        }

        let texture = self.surface.get_current_texture().unwrap();
        let view = texture.texture.create_view(&Default::default());
        let depth_view = self.depth_texture.create_view(&Default::default());
        let mut command_encoder = self.device.create_command_encoder(&Default::default());
        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store
                }),
                stencil_ops: None
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store
                }
            })]
        });

        let fovy = Deg(90.0);
        let aspect = self.config.width as f32 / self.config.height as f32;
        let yaw_rotation = Quaternion::from_angle_z(Deg(yaw)); 
        let pitch_rotation = Quaternion::from_angle_x(Deg(pitch));

        let eye = Point3::new(0.0, 2.0, 0.0);
        let eye = pitch_rotation.rotate_point(eye);
        let eye = yaw_rotation.rotate_point(eye);
        let eye = eye * zoom.powf(2.0);

        let center = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::new(0.0, 0.0, -1.0);
        let view = Matrix4::look_at_rh(eye, center, up);
        
        let proj = perspective(fovy, aspect, 0.01, 100.0);
        self.camera = proj * view;

        self.queue.write_buffer(&self.camera_buffer, 0, cast_slice(&[Camera {
            projection: self.camera.into(),
            position: [eye.x, eye.y, eye.z, 1.0]
        }]));

        render_pass.set_pipeline(&self.render);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, system.planet_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw_indexed(0..self.index_count, 0, 0..system.step(&self.queue));

        render_pass.set_pipeline(&self.grid_render);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);

        drop(render_pass);
        let command_buffer = command_encoder.finish();
        self.queue.submit([command_buffer]);
        texture.present();
    }

    

    fn create_depth_texture(device: &Device, width: u32, height: u32) -> Texture {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1
        };

        let desc = TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        };

        let depth_texture = device.create_texture(&desc);

        /*let view = depth_texture.create_view(&Default::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });*/
        
        depth_texture
    }
}

impl Vertex {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: const {
                &vertex_attr_array![
                    0 => Float32x4,
                    1 => Float32x4
                ]
            }
        }
    }
}