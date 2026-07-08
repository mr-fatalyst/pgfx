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
CMD_SET_VIEW = 9
CMD_RESET_VIEW = 10
CMD_RECT_FILL_EX = 11
CMD_RENDER_TO = 12

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


_BLEND = {"alpha": 0, "add": 1}


def draw_ex(
    spr, x, y, rot=0, scale=1, alpha=1, flip_x=False, flip_y=False, z=0, scale_y=None, blend="alpha"
):
    """Draw a sprite with transformation options. z=0 by default (back), higher
    z = on top. scale_y (default: same as scale) stretches the sprite vertically
    for non-uniform scaling. blend "add" draws additively (glow, fire)."""
    b = _BLEND.get(blend)
    if b is None:
        raise ValueError(f"blend must be 'alpha' or 'add', got {blend!r}")
    sy = scale if scale_y is None else scale_y
    _commands.append((CMD_DRAW_EX, spr, x, y, rot, scale, alpha, flip_x, flip_y, z, sy, b))


def rect_fill(x, y, w, h, color, z=0):
    """Draw a filled rectangle."""
    _commands.append((CMD_RECT_FILL, x, y, w, h, color.r, color.g, color.b, color.a, z))


def rect_fill_ex(x, y, w, h, color, rot=0.0, z=0):
    """Draw a filled rectangle CENTERED on (x, y), rotated by rot around its
    center — like circle_fill, the anchor is the middle, not the corner."""
    _commands.append((CMD_RECT_FILL_EX, x, y, w, h, rot, color.r, color.g, color.b, color.a, z))


def line(x1, y1, x2, y2, color, z=0, width=2):
    """Draw a line of the given width (centered on the segment)."""
    _commands.append((CMD_LINE, x1, y1, x2, y2, color.r, color.g, color.b, color.a, z, width))


def circle_fill(x, y, r, color, z=0):
    """Draw a filled circle."""
    _commands.append((CMD_CIRCLE_FILL, x, y, r, color.r, color.g, color.b, color.a, z))


_TEXT_ALIGN = {"left": 0, "center": 1, "right": 2}


def text(font, string, x, y, color, z=0, align="left"):
    """Draw text. align ("left"/"center"/"right") sets what x anchors:
    each line's left edge, center or right edge."""
    a = _TEXT_ALIGN.get(align)
    if a is None:
        raise ValueError(f"align must be 'left', 'center' or 'right', got {align!r}")
    _commands.append((CMD_TEXT, font, string, x, y, color.r, color.g, color.b, color.a, z, a))


def particles_render(ps, z=0):
    """Render a particle system."""
    _commands.append((CMD_PARTICLES_RENDER, ps, z))


def light_draw(light, x, y, z=0):
    """Draw a light at position."""
    _commands.append((CMD_LIGHT_DRAW, light, x, y, z))


def render_to(target):
    """Redirect subsequent draw calls into a render target (see
    target_create), or back to the screen with render_to(None).

    Resets the view; clear() inside a target block sets that target's clear
    color (default: transparent). Target passes run before the screen pass,
    so a target drawn this frame can be shown on screen in the same frame.
    """
    _commands.append((CMD_RENDER_TO, 0 if target is None else target))


def set_view(x, y, zoom=1.0, rot=0.0):
    """Camera: place world point (x, y) at the screen center for all
    subsequent draw calls, zoomed and rotated around it.

    Resets to screen space at the start of every frame — call it in render()
    before drawing the world. Applies per call (z-sorting does not change
    which view a draw uses).
    """
    if zoom <= 0:
        raise ValueError(f"zoom must be positive, got {zoom}")
    _commands.append((CMD_SET_VIEW, float(x), float(y), float(zoom), float(rot)))


def reset_view():
    """Back to screen space (identity view) — call before drawing the HUD."""
    _commands.append((CMD_RESET_VIEW,))
