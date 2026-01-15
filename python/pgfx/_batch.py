"""Command batching for draw calls."""

from pgfx._native import render_batch as _render_batch

# Command types
CMD_CLEAR = 0
CMD_DRAW = 1
CMD_DRAW_EX = 2
CMD_RECT_FILL = 3
CMD_LINE = 4
CMD_CIRCLE_FILL = 5
CMD_TEXT = 6
CMD_PARTICLES_RENDER = 7
CMD_LIGHT_DRAW = 8

# Command buffer
_commands = []


def _flush():
    """Flush all pending draw commands to the renderer."""
    global _commands
    if _commands:
        _render_batch(_commands)
        _commands = []


def clear(color):
    """Clear the screen with a color."""
    _commands.append((CMD_CLEAR, color.r, color.g, color.b, color.a))


def draw(spr, x, y, z=0):
    """Draw a sprite at position."""
    _commands.append((CMD_DRAW, spr, x, y, z))


def draw_ex(spr, x, y, rot=0, scale=1, alpha=1, flip_x=False, flip_y=False, z=0):
    """Draw a sprite with transformation options. z=0 by default (back), higher z = on top."""
    _commands.append((CMD_DRAW_EX, spr, x, y, rot, scale, alpha, flip_x, flip_y, z))


def rect_fill(x, y, w, h, color, z=0):
    """Draw a filled rectangle."""
    _commands.append((CMD_RECT_FILL, x, y, w, h, color.r, color.g, color.b, color.a, z))


def line(x1, y1, x2, y2, color, z=0):
    """Draw a line."""
    _commands.append((CMD_LINE, x1, y1, x2, y2, color.r, color.g, color.b, color.a, z))


def circle_fill(x, y, r, color, z=0):
    """Draw a filled circle."""
    _commands.append((CMD_CIRCLE_FILL, x, y, r, color.r, color.g, color.b, color.a, z))


def text(font, string, x, y, color, z=0):
    """Draw text."""
    _commands.append((CMD_TEXT, font, string, x, y, color.r, color.g, color.b, color.a, z))


def particles_render(ps, z=0):
    """Render a particle system."""
    _commands.append((CMD_PARTICLES_RENDER, ps, z))


def light_draw(light, x, y, z=0):
    """Draw a light at position."""
    _commands.append((CMD_LIGHT_DRAW, light, x, y, z))
