use pyo3::prelude::*;

use crate::sprite::SpriteId;

#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub fn collide_rects(
    x1: f32,
    y1: f32,
    w1: f32,
    h1: f32,
    x2: f32,
    y2: f32,
    w2: f32,
    h2: f32,
) -> bool {
    x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2
}

#[pyfunction]
pub fn collide_circles(x1: f32, y1: f32, r1: f32, x2: f32, y2: f32, r2: f32) -> bool {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let dist_sq = dx * dx + dy * dy;
    let r_sum = r1 + r2;
    dist_sq <= r_sum * r_sum
}

#[pyfunction]
pub fn collide_circle_rect(cx: f32, cy: f32, r: f32, rx: f32, ry: f32, rw: f32, rh: f32) -> bool {
    let closest_x = cx.clamp(rx, rx + rw);
    let closest_y = cy.clamp(ry, ry + rh);
    let dx = cx - closest_x;
    let dy = cy - closest_y;
    dx * dx + dy * dy <= r * r
}

#[pyfunction]
pub fn point_in_rect(px: f32, py: f32, rx: f32, ry: f32, rw: f32, rh: f32) -> bool {
    px >= rx && px <= rx + rw && py >= ry && py <= ry + rh
}

#[pyfunction]
pub fn point_in_circle(px: f32, py: f32, cx: f32, cy: f32, r: f32) -> bool {
    let dx = px - cx;
    let dy = py - cy;
    dx * dx + dy * dy <= r * r
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub fn raycast_rect(
    ox: f32,
    oy: f32,
    dx: f32,
    dy: f32,
    rx: f32,
    ry: f32,
    rw: f32,
    rh: f32,
) -> Option<f32> {
    let inv_dx = if dx != 0.0 { 1.0 / dx } else { f32::INFINITY };
    let inv_dy = if dy != 0.0 { 1.0 / dy } else { f32::INFINITY };

    let t1 = (rx - ox) * inv_dx;
    let t2 = (rx + rw - ox) * inv_dx;
    let t3 = (ry - oy) * inv_dy;
    let t4 = (ry + rh - oy) * inv_dy;

    let tmin = t1.min(t2).max(t3.min(t4));
    let tmax = t1.max(t2).min(t3.max(t4));

    if tmax < 0.0 || tmin > tmax {
        None
    } else if tmin < 0.0 {
        Some(tmax)
    } else {
        Some(tmin)
    }
}

#[pyfunction]
pub fn sprite_rect(spr: SpriteId, x: f32, y: f32) -> PyResult<(f32, f32, f32, f32)> {
    crate::engine::with_engine(|engine| {
        let sprite = engine.sprites.get(spr).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid sprite ID: {}", spr))
        })?;

        // Get sprite dimensions from region
        let w = sprite.region.2 as f32;
        let h = sprite.region.3 as f32;

        // Get origin (in pixels)
        let (ox, oy) = sprite.origin;

        // Calculate top-left corner based on origin
        let rx = x - ox;
        let ry = y - oy;

        Ok((rx, ry, w, h))
    })?
}

#[pyfunction]
pub fn collide_sprites(
    spr1: SpriteId,
    x1: f32,
    y1: f32,
    spr2: SpriteId,
    x2: f32,
    y2: f32,
) -> PyResult<bool> {
    // Get the bounding rectangles for both sprites
    let (rx1, ry1, rw1, rh1) = sprite_rect(spr1, x1, y1)?;
    let (rx2, ry2, rw2, rh2) = sprite_rect(spr2, x2, y2)?;

    // Use the existing AABB collision function
    Ok(collide_rects(rx1, ry1, rw1, rh1, rx2, ry2, rw2, rh2))
}

#[pyfunction]
pub fn point_in_sprite(px: f32, py: f32, spr: SpriteId, sx: f32, sy: f32) -> PyResult<bool> {
    // Get the bounding rectangle for the sprite
    let (rx, ry, rw, rh) = sprite_rect(spr, sx, sy)?;

    // Use the existing point-in-rect collision function
    Ok(point_in_rect(px, py, rx, ry, rw, rh))
}
