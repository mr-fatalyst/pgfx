use crate::engine::with_engine;
use pyo3::prelude::*;

pub type TextureId = u32;

/// Primitive types for procedural texture generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Circle,
    CircleSoft,
    WhitePixel,
}

/// Texture resource stored on GPU
#[allow(dead_code)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: (u32, u32),
    pub bind_group: Option<wgpu::BindGroup>, // Cached bind group for rendering
}

/// Generate RGBA data for a solid circle
pub fn generate_circle_texture(size: u32) -> Vec<u8> {
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0;
    let radius = center - 1.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center + 0.5;
            let dy = y as f32 - center + 0.5;
            let dist = (dx * dx + dy * dy).sqrt();

            let idx = ((y * size + x) * 4) as usize;
            if dist <= radius {
                // White opaque
                data[idx] = 255; // R
                data[idx + 1] = 255; // G
                data[idx + 2] = 255; // B
                data[idx + 3] = 255; // A
            }
            // else: transparent (already 0)
        }
    }
    data
}

/// Generate RGBA data for a soft circle (gradient to transparent edges)
pub fn generate_circle_soft_texture(size: u32) -> Vec<u8> {
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0;
    let radius = center - 1.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center + 0.5;
            let dy = y as f32 - center + 0.5;
            let dist = (dx * dx + dy * dy).sqrt();

            let idx = ((y * size + x) * 4) as usize;
            if dist <= radius {
                // Soft falloff: alpha decreases from center to edge
                let alpha = ((1.0 - dist / radius) * 255.0) as u8;
                data[idx] = 255; // R
                data[idx + 1] = 255; // G
                data[idx + 2] = 255; // B
                data[idx + 3] = alpha; // A
            }
        }
    }
    data
}

/// Generate RGBA data for a 1x1 white pixel
pub fn generate_white_pixel_texture() -> Vec<u8> {
    vec![255u8; 4] // Single white opaque pixel
}

/// Create a texture from RGBA data (internal helper)
pub(crate) fn create_texture_from_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: Option<&wgpu::BindGroupLayout>,
    data: &[u8],
    width: u32,
    height: u32,
    label: &str,
) -> Texture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    // Use Rgba8Unorm for procedural textures (no sRGB gamma)
    // This ensures alpha channel works correctly
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group = layout.map(|l| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Bind Group", label)),
            layout: l,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
    });

    Texture {
        texture,
        view,
        sampler,
        size: (width, height),
        bind_group,
    }
}

#[pyfunction]
pub fn texture_load(path: &str) -> PyResult<TextureId> {
    with_engine(|engine| {
        // Get device and queue
        let device = engine
            .device
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("GPU not initialized"))?;
        let queue = engine
            .queue
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("GPU not initialized"))?;

        // Load image
        let img = image::open(path)
            .map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!("Failed to load image: {}", e))
            })?
            .to_rgba8();
        let dimensions = img.dimensions();

        // Create wgpu texture
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(path),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Write texture data
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        // Create texture view
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group for this texture (cached for rendering)
        let bind_group = engine
            .sprite_texture_bind_group_layout
            .as_ref()
            .map(|layout| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Texture Bind Group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                })
            });

        // Create Texture struct
        let tex = Texture {
            texture,
            view,
            sampler,
            size: dimensions,
            bind_group,
        };

        // Add to resource pool
        let id = engine.textures.insert(tex);
        Ok(id)
    })?
}

#[pyfunction]
pub fn texture_free(tex: TextureId) -> PyResult<()> {
    with_engine(|engine| {
        engine.textures.remove(tex).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid texture ID: {}", tex))
        })?;
        Ok(())
    })?
}

#[pyfunction]
pub fn texture_size(tex: TextureId) -> PyResult<(u32, u32)> {
    with_engine(|engine| {
        let texture = engine.textures.get(tex).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid texture ID: {}", tex))
        })?;
        Ok(texture.size)
    })?
}
