"""pgfx - Lightweight 2D game library for Python with Rust core."""

from pgfx import _native
from pgfx._batch import (
    circle_fill,
    clear,
    draw,
    draw_ex,
    light_draw,
    line,
    particles_render,
    rect_fill,
    text,
)
from pgfx._native import (
    collide_circle_rect,
    collide_circles,
    # Collision
    collide_rects,
    collide_sprites,
    dt,
    font_free,
    # Text
    font_load,
    fps,
    gamepad_axis,
    gamepad_button,
    gamepad_connected,
    gamepad_trigger,
    # Input
    key_down,
    key_pressed,
    key_released,
    light_free,
    light_set_flicker,
    # Lighting
    light_set_intensity,
    mouse_down,
    mouse_pos,
    mouse_pressed,
    mouse_wheel,
    music_free,
    music_load,
    music_pause,
    music_play,
    music_resume,
    music_stop,
    particles_count,
    particles_emit,
    particles_fire,
    particles_free,
    particles_is_alive,
    # Particles
    particles_load,
    particles_move_to,
    particles_stop,
    particles_update,
    point_in_circle,
    point_in_rect,
    point_in_sprite,
    quit,
    raycast_rect,
    screen_size,
    set_master_volume,
    set_music_volume,
    sound_free,
    # Audio
    sound_load,
    sound_play,
    sound_stop,
    sprite_create,
    sprite_free,
    # Sprite
    sprite_load,
    sprite_rect,
    sprite_set_color,
    sprite_set_origin,
    sprite_sheet,
    texture_free,
    # Texture
    texture_load,
    texture_size,
    time,
)
from pgfx._native import (
    # System
    init as _init,
)
from pgfx._native import (
    # Renderer
    render_batch as _render_batch,
)
from pgfx._native import (
    run as _run,
)
from pgfx.constants import *


def init(width: int, height: int, title: str, **opts):
    """Initialize pgfx window and graphics context."""
    _init(width, height, title, **opts)


def run(update_fn, render_fn, on_ready=None):
    """Run the main game loop.

    Args:
        update_fn: Called each frame with dt (delta time). Return False to exit.
        render_fn: Called each frame to render graphics.
        on_ready: Optional callback called once when GPU is initialized.
    """
    from pgfx._batch import _flush

    def _wrapped_render():
        render_fn()
        _flush()

    _run(update_fn, _wrapped_render, on_ready)


# Lighting wrappers (accept Color)
def set_ambient(color):
    """Set the ambient light color."""
    _native.set_ambient(color.r, color.g, color.b, color.a)


def light_create(radius, color):
    """Create a dynamic light."""
    return _native.light_create(radius, color.r, color.g, color.b, color.a)


def light_set_color(light, color):
    """Change a light's color."""
    _native.light_set_color(light, color.r, color.g, color.b, color.a)


# Particles wrappers (convert Color to tuple in kwargs)
def _convert_color(kwargs):
    for key in ("start_color", "end_color"):
        if key in kwargs and hasattr(kwargs[key], "r"):
            c = kwargs[key]
            kwargs[key] = (c.r, c.g, c.b, c.a)
    return kwargs


def particles_create(sprite=None, primitive=None, **kwargs):
    """Create a particle system."""
    return _native.particles_create(sprite, primitive, **_convert_color(kwargs))


def particles_set(ps, **kwargs):
    """Update particle system parameters."""
    _native.particles_set(ps, **_convert_color(kwargs))
