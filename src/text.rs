use crate::engine::with_engine;
use crate::texture::TextureId;
use pyo3::prelude::*;
use std::collections::HashMap;

pub type FontId = u32;

/// Information about a single glyph in the font atlas
#[derive(Clone, Debug)]
pub struct GlyphInfo {
    pub uv: (f32, f32, f32, f32), // u0, v0, u1, v1 in atlas
    pub size: (f32, f32),         // width, height in pixels
    pub offset: (f32, f32),       // x, y offset from baseline
    pub advance: f32,             // horizontal advance
}

/// Font resource - contains glyph atlas texture and glyph info
pub struct Font {
    pub atlas_texture_id: TextureId,
    pub glyphs: HashMap<char, GlyphInfo>,
    pub smooth: bool, // false = pixel-perfect (round coordinates)
}

/// Create a font atlas from ASCII characters (32-126)
/// Returns RGBA texture data, atlas dimensions, and glyph info map
fn create_font_atlas(
    font: &fontdue::Font,
    size: f32,
) -> (Vec<u8>, u32, u32, HashMap<char, GlyphInfo>) {
    // ASCII printable characters
    let chars: Vec<char> = (32u8..=126u8).map(|c| c as char).collect();

    // Rasterize all glyphs and collect metrics
    let mut rasterized: Vec<(char, fontdue::Metrics, Vec<u8>)> = Vec::new();
    let mut max_glyph_height = 0u32;

    for &c in &chars {
        let (metrics, bitmap) = font.rasterize(c, size);
        max_glyph_height = max_glyph_height.max(metrics.height as u32);
        rasterized.push((c, metrics, bitmap));
    }

    // Simple horizontal layout with padding
    let padding = 2u32;
    let mut current_x = padding;

    // Calculate required atlas width
    for (_, metrics, _) in &rasterized {
        current_x += metrics.width as u32 + padding;
    }
    let mut atlas_width = current_x;

    // Atlas height is max glyph height + padding
    let mut atlas_height = max_glyph_height + padding * 2;

    // Round up to next power of 2 for better GPU compatibility
    atlas_width = atlas_width.next_power_of_two();
    atlas_height = atlas_height.next_power_of_two();

    // Create RGBA atlas texture (initialize with transparent black)
    let mut atlas_data = vec![0u8; (atlas_width * atlas_height * 4) as usize];

    // Pack glyphs into atlas and build glyph info map
    let mut glyphs = HashMap::new();
    current_x = padding;
    let current_y = padding;

    for (c, metrics, bitmap) in rasterized {
        let glyph_width = metrics.width as u32;
        let glyph_height = metrics.height as u32;

        // Copy glyph bitmap into atlas (convert grayscale to RGBA)
        for y in 0..glyph_height {
            for x in 0..glyph_width {
                let atlas_x = current_x + x;
                let atlas_y = current_y + y;
                let atlas_idx = ((atlas_y * atlas_width + atlas_x) * 4) as usize;
                let bitmap_idx = (y * glyph_width + x) as usize;

                if bitmap_idx < bitmap.len() {
                    let alpha = bitmap[bitmap_idx];
                    // White color with varying alpha
                    atlas_data[atlas_idx] = 255; // R
                    atlas_data[atlas_idx + 1] = 255; // G
                    atlas_data[atlas_idx + 2] = 255; // B
                    atlas_data[atlas_idx + 3] = alpha; // A
                }
            }
        }

        // Calculate UV coordinates (normalized to 0-1)
        let u0 = current_x as f32 / atlas_width as f32;
        let v0 = current_y as f32 / atlas_height as f32;
        let u1 = (current_x + glyph_width) as f32 / atlas_width as f32;
        let v1 = (current_y + glyph_height) as f32 / atlas_height as f32;

        // Store glyph info
        // For y offset: we need to align to baseline
        // ymin is the distance from baseline to bottom of glyph (negative means below baseline)
        // To position correctly from top-left, we calculate: size - height - ymin
        let y_offset = size - glyph_height as f32 - metrics.ymin as f32;

        glyphs.insert(
            c,
            GlyphInfo {
                uv: (u0, v0, u1, v1),
                size: (glyph_width as f32, glyph_height as f32),
                offset: (metrics.xmin as f32, y_offset),
                advance: metrics.advance_width,
            },
        );

        // Move to next position
        current_x += glyph_width + padding;
    }

    (atlas_data, atlas_width, atlas_height, glyphs)
}

#[pyfunction]
#[pyo3(signature = (path, size, smooth=true))]
pub fn font_load(path: &str, size: u32, smooth: bool) -> PyResult<FontId> {
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

        // Load TTF font file
        let font_data = std::fs::read(path).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to read font file: {}", e))
        })?;

        // Parse font with fontdue
        let fontdue_font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
            .map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Failed to parse font: {}", e))
        })?;

        // Create glyph atlas
        let size_f32 = size as f32;
        let (atlas_data, atlas_width, atlas_height, glyphs) =
            create_font_atlas(&fontdue_font, size_f32);

        // Create wgpu texture for atlas
        let texture_size = wgpu::Extent3d {
            width: atlas_width,
            height: atlas_height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Font Atlas: {}", path)),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Write atlas data to texture
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * atlas_width),
                rows_per_image: Some(atlas_height),
            },
            texture_size,
        );

        // Create texture view
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create sampler - use Nearest for pixel-perfect text, Linear for smooth
        let filter_mode = if smooth {
            wgpu::FilterMode::Linear
        } else {
            wgpu::FilterMode::Nearest
        };
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group for font atlas texture
        let bind_group = engine
            .sprite_texture_bind_group_layout
            .as_ref()
            .map(|layout| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Font Atlas Bind Group"),
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

        // Create Texture struct for atlas
        let atlas_texture = crate::texture::Texture {
            texture,
            view,
            sampler,
            size: (atlas_width, atlas_height),
            bind_group,
        };

        // Add texture to pool
        let atlas_texture_id = engine.textures.insert(atlas_texture);

        // Create Font struct
        let font = Font {
            atlas_texture_id,
            glyphs,
            smooth,
        };

        // Add to fonts pool
        let font_id = engine.fonts.insert(font);
        Ok(font_id)
    })?
}

#[pyfunction]
pub fn font_free(font: FontId) -> PyResult<()> {
    with_engine(|engine| {
        // Get font and remove its atlas texture
        if let Some(font) = engine.fonts.remove(font) {
            // Also free the atlas texture
            engine.textures.remove(font.atlas_texture_id);
            Ok(())
        } else {
            Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid font ID: {}",
                font
            )))
        }
    })?
}
