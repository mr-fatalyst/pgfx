use pyo3::prelude::*;

pub type LightId = u32;

/// Point light structure
#[derive(Debug, Clone)]
pub struct Light {
    pub radius: f32,
    pub color: [u8; 4],      // RGBA color (0-255)
    pub intensity: f32,      // Light intensity multiplier (default 1.0)
    pub flicker_amount: f32, // Flicker amount (0.0 = no flicker)
    pub flicker_speed: f32,  // Flicker speed
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
    pub ambient: [f32; 4], // Ambient light color (0.0-1.0)
}

impl Default for LightingState {
    fn default() -> Self {
        Self {
            ambient: [1.0, 1.0, 1.0, 1.0], // Full white by default (no darkening)
        }
    }
}

impl LightingState {
    pub fn new() -> Self {
        Self::default()
    }
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
