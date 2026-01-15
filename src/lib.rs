mod audio;
mod collision;
mod engine;
mod input;
mod lighting;
mod particles;
mod renderer;
mod resources;
mod sprite;
mod text;
mod texture;
mod window;

use pyo3::prelude::*;

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // System
    m.add_function(wrap_pyfunction!(engine::init, m)?)?;
    m.add_function(wrap_pyfunction!(engine::run, m)?)?;
    m.add_function(wrap_pyfunction!(engine::quit, m)?)?;
    m.add_function(wrap_pyfunction!(engine::dt, m)?)?;
    m.add_function(wrap_pyfunction!(engine::fps, m)?)?;
    m.add_function(wrap_pyfunction!(engine::time, m)?)?;
    m.add_function(wrap_pyfunction!(engine::screen_size, m)?)?;

    // Input
    m.add_function(wrap_pyfunction!(input::key_down, m)?)?;
    m.add_function(wrap_pyfunction!(input::key_pressed, m)?)?;
    m.add_function(wrap_pyfunction!(input::key_released, m)?)?;
    m.add_function(wrap_pyfunction!(input::mouse_pos, m)?)?;
    m.add_function(wrap_pyfunction!(input::mouse_down, m)?)?;
    m.add_function(wrap_pyfunction!(input::mouse_pressed, m)?)?;
    m.add_function(wrap_pyfunction!(input::mouse_wheel, m)?)?;
    m.add_function(wrap_pyfunction!(input::gamepad_connected, m)?)?;
    m.add_function(wrap_pyfunction!(input::gamepad_button, m)?)?;
    m.add_function(wrap_pyfunction!(input::gamepad_axis, m)?)?;
    m.add_function(wrap_pyfunction!(input::gamepad_trigger, m)?)?;

    // Texture
    m.add_function(wrap_pyfunction!(texture::texture_load, m)?)?;
    m.add_function(wrap_pyfunction!(texture::texture_free, m)?)?;
    m.add_function(wrap_pyfunction!(texture::texture_size, m)?)?;

    // Sprite
    m.add_function(wrap_pyfunction!(sprite::sprite_load, m)?)?;
    m.add_function(wrap_pyfunction!(sprite::sprite_create, m)?)?;
    m.add_function(wrap_pyfunction!(sprite::sprite_sheet, m)?)?;
    m.add_function(wrap_pyfunction!(sprite::sprite_set_origin, m)?)?;
    m.add_function(wrap_pyfunction!(sprite::sprite_set_color, m)?)?;
    m.add_function(wrap_pyfunction!(sprite::sprite_free, m)?)?;

    // Renderer (drawing)
    m.add_function(wrap_pyfunction!(renderer::render_batch, m)?)?;

    // Text
    m.add_function(wrap_pyfunction!(text::font_load, m)?)?;
    m.add_function(wrap_pyfunction!(text::font_free, m)?)?;

    // Collision
    m.add_function(wrap_pyfunction!(collision::collide_rects, m)?)?;
    m.add_function(wrap_pyfunction!(collision::collide_circles, m)?)?;
    m.add_function(wrap_pyfunction!(collision::collide_circle_rect, m)?)?;
    m.add_function(wrap_pyfunction!(collision::point_in_rect, m)?)?;
    m.add_function(wrap_pyfunction!(collision::point_in_circle, m)?)?;
    m.add_function(wrap_pyfunction!(collision::raycast_rect, m)?)?;
    m.add_function(wrap_pyfunction!(collision::sprite_rect, m)?)?;
    m.add_function(wrap_pyfunction!(collision::collide_sprites, m)?)?;
    m.add_function(wrap_pyfunction!(collision::point_in_sprite, m)?)?;

    // Audio
    m.add_function(wrap_pyfunction!(audio::sound_load, m)?)?;
    m.add_function(wrap_pyfunction!(audio::sound_free, m)?)?;
    m.add_function(wrap_pyfunction!(audio::sound_play, m)?)?;
    m.add_function(wrap_pyfunction!(audio::sound_stop, m)?)?;
    m.add_function(wrap_pyfunction!(audio::music_load, m)?)?;
    m.add_function(wrap_pyfunction!(audio::music_free, m)?)?;
    m.add_function(wrap_pyfunction!(audio::music_play, m)?)?;
    m.add_function(wrap_pyfunction!(audio::music_stop, m)?)?;
    m.add_function(wrap_pyfunction!(audio::music_pause, m)?)?;
    m.add_function(wrap_pyfunction!(audio::music_resume, m)?)?;
    m.add_function(wrap_pyfunction!(audio::set_master_volume, m)?)?;
    m.add_function(wrap_pyfunction!(audio::set_music_volume, m)?)?;

    // Particles
    m.add_function(wrap_pyfunction!(particles::particles_load, m)?)?;
    m.add_function(wrap_pyfunction!(particles::particles_create, m)?)?;
    m.add_function(wrap_pyfunction!(particles::particles_free, m)?)?;
    m.add_function(wrap_pyfunction!(particles::particles_set, m)?)?;
    m.add_function(wrap_pyfunction!(particles::particles_fire, m)?)?;
    m.add_function(wrap_pyfunction!(particles::particles_emit, m)?)?;
    m.add_function(wrap_pyfunction!(particles::particles_stop, m)?)?;
    m.add_function(wrap_pyfunction!(particles::particles_move_to, m)?)?;
    m.add_function(wrap_pyfunction!(particles::particles_update, m)?)?;
    m.add_function(wrap_pyfunction!(particles::particles_render, m)?)?;
    m.add_function(wrap_pyfunction!(particles::particles_is_alive, m)?)?;
    m.add_function(wrap_pyfunction!(particles::particles_count, m)?)?;

    // Lighting
    m.add_function(wrap_pyfunction!(lighting::set_ambient, m)?)?;
    m.add_function(wrap_pyfunction!(lighting::light_create, m)?)?;
    m.add_function(wrap_pyfunction!(lighting::light_set_color, m)?)?;
    m.add_function(wrap_pyfunction!(lighting::light_set_intensity, m)?)?;
    m.add_function(wrap_pyfunction!(lighting::light_set_flicker, m)?)?;
    m.add_function(wrap_pyfunction!(lighting::light_free, m)?)?;

    Ok(())
}
