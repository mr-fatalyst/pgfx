"""Error behavior of the API before init() and on invalid arguments.

These tests must run in a process where pgfx.init() was never called.
"""

import pgfx
import pytest

# (function, args) pairs that require an initialized engine
ENGINE_FUNCTIONS = [
    (pgfx.dt, ()),
    (pgfx.fps, ()),
    (pgfx.time, ()),
    (pgfx.screen_size, ()),
    (pgfx.quit, ()),
    (pgfx.key_down, (0,)),
    (pgfx.key_pressed, (0,)),
    (pgfx.key_released, (0,)),
    (pgfx.mouse_pos, ()),
    (pgfx.mouse_down, (0,)),
    (pgfx.mouse_pressed, (0,)),
    (pgfx.mouse_wheel, ()),
    (pgfx.gamepad_connected, ()),
    (pgfx.gamepad_button, (0, 0)),
    (pgfx.gamepad_axis, (0, 0)),
    (pgfx.gamepad_trigger, (0, 0)),
    (pgfx.texture_load, ("nonexistent.png",)),
    (pgfx.texture_free, (1,)),
    (pgfx.texture_size, (1,)),
    (pgfx.sprite_load, ("nonexistent.png",)),
    (pgfx.sprite_create, (1, 0, 0, 1, 1)),
    (pgfx.sprite_set_origin, (1, 0.0, 0.0)),
    (pgfx.sprite_set_color, (1, 255, 255, 255, 255)),
    (pgfx.sprite_free, (1,)),
    (pgfx.font_load, ("nonexistent.ttf", 16)),
    (pgfx.font_free, (1,)),
    (pgfx.sprite_rect, (1, 0.0, 0.0)),
    (pgfx.collide_sprites, (1, 0.0, 0.0, 2, 0.0, 0.0)),
    (pgfx.point_in_sprite, (0.0, 0.0, 1, 0.0, 0.0)),
    (pgfx.particles_create, ()),
    (pgfx.particles_free, (1,)),
    (pgfx.particles_fire, (1, 0.0, 0.0)),
    (pgfx.particles_emit, (1, 0.0, 0.0, 10)),
    (pgfx.particles_stop, (1,)),
    (pgfx.particles_move_to, (1, 0.0, 0.0)),
    (pgfx.particles_update, (1, 0.016)),
    (pgfx.particles_is_alive, (1,)),
    (pgfx.particles_count, (1,)),
    (pgfx.set_ambient, (pgfx.WHITE,)),
    (pgfx.light_create, (100.0, pgfx.WHITE)),
    (pgfx.light_set_intensity, (1, 1.0)),
    (pgfx.light_set_flicker, (1, 0.5, 1.0)),
    (pgfx.light_free, (1,)),
]


@pytest.mark.parametrize(
    "fn,args", ENGINE_FUNCTIONS, ids=[fn.__name__ for fn, _ in ENGINE_FUNCTIONS]
)
def test_engine_functions_raise_before_init(fn, args):
    with pytest.raises(RuntimeError, match="not initialized"):
        fn(*args)


def test_run_raises_before_init():
    with pytest.raises(RuntimeError, match="not initialized"):
        pgfx.run(lambda dt: False, lambda: None)


def test_particles_create_rejects_unknown_primitive():
    with pytest.raises(ValueError, match="Unknown primitive"):
        pgfx.particles_create(primitive="bogus")


def test_sprite_sheet_rejects_zero_dimensions():
    with pytest.raises(ValueError, match="greater than 0"):
        pgfx.sprite_sheet("nonexistent.png", 0, 1)


def test_particles_load_not_implemented():
    with pytest.raises(NotImplementedError):
        pgfx.particles_load("particles.json")
