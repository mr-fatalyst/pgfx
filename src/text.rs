use crate::engine::with_engine;
use crate::resources::ResourcePool;
use crate::texture::{Texture, TextureId};
use pyo3::prelude::*;
use std::collections::HashMap;

pub type FontId = u32;

const ATLAS_PADDING: u32 = 2;
// Default wgpu limit for a texture dimension
const MAX_ATLAS_DIM: u32 = 8192;

/// Information about a single glyph in the font atlas
#[derive(Clone, Debug)]
pub struct GlyphInfo {
    pub rect: (u32, u32, u32, u32), // x, y, w, h in atlas pixels (w=h=0: advance only)
    pub offset: (f32, f32),         // offset from the line's top-left corner
    pub advance: f32,               // horizontal advance
}

/// Shelf packer for the glyph atlas: fills rows left to right, top to bottom.
/// Allocations only ever append, so existing glyph rects stay valid when the
/// atlas grows.
struct AtlasPacker {
    width: u32,
    height: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
}

impl AtlasPacker {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            cursor_x: ATLAS_PADDING,
            cursor_y: ATLAS_PADDING,
            row_height: 0,
        }
    }

    /// Allocate a w×h region; returns its (x, y) or None if the atlas is full
    fn alloc(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        if w + ATLAS_PADDING * 2 > self.width {
            return None;
        }
        if self.cursor_x + w + ATLAS_PADDING > self.width {
            // Start a new row
            self.cursor_y += self.row_height + ATLAS_PADDING;
            self.cursor_x = ATLAS_PADDING;
            self.row_height = 0;
        }
        if self.cursor_y + h + ATLAS_PADDING > self.height {
            return None;
        }
        let pos = (self.cursor_x, self.cursor_y);
        self.cursor_x += w + ATLAS_PADDING;
        self.row_height = self.row_height.max(h);
        Some(pos)
    }
}

/// Font resource: a fontdue font plus a lazily filled glyph atlas.
/// Glyphs are rasterized on first use and appended to the atlas; the atlas
/// grows (and its GPU texture is replaced in place) when it fills up.
pub struct Font {
    pub atlas_texture_id: TextureId,
    pub glyphs: HashMap<char, GlyphInfo>,
    pub smooth: bool,     // false = pixel-perfect (round coordinates)
    pub line_height: f32, // vertical advance for '\n'
    font: fontdue::Font,
    size: f32,
    ascent: f32,
    packer: AtlasPacker,
    atlas_data: Vec<u8>, // CPU copy (RGBA) used when the atlas grows
}

impl Font {
    /// Kerning adjustment between two characters, if the font provides one
    pub fn kern(&self, left: char, right: char) -> Option<f32> {
        self.font.horizontal_kern(left, right, self.size)
    }

    /// The glyph actually drawn for `ch`: the char itself, the white square
    /// (tofu) if the font lacks it, or the font's .notdef as a last resort
    fn glyph_source(&self, ch: char) -> char {
        if self.font.lookup_glyph_index(ch) != 0 {
            ch
        } else if self.font.lookup_glyph_index('\u{25A1}') != 0 {
            '\u{25A1}'
        } else {
            ch // index 0 rasterizes the font's .notdef glyph
        }
    }

    /// Horizontal advance of one character: from the glyph cache when
    /// available, otherwise straight from the font metrics (no rasterization)
    fn advance(&self, ch: char) -> f32 {
        if let Some(glyph) = self.glyphs.get(&ch) {
            return glyph.advance;
        }
        self.font
            .metrics(self.glyph_source(ch), self.size)
            .advance_width
    }

    /// Width of a single line in logical pixels (advances + kerning;
    /// control characters are skipped — same rules as rendering)
    pub fn line_width(&self, line: &str) -> f32 {
        let mut width = 0.0;
        let mut prev: Option<char> = None;
        for ch in line.chars() {
            if ch.is_control() {
                continue;
            }
            if let Some(p) = prev {
                if let Some(kern) = self.kern(p, ch) {
                    width += kern;
                }
            }
            prev = Some(ch);
            width += self.advance(ch);
        }
        width
    }

    /// Size of a text block: (widest line, line count * line height)
    pub fn measure(&self, text: &str) -> (f32, f32) {
        let mut width: f32 = 0.0;
        let mut lines = 0u32;
        for line in text.split('\n') {
            lines += 1;
            width = width.max(self.line_width(line));
        }
        (width, lines as f32 * self.line_height)
    }

    /// Rasterize any glyphs of `text` that are not in the atlas yet.
    /// Must be called before generating vertices for the frame: growing the
    /// atlas changes its size and would invalidate already-computed UVs.
    pub fn ensure_glyphs(
        &mut self,
        text: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: Option<&wgpu::BindGroupLayout>,
        textures: &mut ResourcePool<Texture>,
    ) {
        for ch in text.chars() {
            if ch.is_control() || self.glyphs.contains_key(&ch) {
                continue;
            }
            self.rasterize_glyph(ch, device, queue, layout, textures);
        }
    }

    fn rasterize_glyph(
        &mut self,
        ch: char,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: Option<&wgpu::BindGroupLayout>,
        textures: &mut ResourcePool<Texture>,
    ) {
        let source = self.glyph_source(ch);

        let (metrics, bitmap) = self.font.rasterize(source, self.size);
        let (w, h) = (metrics.width as u32, metrics.height as u32);
        // text() anchors at the line's top-left; the baseline sits `ascent`
        // below it, and ymin is the glyph's offset below the baseline
        let offset = (
            metrics.xmin as f32,
            self.ascent - h as f32 - metrics.ymin as f32,
        );
        let advance = metrics.advance_width;

        // Empty glyphs (space) carry only an advance
        if w == 0 || h == 0 {
            self.glyphs.insert(
                ch,
                GlyphInfo {
                    rect: (0, 0, 0, 0),
                    offset,
                    advance,
                },
            );
            return;
        }

        let pos = loop {
            if let Some(pos) = self.packer.alloc(w, h) {
                break pos;
            }
            if !self.grow_atlas(device, queue, layout, textures) {
                eprintln!(
                    "pgfx: glyph atlas is full ({}x{}), '{}' will not render",
                    self.packer.width, self.packer.height, ch
                );
                self.glyphs.insert(
                    ch,
                    GlyphInfo {
                        rect: (0, 0, 0, 0),
                        offset,
                        advance,
                    },
                );
                return;
            }
        };

        // Write the glyph into the CPU copy (grayscale coverage -> white RGBA)
        let atlas_w = self.packer.width as usize;
        for row in 0..h as usize {
            for col in 0..w as usize {
                let alpha = bitmap[row * w as usize + col];
                let idx = ((pos.1 as usize + row) * atlas_w + pos.0 as usize + col) * 4;
                self.atlas_data[idx] = 255;
                self.atlas_data[idx + 1] = 255;
                self.atlas_data[idx + 2] = 255;
                self.atlas_data[idx + 3] = alpha;
            }
        }

        // Upload only the glyph's region to the GPU atlas
        if let Some(texture) = textures.get(self.atlas_texture_id) {
            let mut region = Vec::with_capacity((w * h * 4) as usize);
            for row in 0..h as usize {
                let start = ((pos.1 as usize + row) * atlas_w + pos.0 as usize) * 4;
                region.extend_from_slice(&self.atlas_data[start..start + w as usize * 4]);
            }
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: pos.0,
                        y: pos.1,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &region,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * w),
                    rows_per_image: Some(h),
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.glyphs.insert(
            ch,
            GlyphInfo {
                rect: (pos.0, pos.1, w, h),
                offset,
                advance,
            },
        );
    }

    /// Double the atlas (height first, then width) and replace the GPU
    /// texture in place (same texture ID). Returns false at the GPU limit.
    fn grow_atlas(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: Option<&wgpu::BindGroupLayout>,
        textures: &mut ResourcePool<Texture>,
    ) -> bool {
        let (old_w, old_h) = (self.packer.width, self.packer.height);
        let (new_w, new_h) = if old_h * 2 <= MAX_ATLAS_DIM {
            (old_w, old_h * 2)
        } else if old_w * 2 <= MAX_ATLAS_DIM {
            (old_w * 2, old_h)
        } else {
            return false;
        };

        let mut new_data = vec![0u8; (new_w * new_h * 4) as usize];
        if new_w == old_w {
            new_data[..self.atlas_data.len()].copy_from_slice(&self.atlas_data);
        } else {
            let (old_stride, new_stride) = (old_w as usize * 4, new_w as usize * 4);
            for row in 0..old_h as usize {
                new_data[row * new_stride..row * new_stride + old_stride]
                    .copy_from_slice(&self.atlas_data[row * old_stride..(row + 1) * old_stride]);
            }
        }
        self.atlas_data = new_data;
        self.packer.width = new_w;
        self.packer.height = new_h;

        let new_texture = create_atlas_texture(
            device,
            queue,
            layout,
            &self.atlas_data,
            new_w,
            new_h,
            self.smooth,
        );
        if let Some(slot) = textures.get_mut(self.atlas_texture_id) {
            *slot = new_texture;
        }
        true
    }
}

/// Create the GPU texture for a font atlas (filter mode depends on `smooth`)
fn create_atlas_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: Option<&wgpu::BindGroupLayout>,
    data: &[u8],
    width: u32,
    height: u32,
    smooth: bool,
) -> Texture {
    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Font Atlas"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
        texture_size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Nearest for pixel-perfect text, Linear for smooth
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

    let bind_group = layout.map(|l| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Font Atlas Bind Group"),
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
#[pyo3(signature = (path, size, smooth=true))]
pub fn font_load(path: &str, size: u32, smooth: bool) -> PyResult<FontId> {
    with_engine(|engine| {
        // Load and parse the TTF font
        let font_data = std::fs::read(path).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to read font file: {}", e))
        })?;
        let fontdue_font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
            .map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Failed to parse font: {}", e))
        })?;

        let size_f = size as f32;
        let (ascent, line_height) = fontdue_font
            .horizontal_line_metrics(size_f)
            .map(|m| (m.ascent, m.new_line_size))
            .unwrap_or((size_f, size_f * 1.2));

        // Initial atlas sized for the common case; grows lazily on demand
        let atlas_w = (size * 16).next_power_of_two().clamp(512, 2048);
        let atlas_h = (size * 4).next_power_of_two().clamp(128, 2048);
        let atlas_data = vec![0u8; (atlas_w * atlas_h * 4) as usize];

        let device = engine
            .device
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("GPU not initialized"))?;
        let queue = engine
            .queue
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("GPU not initialized"))?;
        let layout = engine.sprite_texture_bind_group_layout.as_ref();

        let texture =
            create_atlas_texture(device, queue, layout, &atlas_data, atlas_w, atlas_h, smooth);
        let atlas_texture_id = engine.textures.insert(texture);

        let mut font = Font {
            atlas_texture_id,
            glyphs: HashMap::new(),
            smooth,
            line_height,
            font: fontdue_font,
            size: size_f,
            ascent,
            packer: AtlasPacker::new(atlas_w, atlas_h),
            atlas_data,
        };

        // Pre-rasterize ASCII so common text doesn't hitch on first draw
        let ascii: String = (32u8..=126u8).map(|c| c as char).collect();
        font.ensure_glyphs(&ascii, device, queue, layout, &mut engine.textures);

        Ok(engine.fonts.insert(font))
    })?
}

#[pyfunction]
pub fn text_size(font: FontId, text: &str) -> PyResult<(f32, f32)> {
    with_engine(|engine| {
        engine
            .fonts
            .get(font)
            .map(|f| f.measure(text))
            .ok_or_else(|| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid font ID: {}", font))
            })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packer_fills_rows_left_to_right() {
        let mut packer = AtlasPacker::new(64, 64);
        let a = packer.alloc(20, 10).unwrap();
        let b = packer.alloc(20, 12).unwrap();
        assert_eq!(a.1, b.1); // same row
        assert!(b.0 > a.0);
    }

    #[test]
    fn packer_wraps_to_new_row() {
        let mut packer = AtlasPacker::new(64, 64);
        packer.alloc(30, 10).unwrap();
        packer.alloc(20, 12).unwrap();
        // Doesn't fit the remaining row width -> next row, below the tallest glyph
        let c = packer.alloc(30, 10).unwrap();
        assert_eq!(c.0, ATLAS_PADDING);
        assert!(c.1 >= 12 + ATLAS_PADDING);
    }

    #[test]
    fn packer_returns_none_when_full() {
        let mut packer = AtlasPacker::new(16, 16);
        assert!(packer.alloc(10, 10).is_some());
        // Doesn't fit the row remainder and no second row fits either
        assert!(packer.alloc(10, 10).is_none());
        assert!(packer.alloc(100, 4).is_none()); // wider than the atlas
    }

    /// A Font with no atlas/GPU behind it: measuring goes through fontdue
    /// metrics only, like text_size() on glyphs not yet rasterized
    fn test_font() -> Font {
        let data = std::fs::read("examples/assets/font.ttf").expect("test font");
        let fontdue_font =
            fontdue::Font::from_bytes(data, fontdue::FontSettings::default()).unwrap();
        let size = 16.0;
        let (ascent, line_height) = fontdue_font
            .horizontal_line_metrics(size)
            .map(|m| (m.ascent, m.new_line_size))
            .unwrap_or((size, size * 1.2));
        Font {
            atlas_texture_id: 0,
            glyphs: HashMap::new(),
            smooth: true,
            line_height,
            font: fontdue_font,
            size,
            ascent,
            packer: AtlasPacker::new(64, 64),
            atlas_data: vec![0; 64 * 64 * 4],
        }
    }

    #[test]
    fn measure_single_line() {
        let font = test_font();
        let (w1, h) = font.measure("a");
        let (w2, _) = font.measure("aa");
        assert!(w1 > 0.0);
        assert!(w2 > w1 * 1.5 && w2 < w1 * 2.5); // ~ two advances (+kerning)
        assert_eq!(h, font.line_height);
    }

    #[test]
    fn measure_multiline_takes_widest_line() {
        let font = test_font();
        let (w_wide, _) = font.measure("the widest line");
        let (w, h) = font.measure("the widest line\nx");
        assert_eq!(w, w_wide);
        assert_eq!(h, 2.0 * font.line_height);
    }

    #[test]
    fn measure_skips_control_chars() {
        let font = test_font();
        assert_eq!(font.measure("a\rb\t").0, font.measure("ab").0);
    }

    #[test]
    fn measure_prefers_cached_glyph_advance() {
        // After "rasterization" the cached advance must win over metrics
        let mut font = test_font();
        font.glyphs.insert(
            'a',
            GlyphInfo {
                rect: (0, 0, 0, 0),
                offset: (0.0, 0.0),
                advance: 100.0,
            },
        );
        assert_eq!(font.measure("a").0, 100.0);
    }
}
