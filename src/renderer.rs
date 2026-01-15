use pyo3::prelude::*;
use std::sync::Arc;
use winit::window::Window;

// Command types for batching
pub const CMD_CLEAR: u8 = 0;
pub const CMD_DRAW: u8 = 1;
pub const CMD_DRAW_EX: u8 = 2;
pub const CMD_RECT_FILL: u8 = 3;
pub const CMD_LINE: u8 = 4;
pub const CMD_CIRCLE_FILL: u8 = 5;
pub const CMD_TEXT: u8 = 6;
pub const CMD_PARTICLES_RENDER: u8 = 7;
pub const CMD_LIGHT_DRAW: u8 = 8;

/// Vertex structure for sprite rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

impl SpriteVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x2, // position
        1 => Float32x2, // tex_coords
        2 => Float32x4, // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Create orthographic projection matrix
/// Top-left is (0, 0), bottom-right is (width, height)
fn create_projection_matrix(width: f32, height: f32) -> glam::Mat4 {
    glam::Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0)
}

/// Initialize GPU resources (instance, surface, device, queue, surface_config)
///
/// This function handles the 'static lifetime requirement for wgpu::Surface by using
/// Arc<Window> which ensures the window lives as long as needed.
pub fn init_gpu(
    window: Arc<Window>,
) -> Result<
    (
        wgpu::Instance,
        wgpu::Surface<'static>,
        wgpu::Device,
        wgpu::Queue,
        wgpu::SurfaceConfiguration,
    ),
    String,
> {
    // Create wgpu instance with default backends
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // Create surface - the 'static lifetime is satisfied because:
    // 1. Window is wrapped in Arc, so it won't be dropped
    // 2. The surface will be stored in Engine alongside the Arc<Window>
    // 3. Both will live for the entire program duration
    //
    // SAFETY: The window is in an Arc and will be stored in Engine.
    // The surface will also be stored in Engine, and both will be dropped together.
    // This ensures the window outlives the surface.
    let surface = unsafe {
        instance
            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&*window).unwrap())
            .map_err(|e| format!("Failed to create surface: {}", e))?
    };

    // Request adapter
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .map_err(|e| format!("Failed to find an appropriate adapter: {}", e))?;

    // Print adapter info
    let info = adapter.get_info();
    println!(
        "pgfx: Using GPU: {} ({:?}, {:?})",
        info.name, info.backend, info.device_type
    );

    // Request device and queue
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("pgfx Device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        experimental_features: Default::default(),
        trace: Default::default(),
    }))
    .map_err(|e| format!("Failed to create device: {}", e))?;

    // Get surface capabilities
    let surface_caps = surface.get_capabilities(&adapter);

    // Choose a suitable texture format
    // Prefer sRGB formats for correct color rendering
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    // Get window size
    let size = window.inner_size();

    // Configure the surface
    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo, // VSync
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &surface_config);

    Ok((instance, surface, device, queue, surface_config))
}

/// Sprite draw command (parsed from Python)
#[derive(Debug, Clone)]
struct SpriteDrawCommand {
    sprite_id: u32,
    x: f32,
    y: f32,
    rot: f32,
    scale: f32,
    alpha: f32,
    flip_x: bool,
    flip_y: bool,
    z: i32,                            // Z-order for layer sorting (higher = on top)
    color_override: Option<[f32; 4]>,  // Override sprite color (for particles/primitives)
    size_override: Option<(f32, f32)>, // Override size (w, h) for primitives
}

/// Generate vertices for a sprite with transformations
fn generate_sprite_vertices(
    sprite: &crate::sprite::Sprite,
    texture: &crate::texture::Texture,
    cmd: &SpriteDrawCommand,
) -> [SpriteVertex; 6] {
    let (region_x, region_y, region_w, region_h) = sprite.region;
    let (tex_w, tex_h) = texture.size;
    let (origin_x, origin_y) = sprite.origin;

    // Calculate sprite size (use size_override for primitives, or region * scale)
    let (w, h) = if let Some((ow, oh)) = cmd.size_override {
        (ow, oh)
    } else {
        (region_w as f32 * cmd.scale, region_h as f32 * cmd.scale)
    };

    // Apply origin offset (pivot point) - origin is in pixels, scale it
    let ox = origin_x * cmd.scale;
    let oy = origin_y * cmd.scale;

    // Calculate corner positions before rotation (relative to origin)
    let corners = [
        (-ox, -oy),       // top-left
        (w - ox, -oy),    // top-right
        (-ox, h - oy),    // bottom-left
        (w - ox, h - oy), // bottom-right
    ];

    // Apply rotation (inline to avoid allocation)
    let cos = cmd.rot.cos();
    let sin = cmd.rot.sin();

    let rotated_corners: [[f32; 2]; 4] = [
        [
            cmd.x + corners[0].0 * cos - corners[0].1 * sin,
            cmd.y + corners[0].0 * sin + corners[0].1 * cos,
        ],
        [
            cmd.x + corners[1].0 * cos - corners[1].1 * sin,
            cmd.y + corners[1].0 * sin + corners[1].1 * cos,
        ],
        [
            cmd.x + corners[2].0 * cos - corners[2].1 * sin,
            cmd.y + corners[2].0 * sin + corners[2].1 * cos,
        ],
        [
            cmd.x + corners[3].0 * cos - corners[3].1 * sin,
            cmd.y + corners[3].0 * sin + corners[3].1 * cos,
        ],
    ];

    // Calculate UV coordinates
    let u0 = region_x as f32 / tex_w as f32;
    let v0 = region_y as f32 / tex_h as f32;
    let u1 = (region_x + region_w) as f32 / tex_w as f32;
    let v1 = (region_y + region_h) as f32 / tex_h as f32;

    // Apply flip
    let (u0, u1) = if cmd.flip_x { (u1, u0) } else { (u0, u1) };
    let (v0, v1) = if cmd.flip_y { (v1, v0) } else { (v0, v1) };

    // Calculate color with alpha (use override if provided, e.g. for particles)
    let base_color = cmd.color_override.unwrap_or([
        sprite.color[0] as f32 / 255.0,
        sprite.color[1] as f32 / 255.0,
        sprite.color[2] as f32 / 255.0,
        sprite.color[3] as f32 / 255.0,
    ]);
    let alpha = base_color[3] * cmd.alpha;
    // Premultiply color with alpha for correct blending
    let color = [
        base_color[0] * alpha,
        base_color[1] * alpha,
        base_color[2] * alpha,
        alpha,
    ];

    // Create 6 vertices (2 triangles)
    // Triangle 1: top-left, top-right, bottom-left
    // Triangle 2: top-right, bottom-right, bottom-left
    [
        SpriteVertex {
            position: rotated_corners[0],
            tex_coords: [u0, v0],
            color,
        },
        SpriteVertex {
            position: rotated_corners[1],
            tex_coords: [u1, v0],
            color,
        },
        SpriteVertex {
            position: rotated_corners[2],
            tex_coords: [u0, v1],
            color,
        },
        SpriteVertex {
            position: rotated_corners[1],
            tex_coords: [u1, v0],
            color,
        },
        SpriteVertex {
            position: rotated_corners[3],
            tex_coords: [u1, v1],
            color,
        },
        SpriteVertex {
            position: rotated_corners[2],
            tex_coords: [u0, v1],
            color,
        },
    ]
}

/// Create sprite rendering pipeline
/// Returns (pipeline, projection_bind_group_layout, texture_bind_group_layout, projection_buffer)
pub fn create_sprite_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
) -> Result<
    (
        wgpu::RenderPipeline,
        wgpu::BindGroupLayout,
        wgpu::BindGroupLayout,
        wgpu::Buffer,
    ),
    String,
> {
    // Load shader
    let shader_source = include_str!("../shaders/sprite.wgsl");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Sprite Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // Create uniform buffer for projection matrix
    let projection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Sprite Projection Buffer"),
        size: std::mem::size_of::<glam::Mat4>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Create bind group layout for projection uniform (group 0)
    let projection_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sprite Projection Bind Group Layout"),
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
        });

    // Create bind group layout for texture + sampler (group 1)
    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sprite Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

    // Create pipeline layout
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Sprite Pipeline Layout"),
        bind_group_layouts: &[&projection_bind_group_layout, &texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    // Create render pipeline
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Sprite Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[SpriteVertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    // Return pipeline, both bind group layouts, and projection buffer
    Ok((
        pipeline,
        projection_bind_group_layout,
        texture_bind_group_layout,
        projection_buffer,
    ))
}

// Debug timing flag - set to true to print timing info
const DEBUG_TIMING: bool = false;

#[pyfunction]
#[allow(clippy::type_complexity)]
pub fn render_batch(commands: Vec<Py<PyAny>>) -> PyResult<()> {
    use std::time::Instant;
    let t_start = Instant::now();

    // Get primitive sprite IDs first (cached after first call)
    let (white_pixel_sprite, circle_sprite, circle_soft_sprite) =
        crate::engine::with_engine(|engine| {
            let wp = engine
                .get_or_create_primitive_sprite(crate::texture::PrimitiveType::WhitePixel)
                .ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Failed to create white pixel sprite")
                })?;
            let c = engine
                .get_or_create_primitive_sprite(crate::texture::PrimitiveType::Circle)
                .ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Failed to create circle sprite")
                })?;
            let cs = engine
                .get_or_create_primitive_sprite(crate::texture::PrimitiveType::CircleSoft)
                .ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Failed to create circle soft sprite")
                })?;
            PyResult::Ok((wp, c, cs))
        })??;

    // Parse commands - all go into one list preserving order
    let mut clear_color = wgpu::Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let mut sprite_commands: Vec<SpriteDrawCommand> = Vec::new();
    let mut text_draws: Vec<(u32, String, f32, f32, u8, u8, u8, u8, i32)> = Vec::new();
    // light: (light_id, x, y, z)
    let mut light_draws: Vec<(u32, f32, f32, i32)> = Vec::new();
    // particles: (ps_id, z)
    let mut particle_draws: Vec<(u32, i32)> = Vec::new();

    Python::attach(|py| {
        for cmd_obj in commands.iter() {
            let cmd = cmd_obj.bind(py);

            // Commands are tuples: (cmd_type, ...)
            #[allow(deprecated)]
            if let Ok(tuple) = cmd.downcast::<pyo3::types::PyTuple>() {
                if tuple.len() >= 1 {
                    // Get command type
                    if let Ok(cmd_type) = tuple.get_item(0)?.extract::<u8>() {
                        match cmd_type {
                            CMD_CLEAR if tuple.len() >= 5 => {
                                // Parse RGBA values from tuple (r, g, b, a are in 0-255 range)
                                let r = tuple.get_item(1)?.extract::<u8>().unwrap_or(0);
                                let g = tuple.get_item(2)?.extract::<u8>().unwrap_or(0);
                                let b = tuple.get_item(3)?.extract::<u8>().unwrap_or(0);
                                let a = tuple.get_item(4)?.extract::<u8>().unwrap_or(255);

                                // Convert 0-255 to 0.0-1.0
                                clear_color = wgpu::Color {
                                    r: r as f64 / 255.0,
                                    g: g as f64 / 255.0,
                                    b: b as f64 / 255.0,
                                    a: a as f64 / 255.0,
                                };
                            }

                            CMD_RECT_FILL => {
                                // (CMD_RECT_FILL, x, y, w, h, r, g, b, a, z)
                                let x = tuple.get_item(1)?.extract::<f32>()?;
                                let y = tuple.get_item(2)?.extract::<f32>()?;
                                let w = tuple.get_item(3)?.extract::<f32>()?;
                                let h = tuple.get_item(4)?.extract::<f32>()?;
                                let r = tuple.get_item(5)?.extract::<u8>()? as f32 / 255.0;
                                let g = tuple.get_item(6)?.extract::<u8>()? as f32 / 255.0;
                                let b = tuple.get_item(7)?.extract::<u8>()? as f32 / 255.0;
                                let a = tuple.get_item(8)?.extract::<u8>()? as f32 / 255.0;
                                let z = tuple.get_item(9)?.extract::<i32>()?;

                                sprite_commands.push(SpriteDrawCommand {
                                    sprite_id: white_pixel_sprite,
                                    x,
                                    y,
                                    rot: 0.0,
                                    scale: 1.0,
                                    alpha: 1.0,
                                    flip_x: false,
                                    flip_y: false,
                                    z,
                                    color_override: Some([r, g, b, a]),
                                    size_override: Some((w, h)),
                                });
                            }

                            CMD_LINE => {
                                // (CMD_LINE, x1, y1, x2, y2, r, g, b, a, z)
                                let x1 = tuple.get_item(1)?.extract::<f32>()?;
                                let y1 = tuple.get_item(2)?.extract::<f32>()?;
                                let x2 = tuple.get_item(3)?.extract::<f32>()?;
                                let y2 = tuple.get_item(4)?.extract::<f32>()?;
                                let r = tuple.get_item(5)?.extract::<u8>()? as f32 / 255.0;
                                let g = tuple.get_item(6)?.extract::<u8>()? as f32 / 255.0;
                                let b = tuple.get_item(7)?.extract::<u8>()? as f32 / 255.0;
                                let a = tuple.get_item(8)?.extract::<u8>()? as f32 / 255.0;
                                let z = tuple.get_item(9)?.extract::<i32>()?;

                                let dx = x2 - x1;
                                let dy = y2 - y1;
                                let len = (dx * dx + dy * dy).sqrt();
                                if len > 0.0 {
                                    let rot = dy.atan2(dx);
                                    let thickness = 2.0;
                                    sprite_commands.push(SpriteDrawCommand {
                                        sprite_id: white_pixel_sprite,
                                        x: x1,
                                        y: y1 - thickness / 2.0,
                                        rot,
                                        scale: 1.0,
                                        alpha: 1.0,
                                        flip_x: false,
                                        flip_y: false,
                                        z,
                                        color_override: Some([r, g, b, a]),
                                        size_override: Some((len, thickness)),
                                    });
                                }
                            }

                            CMD_CIRCLE_FILL => {
                                // (CMD_CIRCLE_FILL, x, y, radius, r, g, b, a, z)
                                let cx = tuple.get_item(1)?.extract::<f32>()?;
                                let cy = tuple.get_item(2)?.extract::<f32>()?;
                                let radius = tuple.get_item(3)?.extract::<f32>()?;
                                let r = tuple.get_item(4)?.extract::<u8>()? as f32 / 255.0;
                                let g = tuple.get_item(5)?.extract::<u8>()? as f32 / 255.0;
                                let b = tuple.get_item(6)?.extract::<u8>()? as f32 / 255.0;
                                let a = tuple.get_item(7)?.extract::<u8>()? as f32 / 255.0;
                                let z = tuple.get_item(8)?.extract::<i32>()?;

                                // Circle texture is 1024x1024, scale to diameter
                                let diameter = radius * 2.0;
                                sprite_commands.push(SpriteDrawCommand {
                                    sprite_id: circle_sprite,
                                    x: cx,
                                    y: cy,
                                    rot: 0.0,
                                    scale: diameter / 1024.0,
                                    alpha: 1.0,
                                    flip_x: false,
                                    flip_y: false,
                                    z,
                                    color_override: Some([r, g, b, a]),
                                    size_override: None,
                                });
                            }

                            CMD_DRAW => {
                                // (CMD_DRAW, sprite_id, x, y, z)
                                let sprite_id = tuple.get_item(1)?.extract::<u32>()?;
                                let x = tuple.get_item(2)?.extract::<f32>()?;
                                let y = tuple.get_item(3)?.extract::<f32>()?;
                                let z = tuple.get_item(4)?.extract::<i32>()?;

                                sprite_commands.push(SpriteDrawCommand {
                                    sprite_id,
                                    x,
                                    y,
                                    rot: 0.0,
                                    scale: 1.0,
                                    alpha: 1.0,
                                    flip_x: false,
                                    flip_y: false,
                                    z,
                                    color_override: None,
                                    size_override: None,
                                });
                            }

                            CMD_DRAW_EX => {
                                // (CMD_DRAW_EX, sprite_id, x, y, rot, scale, alpha, flip_x, flip_y, z)
                                let sprite_id = tuple.get_item(1)?.extract::<u32>()?;
                                let x = tuple.get_item(2)?.extract::<f32>()?;
                                let y = tuple.get_item(3)?.extract::<f32>()?;
                                let rot = tuple.get_item(4)?.extract::<f32>()?;
                                let scale = tuple.get_item(5)?.extract::<f32>()?;
                                let alpha = tuple.get_item(6)?.extract::<f32>()?;
                                let flip_x = tuple.get_item(7)?.extract::<bool>()?;
                                let flip_y = tuple.get_item(8)?.extract::<bool>()?;
                                let z = tuple.get_item(9)?.extract::<i32>()?;

                                sprite_commands.push(SpriteDrawCommand {
                                    sprite_id,
                                    x,
                                    y,
                                    rot,
                                    scale,
                                    alpha,
                                    flip_x,
                                    flip_y,
                                    z,
                                    color_override: None,
                                    size_override: None,
                                });
                            }

                            CMD_TEXT => {
                                // (CMD_TEXT, font_id, string, x, y, r, g, b, a, z)
                                let font_id = tuple.get_item(1)?.extract::<u32>()?;
                                let text = tuple.get_item(2)?.extract::<String>()?;
                                let x = tuple.get_item(3)?.extract::<f32>()?;
                                let y = tuple.get_item(4)?.extract::<f32>()?;
                                let r = tuple.get_item(5)?.extract::<u8>()?;
                                let g = tuple.get_item(6)?.extract::<u8>()?;
                                let b = tuple.get_item(7)?.extract::<u8>()?;
                                let a = tuple.get_item(8)?.extract::<u8>()?;
                                let z = tuple.get_item(9)?.extract::<i32>()?;

                                text_draws.push((font_id, text, x, y, r, g, b, a, z));
                            }

                            CMD_LIGHT_DRAW => {
                                // (CMD_LIGHT_DRAW, light_id, x, y, z)
                                let light_id = tuple.get_item(1)?.extract::<u32>()?;
                                let x = tuple.get_item(2)?.extract::<f32>()?;
                                let y = tuple.get_item(3)?.extract::<f32>()?;
                                let z = tuple.get_item(4)?.extract::<i32>()?;

                                light_draws.push((light_id, x, y, z));
                            }

                            CMD_PARTICLES_RENDER => {
                                // (CMD_PARTICLES_RENDER, particle_system_id, z)
                                let ps_id = tuple.get_item(1)?.extract::<u32>()?;
                                let z = tuple.get_item(2)?.extract::<i32>()?;

                                particle_draws.push((ps_id, z));
                            }

                            _ => {}
                        }
                    }
                }
            }
        }
        PyResult::Ok(())
    })?;

    let t_parse = t_start.elapsed();
    let sprite_commands_count = sprite_commands.len();

    // Perform rendering with engine's GPU resources
    crate::engine::with_engine(|engine| {
        // Convert light_draws to sprite commands (need engine.lights access)
        let time = engine.start_time.elapsed().as_secs_f32();
        for (light_id, x, y, z) in light_draws {
            if let Some(light) = engine.lights.get(light_id) {
                // Calculate effective intensity with flicker
                let mut intensity = light.intensity;
                if light.flicker_amount > 0.0 {
                    let flicker_phase = time * light.flicker_speed * 10.0;
                    let flicker = (flicker_phase.sin() * 0.5 + 0.5) * light.flicker_amount;
                    intensity *= 1.0 - flicker;
                }

                // Render light as a soft circle sprite
                let diameter = light.radius * 2.0;
                let color = [
                    light.color[0] as f32 / 255.0,
                    light.color[1] as f32 / 255.0,
                    light.color[2] as f32 / 255.0,
                    intensity,
                ];

                sprite_commands.push(SpriteDrawCommand {
                    sprite_id: circle_soft_sprite,
                    x,
                    y,
                    rot: 0.0,
                    scale: diameter / 1024.0, // CircleSoft is 1024px
                    alpha: 1.0,
                    flip_x: false,
                    flip_y: false,
                    z,
                    color_override: Some(color),
                    size_override: None,
                });
            }
        }

        // Convert particle_draws to sprite commands
        for (ps_id, z) in particle_draws {
            if let Some(system) = engine.particle_systems.get(ps_id) {
                if let Some(sprite_id) = system.config.sprite_id {
                    // Render as sprites (textured particles) with particle color
                    let base_size = engine
                        .sprites
                        .get(sprite_id)
                        .map(|s| s.region.2 as f32)
                        .unwrap_or(32.0);
                    for draw_cmd in system.generate_draw_commands(base_size) {
                        sprite_commands.push(SpriteDrawCommand {
                            sprite_id: draw_cmd.1,
                            x: draw_cmd.2,
                            y: draw_cmd.3,
                            rot: draw_cmd.4,
                            scale: draw_cmd.5,
                            alpha: 1.0, // Alpha is in color_override
                            flip_x: draw_cmd.6,
                            flip_y: draw_cmd.7,
                            z,
                            color_override: Some([
                                draw_cmd.8,
                                draw_cmd.9,
                                draw_cmd.10,
                                draw_cmd.11,
                            ]),
                            size_override: None,
                        });
                    }
                } else {
                    // Render primitive particles as white pixel sprites
                    for vert_cmd in system.generate_vertices() {
                        let color = [
                            vert_cmd.5 as f32 / 255.0,
                            vert_cmd.6 as f32 / 255.0,
                            vert_cmd.7 as f32 / 255.0,
                            vert_cmd.8 as f32 / 255.0,
                        ];
                        sprite_commands.push(SpriteDrawCommand {
                            sprite_id: white_pixel_sprite,
                            x: vert_cmd.1,
                            y: vert_cmd.2,
                            rot: 0.0,
                            scale: 1.0,
                            alpha: 1.0,
                            flip_x: false,
                            flip_y: false,
                            z,
                            color_override: Some(color),
                            size_override: Some((vert_cmd.3, vert_cmd.4)),
                        });
                    }
                }
            }
        }

        // Now get GPU resources (immutable borrows)
        let surface = engine
            .surface
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Surface not initialized"))?;
        let device = engine
            .device
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Device not initialized"))?;
        let queue = engine
            .queue
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Queue not initialized"))?;
        let surface_config = engine.surface_config.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Surface config not initialized")
        })?;

        // Get surface texture
        let output = surface.get_current_texture().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to acquire surface texture: {}",
                e
            ))
        })?;

        // Create texture view
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Render sprites, primitives and text (all through sprite pipeline)
            if !sprite_commands.is_empty() || !text_draws.is_empty() {
                // Get sprite pipeline
                let sprite_pipeline = engine.sprite_pipeline.as_ref().ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Sprite pipeline not initialized")
                })?;
                let sprite_bind_group_layout =
                    engine.sprite_bind_group_layout.as_ref().ok_or_else(|| {
                        pyo3::exceptions::PyRuntimeError::new_err(
                            "Sprite bind group layout not initialized",
                        )
                    })?;
                let sprite_projection_buffer =
                    engine.sprite_projection_buffer.as_ref().ok_or_else(|| {
                        pyo3::exceptions::PyRuntimeError::new_err(
                            "Sprite projection buffer not initialized",
                        )
                    })?;

                // Update projection matrix
                let projection = create_projection_matrix(
                    surface_config.width as f32,
                    surface_config.height as f32,
                );
                queue.write_buffer(
                    sprite_projection_buffer,
                    0,
                    bytemuck::cast_slice(projection.as_ref()),
                );

                // Create projection bind group
                let projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Sprite Projection Bind Group"),
                    layout: sprite_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: sprite_projection_buffer.as_entire_binding(),
                    }],
                });

                let t_sprite_start = Instant::now();

                // Group sprites by texture for batching, track offsets
                let mut all_vertices: Vec<SpriteVertex> =
                    Vec::with_capacity((sprite_commands.len() + text_draws.len() * 20) * 6);
                let mut batches: Vec<(u32, u32, u32)> = Vec::new(); // (texture_id, start, count)

                // Sort by z only, stable sort preserves insertion order within same z
                // Batching works for consecutive commands with same texture
                let mut sorted_commands = sprite_commands;
                sorted_commands.sort_by_key(|cmd| cmd.z);

                let mut current_texture_id: Option<u32> = None;
                let mut batch_start = 0u32;

                for cmd in &sorted_commands {
                    // Get sprite
                    let sprite = engine.sprites.get(cmd.sprite_id).ok_or_else(|| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "Invalid sprite ID: {}",
                            cmd.sprite_id
                        ))
                    })?;

                    // Get texture
                    let texture = engine.textures.get(sprite.texture_id).ok_or_else(|| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "Invalid texture ID: {}",
                            sprite.texture_id
                        ))
                    })?;

                    // Check if texture changed
                    if current_texture_id != Some(sprite.texture_id) {
                        // Save previous batch if any
                        if let Some(tex_id) = current_texture_id {
                            let count = all_vertices.len() as u32 - batch_start;
                            if count > 0 {
                                batches.push((tex_id, batch_start, count));
                            }
                        }
                        current_texture_id = Some(sprite.texture_id);
                        batch_start = all_vertices.len() as u32;
                    }

                    // Generate vertices
                    let verts = generate_sprite_vertices(sprite, texture, cmd);
                    all_vertices.extend_from_slice(&verts);
                }

                // Save last sprite batch
                if let Some(tex_id) = current_texture_id {
                    let count = all_vertices.len() as u32 - batch_start;
                    if count > 0 {
                        batches.push((tex_id, batch_start, count));
                    }
                }

                // Generate text vertices directly (no temporary sprites!)
                // TODO: text should also participate in z-sorting
                for (font_id, text, x, y, r, g, b, a, _z) in &text_draws {
                    let font = match engine.fonts.get(*font_id) {
                        Some(f) => f,
                        None => continue,
                    };

                    let atlas_texture = match engine.textures.get(font.atlas_texture_id) {
                        Some(t) => t,
                        None => continue,
                    };
                    let (_atlas_w, _atlas_h) = atlas_texture.size;

                    // Save batch for font atlas texture
                    if current_texture_id != Some(font.atlas_texture_id) {
                        if let Some(tex_id) = current_texture_id {
                            let count = all_vertices.len() as u32 - batch_start;
                            if count > 0 {
                                batches.push((tex_id, batch_start, count));
                            }
                        }
                        current_texture_id = Some(font.atlas_texture_id);
                        batch_start = all_vertices.len() as u32;
                    }

                    let alpha = *a as f32 / 255.0;
                    // Premultiply color with alpha for correct blending
                    let color = [
                        *r as f32 / 255.0 * alpha,
                        *g as f32 / 255.0 * alpha,
                        *b as f32 / 255.0 * alpha,
                        alpha,
                    ];

                    // For pixel-perfect fonts, round base coordinates
                    let (base_x, base_y) = if font.smooth {
                        (*x, *y)
                    } else {
                        (x.floor(), y.floor())
                    };
                    let mut cursor_x = base_x;
                    let cursor_y = base_y;

                    for ch in text.chars() {
                        if let Some(glyph_info) = font.glyphs.get(&ch) {
                            // For pixel-perfect fonts, round glyph positions
                            let (glyph_x, glyph_y) = if font.smooth {
                                (
                                    cursor_x + glyph_info.offset.0,
                                    cursor_y + glyph_info.offset.1,
                                )
                            } else {
                                (
                                    (cursor_x + glyph_info.offset.0).floor(),
                                    (cursor_y + glyph_info.offset.1).floor(),
                                )
                            };

                            let (u0, v0, u1, v1) = glyph_info.uv;
                            let (glyph_w, glyph_h) = glyph_info.size;

                            // Generate 6 vertices for glyph quad (2 triangles)
                            let x0 = glyph_x;
                            let y0 = glyph_y;
                            let x1 = glyph_x + glyph_w;
                            let y1 = glyph_y + glyph_h;

                            // Triangle 1: top-left, top-right, bottom-left
                            all_vertices.push(SpriteVertex {
                                position: [x0, y0],
                                tex_coords: [u0, v0],
                                color,
                            });
                            all_vertices.push(SpriteVertex {
                                position: [x1, y0],
                                tex_coords: [u1, v0],
                                color,
                            });
                            all_vertices.push(SpriteVertex {
                                position: [x0, y1],
                                tex_coords: [u0, v1],
                                color,
                            });

                            // Triangle 2: top-right, bottom-right, bottom-left
                            all_vertices.push(SpriteVertex {
                                position: [x1, y0],
                                tex_coords: [u1, v0],
                                color,
                            });
                            all_vertices.push(SpriteVertex {
                                position: [x1, y1],
                                tex_coords: [u1, v1],
                                color,
                            });
                            all_vertices.push(SpriteVertex {
                                position: [x0, y1],
                                tex_coords: [u0, v1],
                                color,
                            });

                            cursor_x += glyph_info.advance;
                        }
                    }
                }

                // Save last text batch
                if let Some(tex_id) = current_texture_id {
                    let count = all_vertices.len() as u32 - batch_start;
                    if count > 0 {
                        batches.push((tex_id, batch_start, count));
                    }
                }

                // Create or reuse vertex buffer
                let total_vertices = all_vertices.len();
                if total_vertices > 0 {
                    let needs_new_buffer = engine.sprite_vertex_buffer.is_none()
                        || engine.sprite_vertex_buffer_capacity < total_vertices;

                    if needs_new_buffer {
                        // Create new buffer with some extra capacity
                        let new_capacity = (total_vertices * 2).max(1024);
                        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Sprite Vertex Buffer"),
                            size: (new_capacity * std::mem::size_of::<SpriteVertex>()) as u64,
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        });
                        engine.sprite_vertex_buffer = Some(buffer);
                        engine.sprite_vertex_buffer_capacity = new_capacity;
                    }

                    let t_vertex_gen = t_sprite_start.elapsed();

                    // Write vertices to buffer
                    let vertex_buffer = engine.sprite_vertex_buffer.as_ref().unwrap();
                    queue.write_buffer(vertex_buffer, 0, bytemuck::cast_slice(&all_vertices));

                    let t_buffer_write = t_sprite_start.elapsed();

                    if DEBUG_TIMING && sprite_commands_count > 100 {
                        static FRAME_COUNT: std::sync::atomic::AtomicU64 =
                            std::sync::atomic::AtomicU64::new(0);
                        let frame = FRAME_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if frame.is_multiple_of(60) {
                            println!("TIMING: parse={:.2}ms, vertex_gen={:.2}ms, buf_write={:.2}ms, sprites={}",
                                t_parse.as_secs_f64() * 1000.0,
                                t_vertex_gen.as_secs_f64() * 1000.0,
                                (t_buffer_write - t_vertex_gen).as_secs_f64() * 1000.0,
                                sprite_commands_count);
                        }
                    }

                    // Render all batches
                    render_pass.set_pipeline(sprite_pipeline);
                    render_pass.set_bind_group(0, &projection_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

                    for (texture_id, start, count) in batches {
                        // Get texture with cached bind group
                        let texture = engine.textures.get(texture_id).ok_or_else(|| {
                            pyo3::exceptions::PyRuntimeError::new_err(format!(
                                "Invalid texture ID: {}",
                                texture_id
                            ))
                        })?;

                        // Use cached bind group
                        let bind_group = texture.bind_group.as_ref().ok_or_else(|| {
                            pyo3::exceptions::PyRuntimeError::new_err(
                                "Texture bind group not initialized",
                            )
                        })?;

                        render_pass.set_bind_group(1, bind_group, &[]);
                        render_pass.draw(start..start + count, 0..1);
                    }
                }
            }

            // Render pass ends here (when render_pass is dropped)
        }

        // Submit commands to queue
        queue.submit(std::iter::once(encoder.finish()));

        // Present the surface texture
        output.present();

        let _t_total = t_start.elapsed();

        // Timing output is now inside the sprite rendering block

        Ok(())
    })?
}

/// Render a single black frame - used to clear garbage on initialization
pub fn render_initial_frame(
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Result<(), String> {
    // Get surface texture
    let output = surface
        .get_current_texture()
        .map_err(|e| format!("Failed to acquire surface texture: {}", e))?;

    // Create texture view
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    // Create command encoder
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Initial Clear Encoder"),
    });

    // Begin render pass - just clear to black
    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Initial Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        // Pass ends here, we just wanted to clear
    }

    queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
