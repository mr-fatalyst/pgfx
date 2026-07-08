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
pub const CMD_SET_VIEW: u8 = 9;
pub const CMD_RESET_VIEW: u8 = 10;
pub const CMD_RECT_FILL_EX: u8 = 11;
pub const CMD_RENDER_TO: u8 = 12;

// Blend modes for sprite/particle draws
pub const BLEND_ALPHA: u8 = 0;
pub const BLEND_ADD: u8 = 1;

// Text alignment (the x of a text() call anchors each line's left edge,
// center or right edge)
#[allow(dead_code)] // documents the wire format; it's the match fallback arm
pub const TEXT_ALIGN_LEFT: u8 = 0;
pub const TEXT_ALIGN_CENTER: u8 = 1;
pub const TEXT_ALIGN_RIGHT: u8 = 2;

/// 2D affine view transform: screen = (a*x + c*y + tx, b*x + d*y + ty),
/// stored as [a, b, c, d, tx, ty]. `None` on a command means identity
/// (screen space).
type View = [f32; 6];

/// Build the view that places world point (cx, cy) at the screen center,
/// zoomed and rotated around it. Screen size is in logical pixels.
fn make_view(cx: f32, cy: f32, zoom: f32, rot: f32, screen_w: f32, screen_h: f32) -> View {
    let (sin, cos) = rot.sin_cos();
    let (a, b, c, d) = (zoom * cos, zoom * sin, -zoom * sin, zoom * cos);
    [
        a,
        b,
        c,
        d,
        screen_w / 2.0 - (a * cx + c * cy),
        screen_h / 2.0 - (b * cx + d * cy),
    ]
}

#[inline]
fn view_apply(v: &View, x: f32, y: f32) -> [f32; 2] {
    [v[0] * x + v[2] * y + v[4], v[1] * x + v[3] * y + v[5]]
}

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
pub(crate) fn create_projection_matrix(width: f32, height: f32) -> glam::Mat4 {
    glam::Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0)
}

/// Convert an sRGB-encoded channel (0.0-1.0) to linear.
/// User-facing colors are sRGB (what you'd pick in an image editor); the GPU
/// blends and outputs in linear space, so every color entering a vertex or a
/// clear value must be linearized exactly once. Alpha stays linear (coverage).
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert an 8-bit sRGB channel to linear f32
fn srgb8_to_linear(c: u8) -> f32 {
    srgb_to_linear(c as f32 / 255.0)
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
    scale: f32,   // horizontal scale
    scale_y: f32, // vertical scale (usually equal to scale)
    alpha: f32,
    flip_x: bool,
    flip_y: bool,
    z: i32,                            // Z-order for layer sorting (higher = on top)
    seq: u32,                          // Position in the frame's command list (call order)
    color_override: Option<[f32; 4]>,  // Override sprite color (for particles/primitives)
    size_override: Option<(f32, f32)>, // Override size (w, h) for primitives
    view: Option<View>,                // View transform active when the call was issued
    blend: u8,                         // BLEND_ALPHA or BLEND_ADD
}

/// Text draw command (parsed from Python)
#[derive(Debug, Clone)]
struct TextDrawCommand {
    font_id: u32,
    text: String,
    x: f32,
    y: f32,
    color: [u8; 4],
    z: i32,
    seq: u32,
    view: Option<View>,
    align: u8,
}

/// A single draw command in the unified stream. Everything is sorted by
/// (z, seq) together, so same-z commands keep the user's call order across
/// sprites, primitives, lights, particles and text.
enum DrawItem {
    Sprite(SpriteDrawCommand),
    Text(TextDrawCommand),
}

impl DrawItem {
    fn order_key(&self) -> (i32, u32) {
        match self {
            DrawItem::Sprite(cmd) => (cmd.z, cmd.seq),
            DrawItem::Text(cmd) => (cmd.z, cmd.seq),
        }
    }
}

/// Generate vertices for a sprite with transformations.
/// `texture_size` is the full texture size in pixels (for UV normalization).
fn generate_sprite_vertices(
    sprite: &crate::sprite::Sprite,
    texture_size: (u32, u32),
    cmd: &SpriteDrawCommand,
) -> [SpriteVertex; 6] {
    let (region_x, region_y, region_w, region_h) = sprite.region;
    let (tex_w, tex_h) = texture_size;
    let (origin_x, origin_y) = sprite.origin;

    // Calculate sprite size (use size_override for primitives, or region * scale)
    let (w, h) = if let Some((ow, oh)) = cmd.size_override {
        (ow, oh)
    } else {
        (region_w as f32 * cmd.scale, region_h as f32 * cmd.scale_y)
    };

    // Apply origin offset (pivot point) - origin is in pixels, scale it
    let ox = origin_x * cmd.scale;
    let oy = origin_y * cmd.scale_y;

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

    let mut rotated_corners: [[f32; 2]; 4] = [
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

    // Apply the view transform (camera) captured at call time
    if let Some(v) = &cmd.view {
        for corner in &mut rotated_corners {
            *corner = view_apply(v, corner[0], corner[1]);
        }
    }

    // Calculate UV coordinates
    let u0 = region_x as f32 / tex_w as f32;
    let v0 = region_y as f32 / tex_h as f32;
    let u1 = (region_x + region_w) as f32 / tex_w as f32;
    let v1 = (region_y + region_h) as f32 / tex_h as f32;

    // Apply flip
    let (u0, u1) = if cmd.flip_x { (u1, u0) } else { (u0, u1) };
    let (v0, v1) = if cmd.flip_y { (v1, v0) } else { (v0, v1) };

    // Calculate color with alpha (use override if provided, e.g. for particles).
    // Vertex colors are straight-alpha and linear: RGB is linearized here
    // (inputs are sRGB), premultiplication happens exactly once in the
    // fragment shader (shaders/sprite.wgsl).
    let base_color = cmd.color_override.unwrap_or([
        sprite.color[0] as f32 / 255.0,
        sprite.color[1] as f32 / 255.0,
        sprite.color[2] as f32 / 255.0,
        sprite.color[3] as f32 / 255.0,
    ]);
    let alpha = base_color[3] * cmd.alpha;
    let color = [
        srgb_to_linear(base_color[0]),
        srgb_to_linear(base_color[1]),
        srgb_to_linear(base_color[2]),
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

/// Append glyph quads for a text command to the vertex list.
/// Supports '\n' (new line via the font's line height) and kerning.
/// `atlas_size` is the current atlas texture size for UV normalization.
fn generate_text_vertices(
    font: &crate::text::Font,
    atlas_size: (u32, u32),
    cmd: &TextDrawCommand,
    out: &mut Vec<SpriteVertex>,
) {
    // Straight-alpha linear vertex color; the shader premultiplies once
    let alpha = cmd.color[3] as f32 / 255.0;
    let color = [
        srgb8_to_linear(cmd.color[0]),
        srgb8_to_linear(cmd.color[1]),
        srgb8_to_linear(cmd.color[2]),
        alpha,
    ];

    // For pixel-perfect fonts, round base coordinates
    let (base_x, base_y) = if font.smooth {
        (cmd.x, cmd.y)
    } else {
        (cmd.x.floor(), cmd.y.floor())
    };
    let (atlas_w, atlas_h) = (atlas_size.0 as f32, atlas_size.1 as f32);

    let mut cursor_y = base_y;
    for line in cmd.text.split('\n') {
        // x anchors each line's left edge, center or right edge
        let mut cursor_x = match cmd.align {
            TEXT_ALIGN_CENTER => base_x - font.line_width(line) / 2.0,
            TEXT_ALIGN_RIGHT => base_x - font.line_width(line),
            _ => base_x,
        };
        if !font.smooth {
            cursor_x = cursor_x.floor();
        }
        let mut prev_ch: Option<char> = None;

        for ch in line.chars() {
            if ch.is_control() {
                continue;
            }

            if let Some(prev) = prev_ch {
                if let Some(kern) = font.kern(prev, ch) {
                    cursor_x += kern;
                }
            }
            prev_ch = Some(ch);

            let glyph_info = match font.glyphs.get(&ch) {
                Some(g) => g,
                None => continue,
            };

            let (rect_x, rect_y, rect_w, rect_h) = glyph_info.rect;
            // Empty rect = whitespace or unrenderable glyph: advance only
            if rect_w > 0 && rect_h > 0 {
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

                let u0 = rect_x as f32 / atlas_w;
                let v0 = rect_y as f32 / atlas_h;
                let u1 = (rect_x + rect_w) as f32 / atlas_w;
                let v1 = (rect_y + rect_h) as f32 / atlas_h;

                // Two triangles per glyph quad, through the view transform
                // (camera) captured at call time
                let x0 = glyph_x;
                let y0 = glyph_y;
                let x1 = glyph_x + rect_w as f32;
                let y1 = glyph_y + rect_h as f32;
                let (p00, p10, p01, p11) = match &cmd.view {
                    Some(v) => (
                        view_apply(v, x0, y0),
                        view_apply(v, x1, y0),
                        view_apply(v, x0, y1),
                        view_apply(v, x1, y1),
                    ),
                    None => ([x0, y0], [x1, y0], [x0, y1], [x1, y1]),
                };

                out.push(SpriteVertex {
                    position: p00,
                    tex_coords: [u0, v0],
                    color,
                });
                out.push(SpriteVertex {
                    position: p10,
                    tex_coords: [u1, v0],
                    color,
                });
                out.push(SpriteVertex {
                    position: p01,
                    tex_coords: [u0, v1],
                    color,
                });
                out.push(SpriteVertex {
                    position: p10,
                    tex_coords: [u1, v0],
                    color,
                });
                out.push(SpriteVertex {
                    position: p11,
                    tex_coords: [u1, v1],
                    color,
                });
                out.push(SpriteVertex {
                    position: p01,
                    tex_coords: [u0, v1],
                    color,
                });
            }

            cursor_x += glyph_info.advance;
        }

        cursor_y += font.line_height;
    }
}

/// Create sprite rendering pipeline
/// Returns (pipeline, projection_bind_group_layout, texture_bind_group_layout, projection_buffer)
pub fn create_sprite_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
) -> Result<
    (
        wgpu::RenderPipeline,
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

    // Two identical pipelines that differ only in the blend state: shader
    // output is premultiplied, so additive is simply (One, One)
    let make_pipeline = |label: &str, blend: wgpu::BlendState| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
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
                    blend: Some(blend),
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
        })
    };

    let pipeline = make_pipeline(
        "Sprite Render Pipeline",
        wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
    );
    let additive = wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
    };
    let pipeline_add = make_pipeline("Sprite Render Pipeline (additive)", additive);

    // Return pipelines, both bind group layouts, and projection buffer
    Ok((
        pipeline,
        pipeline_add,
        projection_bind_group_layout,
        texture_bind_group_layout,
        projection_buffer,
    ))
}

/// Format of the offscreen lightmap: linear (non-sRGB) accumulation buffer
pub const LIGHTMAP_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Close the current batch and start one for the given (texture, blend) pair
fn switch_batch(
    batches: &mut Vec<(u32, u32, u32, u8)>,
    current_key: &mut Option<(u32, u8)>,
    batch_start: &mut u32,
    vertices_len: u32,
    texture_id: u32,
    blend: u8,
) {
    if *current_key != Some((texture_id, blend)) {
        if let Some((tex_id, prev_blend)) = *current_key {
            let count = vertices_len - *batch_start;
            if count > 0 {
                batches.push((tex_id, *batch_start, count, prev_blend));
            }
        }
        *current_key = Some((texture_id, blend));
        *batch_start = vertices_len;
    }
}

/// Draw a batch list into an already-configured render pass, switching
/// between the alpha and additive pipelines as needed (they share a layout,
/// so bind groups survive the switch)
fn draw_sprite_batches<'p>(
    pass: &mut wgpu::RenderPass<'p>,
    batches: &[(u32, u32, u32, u8)],
    pipeline: &'p wgpu::RenderPipeline,
    pipeline_add: &'p wgpu::RenderPipeline,
    projection_bind_group: &'p wgpu::BindGroup,
    vertex_buffer: &'p wgpu::Buffer,
    textures: &'p crate::resources::ResourcePool<crate::texture::Texture>,
) -> PyResult<()> {
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, projection_bind_group, &[]);
    pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    let mut current_blend = BLEND_ALPHA;

    for &(texture_id, start, count, blend) in batches {
        if blend != current_blend {
            pass.set_pipeline(if blend == BLEND_ADD {
                pipeline_add
            } else {
                pipeline
            });
            current_blend = blend;
        }
        let bind_group = textures
            .get(texture_id)
            .and_then(|t| t.bind_group.as_ref())
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Invalid texture ID: {}",
                    texture_id
                ))
            })?;
        pass.set_bind_group(1, bind_group, &[]);
        pass.draw(start..start + count, 0..1);
    }
    Ok(())
}

/// Create the two lighting pipelines:
/// - light pipeline: sprite shader rendering additively into the lightmap
/// - multiply pipeline: fullscreen pass multiplying the surface by the lightmap
///
/// Returns (light_pipeline, multiply_pipeline, multiply_bind_group_layout, lightmap_sampler).
pub fn create_lighting_pipelines(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    projection_bind_group_layout: &wgpu::BindGroupLayout,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
) -> (
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::BindGroupLayout,
    wgpu::Sampler,
) {
    // Light pipeline: same sprite shader/vertex layout, additive blending,
    // rendering into the (linear) lightmap texture
    let sprite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Sprite Shader (lights)"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sprite.wgsl").into()),
    });

    let light_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Light Pipeline Layout"),
        bind_group_layouts: &[projection_bind_group_layout, texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let additive = wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
    };

    let light_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Light Render Pipeline"),
        layout: Some(&light_layout),
        vertex: wgpu::VertexState {
            module: &sprite_shader,
            entry_point: Some("vs_main"),
            buffers: &[SpriteVertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &sprite_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: LIGHTMAP_FORMAT,
                blend: Some(additive),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    // Multiply pipeline: fullscreen triangle, scene *= lightmap
    let blit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Lightmap Multiply Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/lightmap.wgsl").into()),
    });

    let multiply_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Lightmap Bind Group Layout"),
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

    let multiply_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Lightmap Multiply Pipeline Layout"),
        bind_group_layouts: &[&multiply_bind_group_layout],
        push_constant_ranges: &[],
    });

    let multiply = wgpu::BlendState {
        // final = dst (scene) * src (lightmap); blending on an sRGB surface
        // happens in linear space, so the multiply is gamma-correct
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Dst,
            dst_factor: wgpu::BlendFactor::Zero,
            operation: wgpu::BlendOperation::Add,
        },
        // keep the destination alpha
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Zero,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
    };

    let multiply_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Lightmap Multiply Pipeline"),
        layout: Some(&multiply_layout),
        vertex: wgpu::VertexState {
            module: &blit_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &blit_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(multiply),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let lightmap_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    (
        light_pipeline,
        multiply_pipeline,
        multiply_bind_group_layout,
        lightmap_sampler,
    )
}

// Debug timing flag - set to true to print timing info
const DEBUG_TIMING: bool = false;

#[pyfunction]
#[allow(clippy::type_complexity)]
pub fn render_batch(commands: Vec<Py<PyAny>>) -> PyResult<()> {
    use std::time::Instant;
    let t_start = Instant::now();

    // Get primitive sprite IDs (cached after first call) and the logical
    // screen size — set_view places the camera point at the screen center
    let (white_pixel_sprite, circle_sprite, circle_soft_sprite, screen_w, screen_h) =
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
            let (sw, sh) = if let Some(config) = &engine.surface_config {
                let scale = engine.scale_factor;
                (
                    (config.width as f64 / scale) as f32,
                    (config.height as f64 / scale) as f32,
                )
            } else {
                (engine.config.width as f32, engine.config.height as f32)
            };
            PyResult::Ok((wp, c, cs, sw, sh))
        })??;

    // Parse commands into per-destination buckets: bucket 0 is the screen,
    // render_to() opens one per target. Within a bucket each item carries its
    // position in the command list (seq) so same-z draws keep call order.
    // Bucket: (target_id 0=screen, items, clear color set this frame)
    let mut buckets: Vec<(u32, Vec<DrawItem>, Option<wgpu::Color>)> =
        vec![(0, Vec::with_capacity(commands.len()), None)];
    let mut bucket_idx = 0usize;
    // light: (light_id, x, y, view) — rendered additively into the lightmap,
    // so their draw order (and z) is irrelevant; lighting is a screen-space
    // pass and ignores render_to()
    let mut light_draws: Vec<(u32, f32, f32, Option<View>)> = Vec::new();
    // particles: (ps_id, z, seq, view, bucket)
    let mut particle_draws: Vec<(u32, i32, u32, Option<View>, usize)> = Vec::new();
    // The view (camera) applies to every draw command issued after set_view()
    // and until reset_view(); each frame (and each render_to switch) starts
    // in screen space. set_view centers on the current destination's size.
    let mut current_view: Option<View> = None;
    let mut view_size = (screen_w, screen_h);

    Python::attach(|py| {
        for (seq, cmd_obj) in commands.iter().enumerate() {
            let seq = seq as u32;
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

                                // sRGB input -> linear clear value (sRGB target);
                                // applies to the current render_to destination
                                buckets[bucket_idx].2 = Some(wgpu::Color {
                                    r: srgb8_to_linear(r) as f64,
                                    g: srgb8_to_linear(g) as f64,
                                    b: srgb8_to_linear(b) as f64,
                                    a: a as f64 / 255.0,
                                });
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

                                buckets[bucket_idx].1.push(DrawItem::Sprite(SpriteDrawCommand {
                                    sprite_id: white_pixel_sprite,
                                    x,
                                    y,
                                    rot: 0.0,
                                    scale: 1.0,
                                    scale_y: 1.0,
                                    alpha: 1.0,
                                    flip_x: false,
                                    flip_y: false,
                                    z,
                                    seq,
                                    color_override: Some([r, g, b, a]),
                                    size_override: Some((w, h)),
                                    view: current_view,
                                    blend: BLEND_ALPHA,
                                }));
                            }

                            CMD_RECT_FILL_EX => {
                                // (CMD_RECT_FILL_EX, x, y, w, h, rot, r, g, b, a, z)
                                // x, y is the CENTER; rotation is around it
                                let cx = tuple.get_item(1)?.extract::<f32>()?;
                                let cy = tuple.get_item(2)?.extract::<f32>()?;
                                let w = tuple.get_item(3)?.extract::<f32>()?;
                                let h = tuple.get_item(4)?.extract::<f32>()?;
                                let rot = tuple.get_item(5)?.extract::<f32>()?;
                                let r = tuple.get_item(6)?.extract::<u8>()? as f32 / 255.0;
                                let g = tuple.get_item(7)?.extract::<u8>()? as f32 / 255.0;
                                let b = tuple.get_item(8)?.extract::<u8>()? as f32 / 255.0;
                                let a = tuple.get_item(9)?.extract::<u8>()? as f32 / 255.0;
                                let z = tuple.get_item(10)?.extract::<i32>()?;

                                // The quad's own origin is its top-left corner:
                                // place it so the rect stays centered after rotation
                                let (sin, cos) = rot.sin_cos();
                                buckets[bucket_idx].1.push(DrawItem::Sprite(SpriteDrawCommand {
                                    sprite_id: white_pixel_sprite,
                                    x: cx - (w * cos - h * sin) / 2.0,
                                    y: cy - (w * sin + h * cos) / 2.0,
                                    rot,
                                    scale: 1.0,
                                    scale_y: 1.0,
                                    alpha: 1.0,
                                    flip_x: false,
                                    flip_y: false,
                                    z,
                                    seq,
                                    color_override: Some([r, g, b, a]),
                                    size_override: Some((w, h)),
                                    view: current_view,
                                    blend: BLEND_ALPHA,
                                }));
                            }

                            CMD_LINE => {
                                // (CMD_LINE, x1, y1, x2, y2, r, g, b, a, z, width)
                                let x1 = tuple.get_item(1)?.extract::<f32>()?;
                                let y1 = tuple.get_item(2)?.extract::<f32>()?;
                                let x2 = tuple.get_item(3)?.extract::<f32>()?;
                                let y2 = tuple.get_item(4)?.extract::<f32>()?;
                                let r = tuple.get_item(5)?.extract::<u8>()? as f32 / 255.0;
                                let g = tuple.get_item(6)?.extract::<u8>()? as f32 / 255.0;
                                let b = tuple.get_item(7)?.extract::<u8>()? as f32 / 255.0;
                                let a = tuple.get_item(8)?.extract::<u8>()? as f32 / 255.0;
                                let z = tuple.get_item(9)?.extract::<i32>()?;
                                let width = tuple.get_item(10)?.extract::<f32>()?;

                                let dx = x2 - x1;
                                let dy = y2 - y1;
                                let len = (dx * dx + dy * dy).sqrt();
                                if len > 0.0 && width > 0.0 {
                                    let rot = dy.atan2(dx);
                                    // Center the quad on the line: offset by half
                                    // the width perpendicular to the direction
                                    let (sin, cos) = rot.sin_cos();
                                    buckets[bucket_idx].1.push(DrawItem::Sprite(SpriteDrawCommand {
                                        sprite_id: white_pixel_sprite,
                                        x: x1 + sin * width / 2.0,
                                        y: y1 - cos * width / 2.0,
                                        rot,
                                        scale: 1.0,
                                        scale_y: 1.0,
                                        alpha: 1.0,
                                        flip_x: false,
                                        flip_y: false,
                                        z,
                                        seq,
                                        color_override: Some([r, g, b, a]),
                                        size_override: Some((len, width)),
                                        view: current_view,
                                        blend: BLEND_ALPHA,
                                    }));
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
                                buckets[bucket_idx].1.push(DrawItem::Sprite(SpriteDrawCommand {
                                    sprite_id: circle_sprite,
                                    x: cx,
                                    y: cy,
                                    rot: 0.0,
                                    scale: diameter / 1024.0,
                                    scale_y: diameter / 1024.0,
                                    alpha: 1.0,
                                    flip_x: false,
                                    flip_y: false,
                                    z,
                                    seq,
                                    color_override: Some([r, g, b, a]),
                                    size_override: None,
                                    view: current_view,
                                    blend: BLEND_ALPHA,
                                }));
                            }

                            CMD_DRAW => {
                                // (CMD_DRAW, sprite_id, x, y, z)
                                let sprite_id = tuple.get_item(1)?.extract::<u32>()?;
                                let x = tuple.get_item(2)?.extract::<f32>()?;
                                let y = tuple.get_item(3)?.extract::<f32>()?;
                                let z = tuple.get_item(4)?.extract::<i32>()?;

                                buckets[bucket_idx].1.push(DrawItem::Sprite(SpriteDrawCommand {
                                    sprite_id,
                                    x,
                                    y,
                                    rot: 0.0,
                                    scale: 1.0,
                                    scale_y: 1.0,
                                    alpha: 1.0,
                                    flip_x: false,
                                    flip_y: false,
                                    z,
                                    seq,
                                    color_override: None,
                                    size_override: None,
                                    view: current_view,
                                    blend: BLEND_ALPHA,
                                }));
                            }

                            CMD_DRAW_EX => {
                                // (CMD_DRAW_EX, sprite_id, x, y, rot, scale, alpha,
                                //  flip_x, flip_y, z, scale_y, blend)
                                let sprite_id = tuple.get_item(1)?.extract::<u32>()?;
                                let x = tuple.get_item(2)?.extract::<f32>()?;
                                let y = tuple.get_item(3)?.extract::<f32>()?;
                                let rot = tuple.get_item(4)?.extract::<f32>()?;
                                let scale = tuple.get_item(5)?.extract::<f32>()?;
                                let alpha = tuple.get_item(6)?.extract::<f32>()?;
                                let flip_x = tuple.get_item(7)?.extract::<bool>()?;
                                let flip_y = tuple.get_item(8)?.extract::<bool>()?;
                                let z = tuple.get_item(9)?.extract::<i32>()?;
                                let scale_y = tuple.get_item(10)?.extract::<f32>()?;
                                let blend = tuple.get_item(11)?.extract::<u8>()?;

                                buckets[bucket_idx].1.push(DrawItem::Sprite(SpriteDrawCommand {
                                    sprite_id,
                                    x,
                                    y,
                                    rot,
                                    scale,
                                    scale_y,
                                    alpha,
                                    flip_x,
                                    flip_y,
                                    z,
                                    seq,
                                    color_override: None,
                                    size_override: None,
                                    view: current_view,
                                    blend,
                                }));
                            }

                            CMD_TEXT => {
                                // (CMD_TEXT, font_id, string, x, y, r, g, b, a, z, align)
                                let font_id = tuple.get_item(1)?.extract::<u32>()?;
                                let text = tuple.get_item(2)?.extract::<String>()?;
                                let x = tuple.get_item(3)?.extract::<f32>()?;
                                let y = tuple.get_item(4)?.extract::<f32>()?;
                                let r = tuple.get_item(5)?.extract::<u8>()?;
                                let g = tuple.get_item(6)?.extract::<u8>()?;
                                let b = tuple.get_item(7)?.extract::<u8>()?;
                                let a = tuple.get_item(8)?.extract::<u8>()?;
                                let z = tuple.get_item(9)?.extract::<i32>()?;
                                let align = tuple.get_item(10)?.extract::<u8>()?;

                                buckets[bucket_idx].1.push(DrawItem::Text(TextDrawCommand {
                                    font_id,
                                    text,
                                    x,
                                    y,
                                    color: [r, g, b, a],
                                    z,
                                    seq,
                                    view: current_view,
                                    align,
                                }));
                            }

                            CMD_LIGHT_DRAW => {
                                // (CMD_LIGHT_DRAW, light_id, x, y, z) — z ignored:
                                // lights are additive and order-independent
                                let light_id = tuple.get_item(1)?.extract::<u32>()?;
                                let x = tuple.get_item(2)?.extract::<f32>()?;
                                let y = tuple.get_item(3)?.extract::<f32>()?;

                                light_draws.push((light_id, x, y, current_view));
                            }

                            CMD_PARTICLES_RENDER => {
                                // (CMD_PARTICLES_RENDER, particle_system_id, z)
                                let ps_id = tuple.get_item(1)?.extract::<u32>()?;
                                let z = tuple.get_item(2)?.extract::<i32>()?;

                                particle_draws.push((ps_id, z, seq, current_view, bucket_idx));
                            }

                            CMD_SET_VIEW => {
                                // (CMD_SET_VIEW, x, y, zoom, rot) — camera for
                                // all subsequent commands until reset_view()
                                let x = tuple.get_item(1)?.extract::<f32>()?;
                                let y = tuple.get_item(2)?.extract::<f32>()?;
                                let zoom = tuple.get_item(3)?.extract::<f32>()?;
                                let rot = tuple.get_item(4)?.extract::<f32>()?;
                                current_view =
                                    Some(make_view(x, y, zoom, rot, view_size.0, view_size.1));
                            }

                            CMD_RESET_VIEW => {
                                current_view = None;
                            }

                            CMD_RENDER_TO => {
                                // (CMD_RENDER_TO, target_id) — 0 = back to the
                                // screen. Every destination starts in its own
                                // screen space, so the view resets.
                                let target_id = tuple.get_item(1)?.extract::<u32>()?;
                                current_view = None;
                                if target_id == 0 {
                                    bucket_idx = 0;
                                    view_size = (screen_w, screen_h);
                                } else {
                                    let size = crate::engine::with_engine(|engine| {
                                        engine.targets.get(target_id).map(|t| t.size)
                                    })?
                                    .ok_or_else(|| {
                                        pyo3::exceptions::PyValueError::new_err(format!(
                                            "Invalid render target ID: {}",
                                            target_id
                                        ))
                                    })?;
                                    view_size = (size.0 as f32, size.1 as f32);
                                    bucket_idx = buckets
                                        .iter()
                                        .position(|b| b.0 == target_id)
                                        .unwrap_or_else(|| {
                                            buckets.push((target_id, Vec::new(), None));
                                            buckets.len() - 1
                                        });
                                }
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

    // Perform rendering with engine's GPU resources
    crate::engine::with_engine(|engine| {
        // Expand light draws into soft-circle quads for the additive lightmap
        // pass (they do not participate in the scene's z/call ordering)
        let time = engine.start_time.elapsed().as_secs_f32();
        let mut light_cmds: Vec<SpriteDrawCommand> = Vec::new();
        for (light_id, x, y, view) in light_draws {
            if let Some(light) = engine.lights.get(light_id) {
                // Calculate effective intensity with flicker
                let mut intensity = light.intensity;
                if light.flicker_amount > 0.0 {
                    let flicker_phase = time * light.flicker_speed * 10.0;
                    let flicker = (flicker_phase.sin() * 0.5 + 0.5) * light.flicker_amount;
                    intensity *= 1.0 - flicker;
                }

                let diameter = light.radius * 2.0;
                let color = [
                    light.color[0] as f32 / 255.0,
                    light.color[1] as f32 / 255.0,
                    light.color[2] as f32 / 255.0,
                    intensity,
                ];

                light_cmds.push(SpriteDrawCommand {
                    sprite_id: circle_soft_sprite,
                    x,
                    y,
                    rot: 0.0,
                    scale: diameter / 1024.0, // CircleSoft is 1024px
                    scale_y: diameter / 1024.0,
                    alpha: 1.0,
                    flip_x: false,
                    flip_y: false,
                    z: 0,
                    seq: 0,
                    color_override: Some(color),
                    size_override: None,
                    view,
                    blend: BLEND_ALPHA, // unused: lights have their own pipeline
                });
            }
        }

        // Expand particle draws into sprite commands (all particles of a
        // system share its z and seq; they stay in generation order) —
        // appended to the bucket the particles_render() call was issued in
        for (ps_id, z, seq, view, bucket) in particle_draws {
            if let Some(system) = engine.particle_systems.get(ps_id) {
                let blend = system.config.blend;
                if let Some(sprite_id) = system.config.sprite_id {
                    // Render as sprites (textured particles) with particle color
                    let base_size = engine
                        .sprites
                        .get(sprite_id)
                        .map(|s| s.region.2 as f32)
                        .unwrap_or(32.0);
                    for draw_cmd in system.generate_draw_commands(base_size) {
                        buckets[bucket].1.push(DrawItem::Sprite(SpriteDrawCommand {
                            sprite_id: draw_cmd.1,
                            x: draw_cmd.2,
                            y: draw_cmd.3,
                            rot: draw_cmd.4,
                            scale: draw_cmd.5,
                            scale_y: draw_cmd.5,
                            alpha: 1.0, // Alpha is in color_override
                            flip_x: draw_cmd.6,
                            flip_y: draw_cmd.7,
                            z,
                            seq,
                            color_override: Some([
                                draw_cmd.8,
                                draw_cmd.9,
                                draw_cmd.10,
                                draw_cmd.11,
                            ]),
                            size_override: None,
                            view,
                            blend,
                        }));
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
                        buckets[bucket].1.push(DrawItem::Sprite(SpriteDrawCommand {
                            sprite_id: white_pixel_sprite,
                            x: vert_cmd.1,
                            y: vert_cmd.2,
                            rot: 0.0,
                            scale: 1.0,
                            scale_y: 1.0,
                            alpha: 1.0,
                            flip_x: false,
                            flip_y: false,
                            z,
                            seq,
                            color_override: Some(color),
                            size_override: Some((vert_cmd.3, vert_cmd.4)),
                            view,
                            blend,
                        }));
                    }
                }
            }
        }

        let items_count: usize = buckets.iter().map(|b| b.1.len()).sum();

        // Rasterize any missing glyphs BEFORE vertex generation: growing the
        // atlas changes its size and would invalidate UVs already computed
        // against the old size earlier in the same frame.
        if let (Some(device), Some(queue)) = (engine.device.as_ref(), engine.queue.as_ref()) {
            for (_, items, _) in &buckets {
                for item in items {
                    if let DrawItem::Text(cmd) = item {
                        if let Some(font) = engine.fonts.get_mut(cmd.font_id) {
                            font.ensure_glyphs(
                                &cmd.text,
                                device,
                                queue,
                                engine.sprite_texture_bind_group_layout.as_ref(),
                                &mut engine.textures,
                            );
                        }
                    }
                }
            }
        }

        // Lighting is active when there is a light to draw or a non-white
        // ambient; otherwise both extra passes are skipped entirely
        let ambient = engine.lighting.ambient;
        let lighting_active =
            !light_cmds.is_empty() || ambient[0] < 1.0 || ambient[1] < 1.0 || ambient[2] < 1.0;

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

        // (Re)create the lightmap on first use or after a resize
        if lighting_active {
            let (sw, sh) = (surface_config.width, surface_config.height);
            if engine.lightmap_texture.is_none() || engine.lightmap_size != (sw, sh) {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Lightmap"),
                    size: wgpu::Extent3d {
                        width: sw,
                        height: sh,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: LIGHTMAP_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let lightmap_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                if let (Some(layout), Some(sampler)) = (
                    engine.multiply_bind_group_layout.as_ref(),
                    engine.lightmap_sampler.as_ref(),
                ) {
                    engine.multiply_bind_group =
                        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Lightmap Multiply Bind Group"),
                            layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&lightmap_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(sampler),
                                },
                            ],
                        }));
                }
                engine.lightmap_texture = Some(texture);
                engine.lightmap_view = Some(lightmap_view);
                engine.lightmap_size = (sw, sh);
            }
        }

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

        let t_sprite_start = Instant::now();

        // ---- Geometry: every bucket and the light quads share one vertex
        // buffer; each bucket gets its own batch list (drawn in its own pass)
        let mut all_vertices: Vec<SpriteVertex> =
            Vec::with_capacity((items_count + light_cmds.len()) * 6);
        let mut bucket_batches: Vec<Vec<(u32, u32, u32, u8)>> = Vec::with_capacity(buckets.len());

        for (target_id, items, _) in &mut buckets {
            // (texture_id, start, count, blend)
            let mut batches: Vec<(u32, u32, u32, u8)> = Vec::new();

            // A target's own texture cannot be sampled while being rendered to
            let target_texture = if *target_id != 0 {
                engine.targets.get(*target_id).map(|t| t.texture_id)
            } else {
                None
            };

            // One unified sort per destination: back-to-front by z, call order
            // within a z. Batching works for consecutive items with the same
            // texture and blend.
            items.sort_by_key(|item| item.order_key());

            let mut current_key: Option<(u32, u8)> = None;
            let mut batch_start = all_vertices.len() as u32;

            for item in items.iter() {
                match item {
                    DrawItem::Sprite(cmd) => {
                        let sprite = engine.sprites.get(cmd.sprite_id).ok_or_else(|| {
                            pyo3::exceptions::PyRuntimeError::new_err(format!(
                                "Invalid sprite ID: {}",
                                cmd.sprite_id
                            ))
                        })?;

                        if Some(sprite.texture_id) == target_texture {
                            return Err(pyo3::exceptions::PyValueError::new_err(
                                "Cannot draw a render target into itself",
                            ));
                        }

                        let texture_size = engine
                            .textures
                            .get(sprite.texture_id)
                            .map(|t| t.size)
                            .ok_or_else(|| {
                                pyo3::exceptions::PyRuntimeError::new_err(format!(
                                    "Invalid texture ID: {}",
                                    sprite.texture_id
                                ))
                            })?;

                        switch_batch(
                            &mut batches,
                            &mut current_key,
                            &mut batch_start,
                            all_vertices.len() as u32,
                            sprite.texture_id,
                            cmd.blend,
                        );

                        let verts = generate_sprite_vertices(sprite, texture_size, cmd);
                        all_vertices.extend_from_slice(&verts);
                    }
                    DrawItem::Text(cmd) => {
                        let font = match engine.fonts.get(cmd.font_id) {
                            Some(f) => f,
                            None => continue,
                        };
                        let atlas_size = match engine.textures.get(font.atlas_texture_id) {
                            Some(t) => t.size,
                            None => continue,
                        };

                        switch_batch(
                            &mut batches,
                            &mut current_key,
                            &mut batch_start,
                            all_vertices.len() as u32,
                            font.atlas_texture_id,
                            BLEND_ALPHA,
                        );

                        generate_text_vertices(font, atlas_size, cmd, &mut all_vertices);
                    }
                }
            }

            // Save the last batch of this bucket
            if let Some((tex_id, blend)) = current_key {
                let count = all_vertices.len() as u32 - batch_start;
                if count > 0 {
                    batches.push((tex_id, batch_start, count, blend));
                }
            }
            bucket_batches.push(batches);
        }

        // Light quads go after the scene vertices; they all use the soft
        // circle texture, so they form a single draw range
        let mut light_range: Option<(u32, u32, u32)> = None; // (texture_id, start, count)
        if lighting_active && !light_cmds.is_empty() {
            let light_start = all_vertices.len() as u32;
            let mut light_texture_id = None;
            for cmd in &light_cmds {
                let sprite = engine.sprites.get(cmd.sprite_id).ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Light sprite missing")
                })?;
                let texture_size = engine
                    .textures
                    .get(sprite.texture_id)
                    .map(|t| t.size)
                    .ok_or_else(|| {
                        pyo3::exceptions::PyRuntimeError::new_err("Light texture missing")
                    })?;
                light_texture_id = Some(sprite.texture_id);
                let verts = generate_sprite_vertices(sprite, texture_size, cmd);
                all_vertices.extend_from_slice(&verts);
            }
            if let Some(tex_id) = light_texture_id {
                light_range = Some((tex_id, light_start, all_vertices.len() as u32 - light_start));
            }
        }

        // Create or reuse the shared vertex buffer
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

            if DEBUG_TIMING && items_count > 100 {
                static FRAME_COUNT: std::sync::atomic::AtomicU64 =
                    std::sync::atomic::AtomicU64::new(0);
                let frame = FRAME_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if frame.is_multiple_of(60) {
                    println!(
                        "TIMING: parse={:.2}ms, vertex_gen={:.2}ms, buf_write={:.2}ms, items={}",
                        t_parse.as_secs_f64() * 1000.0,
                        t_vertex_gen.as_secs_f64() * 1000.0,
                        (t_buffer_write - t_vertex_gen).as_secs_f64() * 1000.0,
                        items_count
                    );
                }
            }
        }

        // Projection bind group (shared by the scene and light passes)
        let projection_bind_group = if total_vertices > 0 {
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

            // Project logical coordinates onto the physical surface: the
            // user draws in the units passed to init(), HiDPI scales up
            let scale = engine.scale_factor as f32;
            let projection = create_projection_matrix(
                surface_config.width as f32 / scale,
                surface_config.height as f32 / scale,
            );
            queue.write_buffer(
                sprite_projection_buffer,
                0,
                bytemuck::cast_slice(projection.as_ref()),
            );

            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Sprite Projection Bind Group"),
                layout: sprite_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sprite_projection_buffer.as_entire_binding(),
                }],
            }))
        } else {
            None
        };

        // ---- Target passes: each render_to() bucket into its texture ----
        // They run before the screen pass, so a target drawn this frame can
        // be used on screen in the same frame
        for (i, (target_id, _, clear)) in buckets.iter().enumerate() {
            if *target_id == 0 {
                continue;
            }
            let target = engine.targets.get(*target_id).ok_or_else(|| {
                pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid render target ID: {}",
                    target_id
                ))
            })?;
            let target_view = &engine
                .textures
                .get(target.texture_id)
                .ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Render target texture missing")
                })?
                .view;

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Target Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // targets default to transparent, unlike the screen
                        load: wgpu::LoadOp::Clear(clear.unwrap_or(wgpu::Color::TRANSPARENT)),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if !bucket_batches[i].is_empty() {
                let sprite_pipeline = engine.sprite_pipeline.as_ref().ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Sprite pipeline not initialized")
                })?;
                let sprite_pipeline_add = engine.sprite_pipeline_add.as_ref().ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Sprite pipeline not initialized")
                })?;
                draw_sprite_batches(
                    &mut pass,
                    &bucket_batches[i],
                    sprite_pipeline,
                    sprite_pipeline_add,
                    &target.projection_bind_group,
                    engine.sprite_vertex_buffer.as_ref().unwrap(),
                    &engine.textures,
                )?;
            }
        }

        // ---- Pass 1: lights -> lightmap (cleared to the ambient color) ----
        if lighting_active {
            let lightmap_view = engine.lightmap_view.as_ref().ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("Lightmap not initialized")
            })?;

            let mut light_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Lightmap Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: lightmap_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: srgb_to_linear(ambient[0]) as f64,
                            g: srgb_to_linear(ambient[1]) as f64,
                            b: srgb_to_linear(ambient[2]) as f64,
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

            if let (Some((tex_id, start, count)), Some(projection_bind_group)) =
                (light_range, projection_bind_group.as_ref())
            {
                let light_pipeline = engine.light_pipeline.as_ref().ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Light pipeline not initialized")
                })?;
                let bind_group = engine
                    .textures
                    .get(tex_id)
                    .and_then(|t| t.bind_group.as_ref())
                    .ok_or_else(|| {
                        pyo3::exceptions::PyRuntimeError::new_err(
                            "Light texture bind group not initialized",
                        )
                    })?;
                let vertex_buffer = engine.sprite_vertex_buffer.as_ref().unwrap();

                light_pass.set_pipeline(light_pipeline);
                light_pass.set_bind_group(0, projection_bind_group, &[]);
                light_pass.set_bind_group(1, bind_group, &[]);
                light_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                light_pass.draw(start..start + count, 0..1);
            }
        }

        // ---- Pass 2: scene -> surface ----
        {
            let clear_color = buckets[0].2.unwrap_or(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            });
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

            if let (false, Some(projection_bind_group)) =
                (bucket_batches[0].is_empty(), projection_bind_group.as_ref())
            {
                let sprite_pipeline = engine.sprite_pipeline.as_ref().ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Sprite pipeline not initialized")
                })?;
                let sprite_pipeline_add = engine.sprite_pipeline_add.as_ref().ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Sprite pipeline not initialized")
                })?;
                draw_sprite_batches(
                    &mut render_pass,
                    &bucket_batches[0],
                    sprite_pipeline,
                    sprite_pipeline_add,
                    projection_bind_group,
                    engine.sprite_vertex_buffer.as_ref().unwrap(),
                    &engine.textures,
                )?;
            }
        }

        // ---- Pass 3: surface *= lightmap ----
        if lighting_active {
            let multiply_pipeline = engine.multiply_pipeline.as_ref().ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("Multiply pipeline not initialized")
            })?;
            let multiply_bind_group = engine.multiply_bind_group.as_ref().ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("Lightmap bind group not initialized")
            })?;

            let mut multiply_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Lightmap Multiply Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            multiply_pass.set_pipeline(multiply_pipeline);
            multiply_pass.set_bind_group(0, multiply_bind_group, &[]);
            multiply_pass.draw(0..3, 0..1);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_sprite() -> crate::sprite::Sprite {
        crate::sprite::Sprite {
            texture_id: 1,
            region: (0, 0, 64, 32),
            origin: (0.0, 0.0),
            color: [255, 255, 255, 255],
        }
    }

    fn test_cmd() -> SpriteDrawCommand {
        SpriteDrawCommand {
            sprite_id: 1,
            x: 10.0,
            y: 20.0,
            rot: 0.0,
            scale: 1.0,
            scale_y: 1.0,
            alpha: 1.0,
            flip_x: false,
            flip_y: false,
            z: 0,
            seq: 0,
            color_override: None,
            size_override: None,
            view: None,
            blend: BLEND_ALPHA,
        }
    }

    fn sprite_item(z: i32, seq: u32) -> DrawItem {
        let mut cmd = test_cmd();
        cmd.z = z;
        cmd.seq = seq;
        DrawItem::Sprite(cmd)
    }

    fn text_item(z: i32, seq: u32) -> DrawItem {
        DrawItem::Text(TextDrawCommand {
            font_id: 1,
            text: "x".to_string(),
            x: 0.0,
            y: 0.0,
            color: [255, 255, 255, 255],
            z,
            seq,
            view: None,
            align: TEXT_ALIGN_LEFT,
        })
    }

    #[test]
    fn draw_items_sort_by_z_then_call_order_across_kinds() {
        // Regression for two ordering bugs: text ignored z entirely, and
        // same-z order was "sprites, then lights/particles, then text"
        // instead of call order.
        let mut items = [
            text_item(1, 0),   // HUD text at z=1, called first
            sprite_item(0, 1), // background sprite
            sprite_item(2, 2), // overlay sprite above the text
            sprite_item(0, 3), // same z as background, called later
            text_item(0, 4),   // same z, called last
        ];
        items.sort_by_key(|item| item.order_key());

        let keys: Vec<(i32, u32)> = items.iter().map(|i| i.order_key()).collect();
        assert_eq!(keys, vec![(0, 1), (0, 3), (0, 4), (1, 0), (2, 2)]);
        // The z=2 sprite must come after the z=1 text
        assert!(matches!(items.last().unwrap(), DrawItem::Sprite(_)));
    }

    #[test]
    fn batches_split_on_blend_change() {
        let mut batches = Vec::new();
        let mut key = None;
        let mut start = 0u32;
        switch_batch(&mut batches, &mut key, &mut start, 0, 1, BLEND_ALPHA);
        // same texture, new blend -> the alpha batch is closed
        switch_batch(&mut batches, &mut key, &mut start, 6, 1, BLEND_ADD);
        // same (texture, blend) -> no split
        switch_batch(&mut batches, &mut key, &mut start, 12, 1, BLEND_ADD);
        assert_eq!(batches, vec![(1, 0, 6, BLEND_ALPHA)]);
        assert_eq!(key, Some((1, BLEND_ADD)));
        assert_eq!(start, 6);
    }

    #[test]
    fn rect_fill_ex_corner_math_centers_the_quad() {
        // Mirrors the CMD_RECT_FILL_EX corner placement: after rotation the
        // quad must stay centered on the requested point
        let (w, h, rot) = (40.0f32, 20.0f32, 0.7f32);
        let (cx, cy) = (100.0f32, 50.0f32);
        let (sin, cos) = rot.sin_cos();
        let mut cmd = test_cmd();
        cmd.x = cx - (w * cos - h * sin) / 2.0;
        cmd.y = cy - (w * sin + h * cos) / 2.0;
        cmd.rot = rot;
        cmd.size_override = Some((w, h));

        let verts = generate_sprite_vertices(&test_sprite(), (64, 32), &cmd);
        // center = midpoint of the diagonal (top-left, bottom-right)
        let ccx = (verts[0].position[0] + verts[4].position[0]) / 2.0;
        let ccy = (verts[0].position[1] + verts[4].position[1]) / 2.0;
        assert!((ccx - cx).abs() < 1e-3);
        assert!((ccy - cy).abs() < 1e-3);
    }

    #[test]
    fn scale_y_stretches_height_only() {
        let sprite = test_sprite(); // 64x32 region
        let mut cmd = test_cmd(); // at (10, 20)
        cmd.scale = 2.0;
        cmd.scale_y = 3.0;

        let verts = generate_sprite_vertices(&sprite, (64, 32), &cmd);
        assert_eq!(verts[0].position, [10.0, 20.0]);
        assert_eq!(verts[4].position, [10.0 + 64.0 * 2.0, 20.0 + 32.0 * 3.0]);
    }

    #[test]
    fn make_view_centers_the_camera_point() {
        // The world point given to set_view must land at the screen center
        // for any zoom/rotation
        let v = make_view(300.0, 200.0, 1.7, 0.6, 640.0, 480.0);
        let p = view_apply(&v, 300.0, 200.0);
        assert!((p[0] - 320.0).abs() < 1e-3);
        assert!((p[1] - 240.0).abs() < 1e-3);
    }

    #[test]
    fn view_scales_and_translates_sprite_vertices() {
        // Camera at world origin, zoom 2, 200x100 screen: p -> 2p + (100, 50)
        let sprite = test_sprite();
        let mut cmd = test_cmd(); // x=10, y=20, region 64x32
        cmd.view = Some(make_view(0.0, 0.0, 2.0, 0.0, 200.0, 100.0));

        let verts = generate_sprite_vertices(&sprite, (64, 32), &cmd);
        assert_eq!(verts[0].position, [120.0, 90.0]); // top-left (10, 20)
        assert_eq!(verts[4].position, [248.0, 154.0]); // bottom-right (74, 52)
    }

    #[test]
    fn view_rotation_spins_the_world_around_the_camera() {
        // 90° view rotation, camera at origin, zero-size screen for clarity:
        // world +x must map onto screen +y
        let v = make_view(0.0, 0.0, 1.0, std::f32::consts::FRAC_PI_2, 0.0, 0.0);
        let p = view_apply(&v, 1.0, 0.0);
        assert!(p[0].abs() < 1e-6);
        assert!((p[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn vertex_colors_are_straight_alpha() {
        // Regression for the double-premultiply bug: with alpha=0.5 the RGB
        // components must stay 1.0 — premultiplication happens in the shader.
        let sprite = test_sprite();
        let mut cmd = test_cmd();
        cmd.alpha = 0.5;

        let verts = generate_sprite_vertices(&sprite, (64, 32), &cmd);
        for v in &verts {
            assert_eq!(v.color, [1.0, 1.0, 1.0, 0.5]);
        }
    }

    #[test]
    fn sprite_color_combines_with_command_alpha() {
        let mut sprite = test_sprite();
        sprite.color = [255, 0, 0, 128];
        let mut cmd = test_cmd();
        cmd.alpha = 0.5;

        let verts = generate_sprite_vertices(&sprite, (64, 32), &cmd);
        let a = 128.0 / 255.0 * 0.5;
        for v in &verts {
            assert!((v.color[0] - 1.0).abs() < 1e-6);
            assert_eq!(v.color[1], 0.0);
            assert!((v.color[3] - a).abs() < 1e-6);
        }
    }

    #[test]
    fn vertex_colors_are_linearized() {
        // Regression for the gamma bug: sRGB 128 must become linear ~0.2158,
        // not 0.502; alpha stays linear (it's coverage, not color)
        assert!((srgb_to_linear(128.0 / 255.0) - 0.2158).abs() < 1e-3);
        assert_eq!(srgb_to_linear(0.0), 0.0);
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 1e-6);

        let mut sprite = test_sprite();
        sprite.color = [128, 128, 128, 128];
        let cmd = test_cmd();

        let verts = generate_sprite_vertices(&sprite, (64, 32), &cmd);
        for v in &verts {
            assert!((v.color[0] - 0.2158).abs() < 1e-3);
            assert!((v.color[3] - 128.0 / 255.0).abs() < 1e-6); // alpha untouched
        }
    }

    #[test]
    fn unrotated_quad_positions_and_uvs() {
        let sprite = test_sprite();
        let cmd = test_cmd();

        let verts = generate_sprite_vertices(&sprite, (64, 32), &cmd);
        // Top-left vertex at (x, y), full-texture UVs
        assert_eq!(verts[0].position, [10.0, 20.0]);
        assert_eq!(verts[0].tex_coords, [0.0, 0.0]);
        // Bottom-right vertex at (x + w, y + h)
        assert_eq!(verts[4].position, [74.0, 52.0]);
        assert_eq!(verts[4].tex_coords, [1.0, 1.0]);
    }

    #[test]
    fn flip_x_swaps_us() {
        let sprite = test_sprite();
        let mut cmd = test_cmd();
        cmd.flip_x = true;

        let verts = generate_sprite_vertices(&sprite, (64, 32), &cmd);
        assert_eq!(verts[0].tex_coords, [1.0, 0.0]);
        assert_eq!(verts[4].tex_coords, [0.0, 1.0]);
    }

    #[test]
    fn origin_shifts_quad() {
        let mut sprite = test_sprite();
        sprite.origin = (32.0, 16.0); // center
        let cmd = test_cmd();

        let verts = generate_sprite_vertices(&sprite, (64, 32), &cmd);
        assert_eq!(verts[0].position, [10.0 - 32.0, 20.0 - 16.0]);
    }
}
