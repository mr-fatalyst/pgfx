use crate::engine::with_engine;
use crate::sprite::SpriteId;
use pyo3::prelude::*;
use pyo3::BoundObject;
use std::f32::consts::PI;

pub type ParticleSystemId = u32;

/// Individual particle data
#[derive(Clone)]
struct Particle {
    position: [f32; 2],
    velocity: [f32; 2],
    life: f32,     // Current lifetime
    max_life: f32, // Maximum lifetime
    size: f32,
    color: [f32; 4], // RGBA 0-1
}

impl Particle {
    fn new() -> Self {
        Self {
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
            life: 0.0,
            max_life: 1.0,
            size: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    fn is_alive(&self) -> bool {
        self.life > 0.0
    }

    fn update(&mut self, dt: f32, config: &ParticleConfig) {
        if !self.is_alive() {
            return;
        }

        // Update lifetime
        self.life -= dt;

        // Apply gravity
        self.velocity[0] += config.gravity[0] * dt;
        self.velocity[1] += config.gravity[1] * dt;

        // Update position
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;

        // Interpolate color
        let life_ratio = self.life / self.max_life;
        self.color = [
            config.start_color[0] * life_ratio + config.end_color[0] * (1.0 - life_ratio),
            config.start_color[1] * life_ratio + config.end_color[1] * (1.0 - life_ratio),
            config.start_color[2] * life_ratio + config.end_color[2] * (1.0 - life_ratio),
            config.start_color[3] * life_ratio + config.end_color[3] * (1.0 - life_ratio),
        ];

        // Interpolate size
        self.size = config.start_size * life_ratio + config.end_size * (1.0 - life_ratio);
    }
}

/// Particle system configuration
#[derive(Clone)]
pub struct ParticleConfig {
    pub lifetime_min: f32,
    pub lifetime_max: f32,
    pub speed_min: f32,
    pub speed_max: f32,
    pub direction: f32,        // Base direction in radians
    pub spread: f32,           // Spread angle in radians
    pub gravity: [f32; 2],     // Gravity acceleration (x, y)
    pub start_color: [f32; 4], // Start color RGBA 0-1
    pub end_color: [f32; 4],   // End color RGBA 0-1
    pub start_size: f32,
    pub end_size: f32,
    pub emission_rate: f32,     // Particles per second
    pub sprite_id: Option<u32>, // Optional sprite for textured particles
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            lifetime_min: 1.0,
            lifetime_max: 2.0,
            speed_min: 50.0,
            speed_max: 100.0,
            direction: -PI / 2.0,              // Up
            spread: PI / 4.0,                  // 45 degree spread
            gravity: [0.0, 98.0],              // Gravity pointing down
            start_color: [1.0, 1.0, 1.0, 1.0], // White
            end_color: [1.0, 1.0, 1.0, 0.0],   // Transparent white
            start_size: 4.0,
            end_size: 0.0,
            emission_rate: 0.0, // Manual emission by default
            sprite_id: None,
        }
    }
}

/// Particle system
pub(crate) struct ParticleSystem {
    pub(crate) config: ParticleConfig,
    particles: Vec<Particle>,
    max_particles: usize,
    position: [f32; 2], // Emitter position
    is_emitting: bool,
    emission_accumulator: f32,
}

impl ParticleSystem {
    pub fn new(max_particles: usize) -> Self {
        Self {
            config: ParticleConfig::default(),
            particles: Vec::with_capacity(max_particles),
            max_particles,
            position: [0.0, 0.0],
            is_emitting: false,
            emission_accumulator: 0.0,
        }
    }

    pub fn with_sprite(sprite_id: SpriteId, max_particles: usize) -> Self {
        let mut system = Self::new(max_particles);
        system.config.sprite_id = Some(sprite_id);
        system
    }

    pub fn configure(&mut self, config: ParticleConfig) {
        self.config = config;
    }

    pub fn emit(&mut self, x: f32, y: f32, count: u32) {
        self.position = [x, y];

        for _ in 0..count {
            if self.particles.len() >= self.max_particles {
                break;
            }
            self.spawn_particle();
        }
    }

    pub fn fire(&mut self, x: f32, y: f32) {
        self.position = [x, y];
        self.is_emitting = true;
    }

    pub fn stop(&mut self) {
        self.is_emitting = false;
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.position = [x, y];
    }

    fn spawn_particle(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Random lifetime
        let lifetime = rng.gen_range(self.config.lifetime_min..=self.config.lifetime_max);

        // Random speed
        let speed = rng.gen_range(self.config.speed_min..=self.config.speed_max);

        // Random direction within spread
        let angle_offset = rng.gen_range(-self.config.spread / 2.0..=self.config.spread / 2.0);
        let angle = self.config.direction + angle_offset;

        // Calculate velocity
        let velocity = [angle.cos() * speed, angle.sin() * speed];

        let mut particle = Particle::new();
        particle.position = self.position;
        particle.velocity = velocity;
        particle.life = lifetime;
        particle.max_life = lifetime;
        particle.size = self.config.start_size;
        particle.color = self.config.start_color;

        self.particles.push(particle);
    }

    pub fn update(&mut self, dt: f32) {
        // Update existing particles
        for particle in &mut self.particles {
            particle.update(dt, &self.config);
        }

        // Remove dead particles
        self.particles.retain(|p| p.is_alive());

        // Auto-emission
        if self.is_emitting && self.config.emission_rate > 0.0 {
            self.emission_accumulator += dt * self.config.emission_rate;

            while self.emission_accumulator >= 1.0 {
                if self.particles.len() < self.max_particles {
                    self.spawn_particle();
                }
                self.emission_accumulator -= 1.0;
            }
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.particles.is_empty() || self.is_emitting
    }

    pub fn count(&self) -> u32 {
        self.particles.len() as u32
    }

    /// Generate draw commands for rendering particles
    /// Returns (cmd, sprite_id, x, y, rot, scale, flip_x, flip_y, r, g, b, a)
    #[allow(clippy::type_complexity)]
    pub fn generate_draw_commands(
        &self,
        base_sprite_size: f32,
    ) -> Vec<(u8, u32, f32, f32, f32, f32, bool, bool, f32, f32, f32, f32)> {
        let mut commands = Vec::new();

        if let Some(sprite_id) = self.config.sprite_id {
            // Render as sprites with particle color
            // Scale so that particle.size represents actual pixel size
            for particle in &self.particles {
                let scale = particle.size / base_sprite_size;
                commands.push((
                    crate::renderer::CMD_DRAW_EX,
                    sprite_id,
                    particle.position[0],
                    particle.position[1],
                    0.0, // rotation
                    scale,
                    false,             // flip_x
                    false,             // flip_y
                    particle.color[0], // r
                    particle.color[1], // g
                    particle.color[2], // b
                    particle.color[3], // a
                ));
            }
        }

        commands
    }

    /// Generate primitive vertices for rendering particles as colored quads
    #[allow(clippy::type_complexity)]
    pub fn generate_vertices(&self) -> Vec<(u8, f32, f32, f32, f32, u8, u8, u8, u8)> {
        let mut commands = Vec::new();

        for particle in &self.particles {
            let half_size = particle.size * 0.5;
            let x = particle.position[0] - half_size;
            let y = particle.position[1] - half_size;
            let size = particle.size;

            // Convert color from 0-1 to 0-255
            let r = (particle.color[0] * 255.0).clamp(0.0, 255.0) as u8;
            let g = (particle.color[1] * 255.0).clamp(0.0, 255.0) as u8;
            let b = (particle.color[2] * 255.0).clamp(0.0, 255.0) as u8;
            let a = (particle.color[3] * 255.0).clamp(0.0, 255.0) as u8;

            // Add rect_fill command
            commands.push((crate::renderer::CMD_RECT_FILL, x, y, size, size, r, g, b, a));
        }

        commands
    }
}

/// Parse particle configuration from Python kwargs
fn parse_particle_config(
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<ParticleConfig> {
    let mut config = ParticleConfig::default();

    if let Some(dict) = kwargs {
        // Lifetime
        if let Some(val) = dict.get_item("lifetime_min")? {
            config.lifetime_min = val.extract()?;
        }
        if let Some(val) = dict.get_item("lifetime_max")? {
            config.lifetime_max = val.extract()?;
        }
        if let Some(val) = dict.get_item("lifetime")? {
            // Single lifetime value sets both min and max
            let lifetime: f32 = val.extract()?;
            config.lifetime_min = lifetime;
            config.lifetime_max = lifetime;
        }

        // Speed
        if let Some(val) = dict.get_item("speed_min")? {
            config.speed_min = val.extract()?;
        }
        if let Some(val) = dict.get_item("speed_max")? {
            config.speed_max = val.extract()?;
        }
        if let Some(val) = dict.get_item("speed")? {
            // Single speed value sets both min and max
            let speed: f32 = val.extract()?;
            config.speed_min = speed;
            config.speed_max = speed;
        }

        // Direction and spread
        if let Some(val) = dict.get_item("direction")? {
            config.direction = val.extract()?;
        }
        if let Some(val) = dict.get_item("spread")? {
            config.spread = val.extract()?;
        }

        // Gravity
        if let Some(val) = dict.get_item("gravity_x")? {
            config.gravity[0] = val.extract()?;
        }
        if let Some(val) = dict.get_item("gravity_y")? {
            config.gravity[1] = val.extract()?;
        }
        if let Some(val) = dict.get_item("gravity")? {
            // Tuple of (x, y)
            let gravity: (f32, f32) = val.extract()?;
            config.gravity = [gravity.0, gravity.1];
        }

        // Colors
        if let Some(val) = dict.get_item("start_color")? {
            let color: (u8, u8, u8, u8) = val.extract()?;
            config.start_color = [
                color.0 as f32 / 255.0,
                color.1 as f32 / 255.0,
                color.2 as f32 / 255.0,
                color.3 as f32 / 255.0,
            ];
        }
        if let Some(val) = dict.get_item("end_color")? {
            let color: (u8, u8, u8, u8) = val.extract()?;
            config.end_color = [
                color.0 as f32 / 255.0,
                color.1 as f32 / 255.0,
                color.2 as f32 / 255.0,
                color.3 as f32 / 255.0,
            ];
        }
        if let Some(val) = dict.get_item("color")? {
            // Single color sets both start and end
            let color: (u8, u8, u8, u8) = val.extract()?;
            let rgba = [
                color.0 as f32 / 255.0,
                color.1 as f32 / 255.0,
                color.2 as f32 / 255.0,
                color.3 as f32 / 255.0,
            ];
            config.start_color = rgba;
            config.end_color = rgba;
        }

        // Size
        if let Some(val) = dict.get_item("start_size")? {
            config.start_size = val.extract()?;
        }
        if let Some(val) = dict.get_item("end_size")? {
            config.end_size = val.extract()?;
        }
        if let Some(val) = dict.get_item("size")? {
            // Single size value
            let size: f32 = val.extract()?;
            config.start_size = size;
            config.end_size = size;
        }
        if let Some(val) = dict.get_item("scale")? {
            // Legacy parameter name
            let scale: f32 = val.extract()?;
            config.start_size = scale * 4.0;
            config.end_size = scale * 4.0;
        }

        // Emission rate
        if let Some(val) = dict.get_item("emission_rate")? {
            config.emission_rate = val.extract()?;
        }
        if let Some(val) = dict.get_item("emission")? {
            // Legacy parameter name
            config.emission_rate = val.extract()?;
        }
    }

    Ok(config)
}

// Python API functions

#[pyfunction]
pub fn particles_load(path: &str) -> PyResult<ParticleSystemId> {
    // Load particle system from JSON file
    // For now, we'll just return an error - JSON loading can be implemented later
    let _ = path;
    Err(pyo3::exceptions::PyNotImplementedError::new_err(
        "particles_load is not yet implemented. Use particles_create instead.",
    ))
}

#[pyfunction]
#[pyo3(signature = (sprite=None, primitive=None, **kwargs))]
pub fn particles_create(
    sprite: Option<SpriteId>,
    primitive: Option<&str>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<ParticleSystemId> {
    // Parse configuration
    let config = parse_particle_config(kwargs)?;

    // Get max_particles from kwargs or use default
    let max_particles = if let Some(dict) = kwargs {
        if let Some(val) = dict.get_item("max_particles")? {
            val.extract()?
        } else {
            1000
        }
    } else {
        1000
    };

    // Determine sprite to use
    let sprite_id: SpriteId = if let Some(s) = sprite {
        // Explicit sprite provided
        s
    } else if let Some(prim) = primitive {
        // Create primitive sprite
        let prim_type = match prim {
            "circle" => crate::texture::PrimitiveType::Circle,
            "circle_soft" => crate::texture::PrimitiveType::CircleSoft,
            "square" | "pixel" => crate::texture::PrimitiveType::WhitePixel,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown primitive: '{}'. Use 'circle', 'circle_soft', 'square', or 'pixel'",
                    prim
                )))
            }
        };

        with_engine(|engine| {
            engine
                .get_or_create_primitive_sprite(prim_type)
                .ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Failed to create primitive texture")
                })
        })??
    } else {
        // Default to circle_soft
        with_engine(|engine| {
            engine
                .get_or_create_primitive_sprite(crate::texture::PrimitiveType::CircleSoft)
                .ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Failed to create primitive texture")
                })
        })??
    };

    // Create particle system
    with_engine(|engine| {
        let mut system = ParticleSystem::with_sprite(sprite_id, max_particles);
        // Configure but preserve sprite_id
        let mut final_config = config;
        final_config.sprite_id = Some(sprite_id);
        system.configure(final_config);

        let id = engine.particle_systems.insert(system);
        Ok(id)
    })?
}

#[pyfunction]
pub fn particles_free(ps: ParticleSystemId) -> PyResult<()> {
    with_engine(|engine| {
        engine.particle_systems.remove(ps).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid particle system ID: {}", ps))
        })?;
        Ok(())
    })?
}

#[pyfunction]
#[pyo3(signature = (ps, **kwargs))]
pub fn particles_set(
    ps: ParticleSystemId,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<()> {
    let config = parse_particle_config(kwargs)?;

    with_engine(|engine| {
        let system = engine.particle_systems.get_mut(ps).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid particle system ID: {}", ps))
        })?;

        // Preserve sprite_id when updating config
        let mut final_config = config;
        final_config.sprite_id = system.config.sprite_id;
        system.configure(final_config);
        Ok(())
    })?
}

#[pyfunction]
pub fn particles_fire(ps: ParticleSystemId, x: f32, y: f32) -> PyResult<()> {
    with_engine(|engine| {
        let system = engine.particle_systems.get_mut(ps).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid particle system ID: {}", ps))
        })?;

        system.fire(x, y);
        Ok(())
    })?
}

#[pyfunction]
pub fn particles_emit(ps: ParticleSystemId, x: f32, y: f32, count: u32) -> PyResult<()> {
    with_engine(|engine| {
        let system = engine.particle_systems.get_mut(ps).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid particle system ID: {}", ps))
        })?;

        system.emit(x, y, count);
        Ok(())
    })?
}

#[pyfunction]
pub fn particles_stop(ps: ParticleSystemId) -> PyResult<()> {
    with_engine(|engine| {
        let system = engine.particle_systems.get_mut(ps).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid particle system ID: {}", ps))
        })?;

        system.stop();
        Ok(())
    })?
}

#[pyfunction]
pub fn particles_move_to(ps: ParticleSystemId, x: f32, y: f32) -> PyResult<()> {
    with_engine(|engine| {
        let system = engine.particle_systems.get_mut(ps).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid particle system ID: {}", ps))
        })?;

        system.move_to(x, y);
        Ok(())
    })?
}

#[pyfunction]
pub fn particles_update(ps: ParticleSystemId, dt: f32) -> PyResult<()> {
    with_engine(|engine| {
        let system = engine.particle_systems.get_mut(ps).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid particle system ID: {}", ps))
        })?;

        system.update(dt);
        Ok(())
    })?
}

#[pyfunction]
pub fn particles_is_alive(ps: ParticleSystemId) -> PyResult<bool> {
    with_engine(|engine| {
        let system = engine.particle_systems.get(ps).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid particle system ID: {}", ps))
        })?;

        Ok(system.is_alive())
    })?
}

#[pyfunction]
pub fn particles_count(ps: ParticleSystemId) -> PyResult<u32> {
    with_engine(|engine| {
        let system = engine.particle_systems.get(ps).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid particle system ID: {}", ps))
        })?;

        Ok(system.count())
    })?
}

/// Generate render commands for a particle system
/// Returns a list of tuples that can be added to the render batch
#[pyfunction]
pub fn particles_render(py: Python<'_>, ps: ParticleSystemId) -> PyResult<Vec<Py<PyAny>>> {
    with_engine(|engine| {
        let system = engine.particle_systems.get(ps).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid particle system ID: {}", ps))
        })?;

        let mut commands = Vec::new();

        if let Some(sprite_id) = system.config.sprite_id {
            // Render as sprites (textured particles)
            // Get sprite size for correct scaling (particle.size is in pixels)
            let base_size = engine
                .sprites
                .get(sprite_id)
                .map(|s| s.region.2 as f32) // width of sprite region
                .unwrap_or(32.0); // fallback to primitive size

            // Note: this returns Python commands, but internally we use renderer directly
            // Format: (cmd, sprite_id, x, y, rot, scale, flip_x, flip_y, r, g, b, a)
            for draw_cmd in system.generate_draw_commands(base_size) {
                let items: Vec<Py<PyAny>> = vec![
                    draw_cmd.0.into_pyobject(py)?.unbind().into(), // CMD_DRAW_EX
                    draw_cmd.1.into_pyobject(py)?.unbind().into(), // sprite_id
                    draw_cmd.2.into_pyobject(py)?.unbind().into(), // x
                    draw_cmd.3.into_pyobject(py)?.unbind().into(), // y
                    draw_cmd.4.into_pyobject(py)?.unbind().into(), // rot
                    draw_cmd.5.into_pyobject(py)?.unbind().into(), // scale
                    pyo3::types::PyBool::new(py, draw_cmd.6)
                        .into_bound()
                        .unbind()
                        .into(), // flip_x
                    pyo3::types::PyBool::new(py, draw_cmd.7)
                        .into_bound()
                        .unbind()
                        .into(), // flip_y
                    draw_cmd.8.into_pyobject(py)?.unbind().into(), // r
                    draw_cmd.9.into_pyobject(py)?.unbind().into(), // g
                    draw_cmd.10.into_pyobject(py)?.unbind().into(), // b
                    draw_cmd.11.into_pyobject(py)?.unbind().into(), // a
                ];
                let tuple = pyo3::types::PyTuple::new(py, items.as_slice())?;
                commands.push(tuple.unbind().into());
            }
        } else {
            // Render as colored quads (primitive particles)
            for vert_cmd in system.generate_vertices() {
                let items: Vec<Py<PyAny>> = vec![
                    vert_cmd.0.into_pyobject(py)?.unbind().into(), // CMD_RECT_FILL
                    vert_cmd.1.into_pyobject(py)?.unbind().into(), // x
                    vert_cmd.2.into_pyobject(py)?.unbind().into(), // y
                    vert_cmd.3.into_pyobject(py)?.unbind().into(), // w
                    vert_cmd.4.into_pyobject(py)?.unbind().into(), // h
                    vert_cmd.5.into_pyobject(py)?.unbind().into(), // r
                    vert_cmd.6.into_pyobject(py)?.unbind().into(), // g
                    vert_cmd.7.into_pyobject(py)?.unbind().into(), // b
                    vert_cmd.8.into_pyobject(py)?.unbind().into(), // a
                ];
                let tuple = pyo3::types::PyTuple::new(py, items.as_slice())?;
                commands.push(tuple.unbind().into());
            }
        }

        Ok(commands)
    })?
}
