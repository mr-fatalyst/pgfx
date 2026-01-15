use crate::engine::with_engine;
use crate::texture::TextureId;
use pyo3::prelude::*;

pub type SpriteId = u32;

/// Sprite resource - a region of a texture with rendering properties
#[derive(Clone)]
pub struct Sprite {
    pub texture_id: TextureId,
    pub region: (u32, u32, u32, u32), // x, y, w, h in texture
    pub origin: (f32, f32),           // pivot point in pixels, default (0, 0) = top-left
    pub color: [u8; 4],               // tint RGBA, default white
}

impl Sprite {
    /// Create a new sprite with default values
    pub fn new(texture_id: TextureId, region: (u32, u32, u32, u32)) -> Self {
        Self {
            texture_id,
            region,
            origin: (0.0, 0.0),          // Top-left by default (like HGE)
            color: [255, 255, 255, 255], // White by default
        }
    }
}

#[pyfunction]
pub fn sprite_load(path: &str) -> PyResult<SpriteId> {
    // Load texture
    let texture_id = crate::texture::texture_load(path)?;

    // Get texture size
    let (w, h) = crate::texture::texture_size(texture_id)?;

    // Create sprite covering the entire texture
    with_engine(|engine| {
        let sprite = Sprite::new(texture_id, (0, 0, w, h));
        let id = engine.sprites.insert(sprite);
        Ok(id)
    })?
}

#[pyfunction]
pub fn sprite_create(tex: TextureId, x: u32, y: u32, w: u32, h: u32) -> PyResult<SpriteId> {
    with_engine(|engine| {
        // Verify texture exists
        if engine.textures.get(tex).is_none() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid texture ID: {}",
                tex
            )));
        }

        let sprite = Sprite::new(tex, (x, y, w, h));
        let id = engine.sprites.insert(sprite);
        Ok(id)
    })?
}

#[pyfunction]
pub fn sprite_sheet(path: &str, cols: u32, rows: u32) -> PyResult<Vec<SpriteId>> {
    if cols == 0 || rows == 0 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Cols and rows must be greater than 0",
        ));
    }

    // Load texture
    let texture_id = crate::texture::texture_load(path)?;

    // Get texture size
    let (tex_w, tex_h) = crate::texture::texture_size(texture_id)?;

    // Calculate frame size
    let frame_w = tex_w / cols;
    let frame_h = tex_h / rows;

    // Create sprites for each frame
    let mut sprite_ids = Vec::new();

    with_engine(|engine| {
        for row in 0..rows {
            for col in 0..cols {
                let x = col * frame_w;
                let y = row * frame_h;
                let sprite = Sprite::new(texture_id, (x, y, frame_w, frame_h));
                let id = engine.sprites.insert(sprite);
                sprite_ids.push(id);
            }
        }
        Ok(sprite_ids)
    })?
}

#[pyfunction]
pub fn sprite_set_origin(spr: SpriteId, ox: f32, oy: f32) -> PyResult<()> {
    with_engine(|engine| {
        let sprite = engine.sprites.get_mut(spr).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid sprite ID: {}", spr))
        })?;
        sprite.origin = (ox, oy);
        Ok(())
    })?
}

#[pyfunction]
pub fn sprite_set_color(spr: SpriteId, r: u8, g: u8, b: u8, a: u8) -> PyResult<()> {
    with_engine(|engine| {
        let sprite = engine.sprites.get_mut(spr).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid sprite ID: {}", spr))
        })?;
        sprite.color = [r, g, b, a];
        Ok(())
    })?
}

#[pyfunction]
pub fn sprite_free(spr: SpriteId) -> PyResult<()> {
    with_engine(|engine| {
        engine.sprites.remove(spr).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid sprite ID: {}", spr))
        })?;
        Ok(())
    })?
}
