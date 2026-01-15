"""Tests that API functions are exported correctly."""

import pgfx


def test_system_functions_exist():
    assert callable(pgfx.init)
    assert callable(pgfx.run)
    assert callable(pgfx.quit)
    assert callable(pgfx.dt)
    assert callable(pgfx.fps)
    assert callable(pgfx.time)
    assert callable(pgfx.screen_size)


def test_input_functions_exist():
    assert callable(pgfx.key_down)
    assert callable(pgfx.key_pressed)
    assert callable(pgfx.key_released)
    assert callable(pgfx.mouse_pos)
    assert callable(pgfx.mouse_down)
    assert callable(pgfx.mouse_pressed)
    assert callable(pgfx.mouse_wheel)
    assert callable(pgfx.gamepad_connected)
    assert callable(pgfx.gamepad_button)
    assert callable(pgfx.gamepad_axis)
    assert callable(pgfx.gamepad_trigger)


def test_texture_functions_exist():
    assert callable(pgfx.texture_load)
    assert callable(pgfx.texture_free)
    assert callable(pgfx.texture_size)


def test_sprite_functions_exist():
    assert callable(pgfx.sprite_load)
    assert callable(pgfx.sprite_create)
    assert callable(pgfx.sprite_sheet)
    assert callable(pgfx.sprite_set_origin)
    assert callable(pgfx.sprite_set_color)
    assert callable(pgfx.sprite_free)


def test_drawing_functions_exist():
    assert callable(pgfx.clear)
    assert callable(pgfx.draw)
    assert callable(pgfx.draw_ex)
    assert callable(pgfx.rect_fill)
    assert callable(pgfx.line)
    assert callable(pgfx.circle_fill)


def test_text_functions_exist():
    assert callable(pgfx.font_load)
    assert callable(pgfx.font_free)
    assert callable(pgfx.text)


def test_audio_functions_exist():
    assert callable(pgfx.sound_load)
    assert callable(pgfx.sound_free)
    assert callable(pgfx.sound_play)
    assert callable(pgfx.sound_stop)
    assert callable(pgfx.music_load)
    assert callable(pgfx.music_free)
    assert callable(pgfx.music_play)
    assert callable(pgfx.music_stop)
    assert callable(pgfx.music_pause)
    assert callable(pgfx.music_resume)
    assert callable(pgfx.set_master_volume)
    assert callable(pgfx.set_music_volume)


def test_collision_functions_exist():
    assert callable(pgfx.collide_rects)
    assert callable(pgfx.collide_circles)
    assert callable(pgfx.collide_circle_rect)
    assert callable(pgfx.point_in_rect)
    assert callable(pgfx.point_in_circle)
    assert callable(pgfx.raycast_rect)
    assert callable(pgfx.sprite_rect)
    assert callable(pgfx.collide_sprites)
    assert callable(pgfx.point_in_sprite)


def test_particle_functions_exist():
    assert callable(pgfx.particles_load)
    assert callable(pgfx.particles_create)
    assert callable(pgfx.particles_free)
    assert callable(pgfx.particles_set)
    assert callable(pgfx.particles_fire)
    assert callable(pgfx.particles_emit)
    assert callable(pgfx.particles_stop)
    assert callable(pgfx.particles_move_to)
    assert callable(pgfx.particles_update)
    assert callable(pgfx.particles_render)
    assert callable(pgfx.particles_is_alive)
    assert callable(pgfx.particles_count)


def test_lighting_functions_exist():
    assert callable(pgfx.set_ambient)
    assert callable(pgfx.light_create)
    assert callable(pgfx.light_set_color)
    assert callable(pgfx.light_set_intensity)
    assert callable(pgfx.light_set_flicker)
    assert callable(pgfx.light_draw)
    assert callable(pgfx.light_free)
