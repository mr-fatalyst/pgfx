use pyo3::prelude::*;

pub type LightId = u32;

/// Point light structure
#[derive(Debug, Clone)]
pub struct Light {
    pub radius: f32,
    pub color: [u8; 4],       // RGBA color (0-255)
    pub intensity: f32,        // Light intensity multiplier (default 1.0)
    pub flicker_amount: f32,   // Flicker amount (0.0 = no flicker)
    pub flicker_speed: f32,    // Flicker speed
}

impl Light {
    pub fn new(radius: f32, color: [u8; 4]) -> Self {
        Self {
            radius,
            color,
            intensity: 1.0,
            flicker_amount: 0.0,
            flicker_speed: 1.0,
        }
    }
}

/// Global lighting state
pub struct LightingState {
    pub ambient: [f32; 4],  // Ambient light color (0.0-1.0)
}

impl Default for LightingState {
    fn default() -> Self {
        Self {
            ambient: [1.0, 1.0, 1.0, 1.0],  // Full white by default (no darkening)
        }
    }
}

impl LightingState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Calculate light contribution at a given point
/// Returns a color multiplier [r, g, b, a] in 0.0-1.0 range
pub fn calculate_light_contribution(
    x: f32,
    y: f32,
    lights: &[(f32, f32, &Light)],  // (x, y, light) tuples
    ambient: [f32; 4],
    time: f32,  // For flicker animation
) -> [f32; 4] {
    // Start with ambient light
    let mut total_r = ambient[0];
    let mut total_g = ambient[1];
    let mut total_b = ambient[2];

    // Add contribution from each light
    for (lx, ly, light) in lights {
        let dx = x - lx;
        let dy = y - ly;
        let dist_sq = dx * dx + dy * dy;
        let radius = light.radius;

        if dist_sq < radius * radius {
            // Calculate attenuation (inverse square law with smoothing)
            let dist = dist_sq.sqrt();
            let attenuation = 1.0 - (dist / radius).min(1.0);
            let attenuation = attenuation * attenuation; // Quadratic falloff for smoother gradient

            // Apply flicker if enabled
            let mut intensity = light.intensity;
            if light.flicker_amount > 0.0 {
                let flicker_phase = time * light.flicker_speed * 10.0;
                let flicker = (flicker_phase.sin() * 0.5 + 0.5) * light.flicker_amount;
                intensity *= 1.0 - flicker;
            }

            // Convert light color to 0.0-1.0 range
            let light_r = light.color[0] as f32 / 255.0;
            let light_g = light.color[1] as f32 / 255.0;
            let light_b = light.color[2] as f32 / 255.0;

            // Add light contribution
            total_r += light_r * attenuation * intensity;
            total_g += light_g * attenuation * intensity;
            total_b += light_b * attenuation * intensity;
        }
    }

    // Clamp to valid range
    [
        total_r.min(1.0),
        total_g.min(1.0),
        total_b.min(1.0),
        1.0,  // Alpha stays at 1.0
    ]
}

#[pyfunction]
pub fn set_ambient(r: u8, g: u8, b: u8, a: u8) -> PyResult<()> {
    crate::engine::with_engine(|engine| {
        engine.lighting.ambient = [
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ];
    })
}

#[pyfunction]
pub fn light_create(radius: f32, r: u8, g: u8, b: u8, a: u8) -> PyResult<LightId> {
    crate::engine::with_engine(|engine| {
        let light = Light::new(radius, [r, g, b, a]);
        engine.lights.insert(light)
    })
}

#[pyfunction]
pub fn light_set_color(light: LightId, r: u8, g: u8, b: u8, a: u8) -> PyResult<()> {
    crate::engine::with_engine(|engine| {
        if let Some(l) = engine.lights.get_mut(light) {
            l.color = [r, g, b, a];
            Ok(())
        } else {
            Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid light ID: {}",
                light
            )))
        }
    })?
}

#[pyfunction]
pub fn light_set_intensity(light: LightId, intensity: f32) -> PyResult<()> {
    crate::engine::with_engine(|engine| {
        if let Some(l) = engine.lights.get_mut(light) {
            l.intensity = intensity;
            Ok(())
        } else {
            Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid light ID: {}",
                light
            )))
        }
    })?
}

#[pyfunction]
pub fn light_set_flicker(light: LightId, amount: f32, speed: f32) -> PyResult<()> {
    crate::engine::with_engine(|engine| {
        if let Some(l) = engine.lights.get_mut(light) {
            l.flicker_amount = amount;
            l.flicker_speed = speed;
            Ok(())
        } else {
            Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid light ID: {}",
                light
            )))
        }
    })?
}

#[pyfunction]
pub fn light_free(light: LightId) -> PyResult<()> {
    crate::engine::with_engine(|engine| {
        if engine.lights.remove(light).is_some() {
            Ok(())
        } else {
            Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid light ID: {}",
                light
            )))
        }
    })?
}
