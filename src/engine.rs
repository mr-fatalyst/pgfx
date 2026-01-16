use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::texture::PrimitiveType;

/// Global engine instance
static ENGINE: OnceLock<Mutex<Engine>> = OnceLock::new();

/// Engine configuration passed to init()
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub fullscreen: bool,
    pub resizable: bool,
    pub fps_limit: u32,
}

/// Main engine state
pub struct Engine {
    // Configuration
    pub(crate) config: EngineConfig,

    // Window
    pub(crate) window: Option<Arc<Window>>,

    // GPU
    pub(crate) instance: Option<wgpu::Instance>,
    pub(crate) surface: Option<wgpu::Surface<'static>>,
    pub(crate) device: Option<wgpu::Device>,
    pub(crate) queue: Option<wgpu::Queue>,
    pub(crate) surface_config: Option<wgpu::SurfaceConfiguration>,

    // Sprite rendering pipeline (used for everything - sprites, primitives, text)
    pub(crate) sprite_pipeline: Option<wgpu::RenderPipeline>,
    pub(crate) sprite_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(crate) sprite_texture_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(crate) sprite_projection_buffer: Option<wgpu::Buffer>,
    pub(crate) sprite_vertex_buffer: Option<wgpu::Buffer>,
    pub(crate) sprite_vertex_buffer_capacity: usize, // in vertices

    // Resources
    pub(crate) textures: crate::resources::ResourcePool<crate::texture::Texture>,
    pub(crate) sprites: crate::resources::ResourcePool<crate::sprite::Sprite>,
    pub(crate) fonts: crate::resources::ResourcePool<crate::text::Font>,
    pub(crate) particle_systems: crate::resources::ResourcePool<crate::particles::ParticleSystem>,
    pub(crate) lights: crate::resources::ResourcePool<crate::lighting::Light>,

    // Cached primitive textures (circle, circle_soft, square)
    pub(crate) primitive_textures:
        HashMap<PrimitiveType, (crate::texture::TextureId, crate::sprite::SpriteId)>,

    // Lighting
    pub(crate) lighting: crate::lighting::LightingState,

    // Input
    pub(crate) input: crate::input::InputState,

    // Gamepad
    pub(crate) gamepad: Option<crate::input::GamepadManager>,

    // Timing
    pub(crate) start_time: Instant,
    pub(crate) last_frame: Instant,
    pub(crate) dt: f32,
    pub(crate) fps: u32,

    // Running state
    pub(crate) running: bool,
}

impl Engine {
    fn new(config: EngineConfig) -> Self {
        let now = Instant::now();
        Self {
            config,
            window: None,
            instance: None,
            surface: None,
            device: None,
            queue: None,
            surface_config: None,
            sprite_pipeline: None,
            sprite_bind_group_layout: None,
            sprite_texture_bind_group_layout: None,
            sprite_projection_buffer: None,
            sprite_vertex_buffer: None,
            sprite_vertex_buffer_capacity: 0,
            textures: crate::resources::ResourcePool::new(),
            sprites: crate::resources::ResourcePool::new(),
            fonts: crate::resources::ResourcePool::new(),
            particle_systems: crate::resources::ResourcePool::new(),
            lights: crate::resources::ResourcePool::new(),
            primitive_textures: HashMap::new(),
            lighting: crate::lighting::LightingState::new(),
            input: crate::input::InputState::new(),
            gamepad: None, // Initialized on first use
            start_time: now,
            last_frame: now,
            dt: 0.016, // ~60 fps default
            fps: 60,
            running: true,
        }
    }

    fn set_window(&mut self, window: Window) {
        self.window = Some(Arc::new(window));
    }

    pub(crate) fn set_gpu(
        &mut self,
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface_config: wgpu::SurfaceConfiguration,
    ) {
        self.instance = Some(instance);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface_config = Some(surface_config);
    }

    pub(crate) fn reconfigure_surface(&mut self, width: u32, height: u32) {
        if let (Some(surface), Some(device), Some(config)) =
            (&self.surface, &self.device, &mut self.surface_config)
        {
            config.width = width;
            config.height = height;
            surface.configure(device, config);
        }
    }

    /// Get or create a primitive sprite (cached)
    pub(crate) fn get_or_create_primitive_sprite(
        &mut self,
        primitive: PrimitiveType,
    ) -> Option<crate::sprite::SpriteId> {
        // Return cached if exists
        if let Some((_, sprite_id)) = self.primitive_textures.get(&primitive) {
            return Some(*sprite_id);
        }

        // Need to create - get GPU resources
        let device = self.device.as_ref()?;
        let queue = self.queue.as_ref()?;
        let layout = self.sprite_texture_bind_group_layout.as_ref();

        // Generate texture data
        const CIRCLE_SIZE: u32 = 1024; // Large for quality when scaling down
        let (data, size, label) = match primitive {
            PrimitiveType::Circle => (
                crate::texture::generate_circle_texture(CIRCLE_SIZE),
                CIRCLE_SIZE,
                "Primitive Circle",
            ),
            PrimitiveType::CircleSoft => (
                crate::texture::generate_circle_soft_texture(CIRCLE_SIZE),
                CIRCLE_SIZE,
                "Primitive Circle Soft",
            ),
            PrimitiveType::WhitePixel => (
                crate::texture::generate_white_pixel_texture(),
                1,
                "Primitive White Pixel",
            ),
        };

        // Create texture
        let texture = crate::texture::create_texture_from_rgba(
            device, queue, layout, &data, size, size, label,
        );

        let texture_id = self.textures.insert(texture);

        // Create sprite from texture
        // Origin in pixels: WhitePixel at (0,0), circles at center
        let origin = match primitive {
            PrimitiveType::WhitePixel => (0.0, 0.0),
            PrimitiveType::Circle | PrimitiveType::CircleSoft => {
                (size as f32 / 2.0, size as f32 / 2.0)
            }
        };
        let sprite = crate::sprite::Sprite {
            texture_id,
            region: (0, 0, size, size),
            origin,
            color: [255, 255, 255, 255],
        };
        let sprite_id = self.sprites.insert(sprite);

        // Cache it
        self.primitive_textures
            .insert(primitive, (texture_id, sprite_id));

        Some(sprite_id)
    }
}

/// Helper function to access engine state
pub fn with_engine<F, R>(f: F) -> PyResult<R>
where
    F: FnOnce(&mut Engine) -> R,
{
    let engine = ENGINE.get().ok_or_else(|| {
        pyo3::exceptions::PyRuntimeError::new_err("pgfx not initialized. Call init() first.")
    })?;

    let mut guard = engine.lock().unwrap();
    Ok(f(&mut guard))
}

/// Simple frame rate limiter
struct FrameLimiter {
    target_frame_time: Option<std::time::Duration>,
    frame_start: Instant,
}

impl FrameLimiter {
    fn new(fps_limit: u32) -> Self {
        let target_frame_time = if fps_limit > 0 {
            Some(std::time::Duration::from_secs_f64(1.0 / fps_limit as f64))
        } else {
            None
        };
        Self {
            target_frame_time,
            frame_start: Instant::now(),
        }
    }

    fn frame_start(&mut self) {
        self.frame_start = Instant::now();
    }

    fn frame_end(&self) {
        if let Some(target) = self.target_frame_time {
            let elapsed = self.frame_start.elapsed();
            if elapsed < target {
                std::thread::sleep(target - elapsed);
            }
        }
    }
}

/// Application handler for winit event loop
struct AppHandler {
    update_fn: Py<PyAny>,
    render_fn: Py<PyAny>,
    on_ready_fn: Option<Py<PyAny>>,
    window_created: bool,
    ready_called: bool,
    frame_limiter: FrameLimiter,
}

impl AppHandler {
    fn new(
        update_fn: Py<PyAny>,
        render_fn: Py<PyAny>,
        on_ready_fn: Option<Py<PyAny>>,
        fps_limit: u32,
    ) -> Self {
        Self {
            update_fn,
            render_fn,
            on_ready_fn,
            window_created: false,
            ready_called: false,
            frame_limiter: FrameLimiter::new(fps_limit),
        }
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window on first resume
        if !self.window_created {
            // Get config from engine
            let (width, height, title, resizable, fullscreen) = with_engine(|engine| {
                (
                    engine.config.width,
                    engine.config.height,
                    engine.config.title.clone(),
                    engine.config.resizable,
                    engine.config.fullscreen,
                )
            })
            .expect("Engine not initialized");

            // Create window attributes - start hidden to avoid garbage frame
            let mut window_attrs = Window::default_attributes()
                .with_title(title)
                .with_inner_size(winit::dpi::LogicalSize::new(width, height))
                .with_resizable(resizable)
                .with_visible(false);

            if fullscreen {
                window_attrs =
                    window_attrs.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }

            // Create the window
            let window = event_loop
                .create_window(window_attrs)
                .expect("Failed to create window");

            // Store window in engine and get Arc reference for GPU initialization
            let window_arc = with_engine(|engine| {
                engine.set_window(window);
                engine.window.clone().expect("Window should be set")
            })
            .expect("Engine not initialized");

            // Initialize GPU
            match crate::renderer::init_gpu(window_arc) {
                Ok((instance, surface, device, queue, surface_config)) => {
                    // Create sprite pipeline (used for everything)
                    let (
                        sprite_pipeline,
                        sprite_bind_group_layout,
                        sprite_texture_bind_group_layout,
                        sprite_projection_buffer,
                    ) = match crate::renderer::create_sprite_pipeline(
                        &device,
                        surface_config.format,
                    ) {
                        Ok(pipeline) => pipeline,
                        Err(e) => {
                            eprintln!("Failed to create sprite pipeline: {}", e);
                            event_loop.exit();
                            return;
                        }
                    };

                    with_engine(|engine| {
                        // Render initial black frame to clear garbage
                        if let Err(e) =
                            crate::renderer::render_initial_frame(&surface, &device, &queue)
                        {
                            eprintln!("Warning: Failed to render initial frame: {}", e);
                        }

                        // Now show the window (it was created hidden)
                        if let Some(ref window) = engine.window {
                            window.set_visible(true);
                        }

                        engine.set_gpu(instance, surface, device, queue, surface_config);
                        engine.sprite_pipeline = Some(sprite_pipeline);
                        engine.sprite_bind_group_layout = Some(sprite_bind_group_layout);
                        engine.sprite_texture_bind_group_layout =
                            Some(sprite_texture_bind_group_layout);
                        engine.sprite_projection_buffer = Some(sprite_projection_buffer);
                    })
                    .expect("Engine not initialized");

                    // Call on_ready callback if provided
                    if !self.ready_called {
                        if let Some(ref on_ready) = self.on_ready_fn {
                            Python::attach(|py| {
                                if let Err(e) = on_ready.call0(py) {
                                    eprintln!("Error in on_ready callback: {}", e);
                                }
                            });
                        }
                        self.ready_called = true;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to initialize GPU: {}", e);
                    event_loop.exit();
                    return;
                }
            }

            self.window_created = true;
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                with_engine(|engine| {
                    engine.running = false;
                })
                .ok();
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                // Reconfigure surface with new size
                with_engine(|engine| {
                    engine.reconfigure_surface(physical_size.width, physical_size.height);
                })
                .ok();
            }
            WindowEvent::RedrawRequested => {
                // Start frame timing for FPS limit
                self.frame_limiter.frame_start();

                // Update timing
                let should_continue = with_engine(|engine| {
                    if !engine.running {
                        return false;
                    }

                    // Poll gamepad state
                    if let Some(gamepad) = &mut engine.gamepad {
                        gamepad.poll();
                    }

                    let now = Instant::now();
                    let frame_time = now.duration_since(engine.last_frame).as_secs_f32();
                    engine.last_frame = now;
                    engine.dt = frame_time;

                    if frame_time > 0.0 {
                        engine.fps = (1.0 / frame_time) as u32;
                    }

                    true
                })
                .unwrap_or(false);

                if !should_continue {
                    event_loop.exit();
                    return;
                }

                // Call Python update function
                let dt = with_engine(|engine| engine.dt as f64).unwrap_or(0.016);
                let mut should_exit = false;

                Python::attach(|py| match self.update_fn.call1(py, (dt,)) {
                    Ok(result) => {
                        if let Ok(continue_running) = result.extract::<bool>(py) {
                            if !continue_running {
                                should_exit = true;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error in update function: {}", e);
                        should_exit = true;
                    }
                });

                if should_exit {
                    with_engine(|engine| engine.running = false).ok();
                    event_loop.exit();
                    return;
                }

                // Call Python render function
                Python::attach(|py| {
                    if let Err(e) = self.render_fn.call0(py) {
                        eprintln!("Error in render function: {}", e);
                        event_loop.exit();
                    }
                });

                // Clear per-frame input state
                with_engine(|engine| {
                    engine.input.clear_frame_state();
                })
                .ok();

                // Sleep to maintain target frame rate
                self.frame_limiter.frame_end();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Handle keyboard input
                crate::input::handle_keyboard_input(event.physical_key, event.state.is_pressed());
            }
            WindowEvent::CursorMoved { position, .. } => {
                // Handle mouse movement
                crate::input::handle_mouse_move(position.x, position.y);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                // Handle mouse button
                crate::input::handle_mouse_button(button, state.is_pressed());
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Handle mouse wheel
                use winit::event::MouseScrollDelta;
                let wheel_delta = match delta {
                    MouseScrollDelta::LineDelta(_x, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => (pos.y / 16.0) as f32,
                };
                crate::input::handle_mouse_wheel(wheel_delta);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request next frame - this is called after present() blocks on VSync
        with_engine(|engine| {
            if engine.running {
                if let Some(window) = &engine.window {
                    window.request_redraw();
                }
            }
        })
        .ok();
    }
}

#[pyfunction]
#[pyo3(signature = (width, height, title, fullscreen=false, resizable=false, fps_limit=60))]
pub fn init(
    width: u32,
    height: u32,
    title: &str,
    fullscreen: bool,
    resizable: bool,
    fps_limit: u32,
) -> PyResult<()> {
    // Check if already initialized
    if ENGINE.get().is_some() {
        return Err(pyo3::exceptions::PyRuntimeError::new_err(
            "pgfx already initialized. Call quit() first.",
        ));
    }

    let config = EngineConfig {
        width,
        height,
        title: title.to_string(),
        fullscreen,
        resizable,
        fps_limit,
    };

    let engine = Engine::new(config);

    ENGINE
        .set(Mutex::new(engine))
        .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("Failed to initialize engine"))?;

    Ok(())
}

#[pyfunction]
#[pyo3(signature = (update_fn, render_fn, on_ready=None))]
pub fn run(
    update_fn: Py<PyAny>,
    render_fn: Py<PyAny>,
    on_ready: Option<Py<PyAny>>,
) -> PyResult<()> {
    // Check engine is initialized
    if ENGINE.get().is_none() {
        return Err(pyo3::exceptions::PyRuntimeError::new_err(
            "pgfx not initialized. Call init() first.",
        ));
    }

    // Get fps_limit from config
    let fps_limit = with_engine(|engine| engine.config.fps_limit)?;

    // Create event loop
    let event_loop = EventLoop::new().map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to create event loop: {}", e))
    })?;

    // Create application handler
    let mut app = AppHandler::new(update_fn, render_fn, on_ready, fps_limit);

    // Run the event loop - this takes ownership and never returns until exit
    event_loop.run_app(&mut app).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Event loop error: {}", e))
    })?;

    Ok(())
}

#[pyfunction]
pub fn quit() -> PyResult<()> {
    with_engine(|engine| {
        engine.running = false;
    })?;

    // Clear the global state
    // Note: OnceLock doesn't have a clear method, so we just mark as not running
    // The actual cleanup will happen when the process exits or when we implement
    // a more sophisticated cleanup mechanism

    Ok(())
}

#[pyfunction]
pub fn dt() -> PyResult<f64> {
    with_engine(|engine| engine.dt as f64)
}

#[pyfunction]
pub fn fps() -> PyResult<u32> {
    with_engine(|engine| engine.fps)
}

#[pyfunction]
pub fn time() -> PyResult<f64> {
    with_engine(|engine| engine.start_time.elapsed().as_secs_f64())
}

#[pyfunction]
pub fn screen_size() -> PyResult<(u32, u32)> {
    with_engine(|engine| (engine.config.width, engine.config.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests use thread-local testing since ENGINE is global
    // We cannot easily reset ENGINE between tests in the same process

    #[test]
    fn test_engine_config_creation() {
        let config = EngineConfig {
            width: 1024,
            height: 768,
            title: "Test".to_string(),
            fullscreen: false,
            resizable: true,
            fps_limit: 60,
        };

        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.title, "Test");
        assert!(!config.fullscreen);
        assert!(config.resizable);
        assert_eq!(config.fps_limit, 60);
    }

    #[test]
    fn test_engine_creation() {
        let config = EngineConfig {
            width: 800,
            height: 600,
            title: "Test Engine".to_string(),
            fullscreen: false,
            resizable: false,
            fps_limit: 0,
        };

        let engine = Engine::new(config);
        assert_eq!(engine.config.width, 800);
        assert_eq!(engine.config.height, 600);
        assert_eq!(engine.fps, 60);
        assert!((engine.dt - 0.016).abs() < 0.001);
        assert!(engine.running);
    }
}
