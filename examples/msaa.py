"""MSAA demo: the same spinning shapes with and without antialiasing.

The window is created with init(..., msaa=4), so everything drawn to the
screen gets smooth polygon edges. Render targets are never multisampled —
the right panel draws the exact same scene through a target, which is how
it looked before MSAA: jagged. One frame, honest comparison.
"""

import math
import os

import pgfx

SCREEN_W, SCREEN_H = 960, 600
PANEL_W, PANEL_H = 420, 460
BG = pgfx.Color(24, 26, 34)
PANEL_BG = pgfx.Color(32, 35, 46)
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx msaa", msaa=4)

font = target = target_spr = None


def on_ready():
    global font, target, target_spr
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 18)
    target = pgfx.target_create(PANEL_W, PANEL_H)
    target_spr = pgfx.target_sprite(target)


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def draw_shapes(ox, oy, t):
    """Slowly spinning primitives — polygon edges show aliasing the best."""
    cx, cy = ox + PANEL_W / 2, oy + PANEL_H / 2

    # nested rotating squares
    for i in range(4):
        size = 300 - i * 70
        c = pgfx.Color(90 + i * 45, 130 + i * 30, 220 - i * 40)
        pgfx.rect_fill_ex(cx, cy - 40, size, size, c, rot=t * 0.2 + i * 0.35, z=i)

    # a wheel of thick lines
    for i in range(8):
        a = t * 0.4 + i * math.pi / 4
        pgfx.line(
            cx - math.cos(a) * 150,
            cy + 160 - math.sin(a) * 40,
            cx + math.cos(a) * 150,
            cy + 160 + math.sin(a) * 40,
            pgfx.Color(240, 200, 60),
            z=5,
            width=6,
        )


def render():
    pgfx.clear(BG)
    if target is None:
        return
    t = pgfx.time()

    left_x, right_x, top_y = 30, SCREEN_W - PANEL_W - 30, 70

    # left panel: straight to the (multisampled) screen
    pgfx.rect_fill(left_x, top_y, PANEL_W, PANEL_H, PANEL_BG)
    draw_shapes(left_x, top_y, t)

    # right panel: the same scene through a render target (always 1 sample)
    pgfx.render_to(target)
    pgfx.clear(PANEL_BG)
    draw_shapes(0, 0, t)
    pgfx.render_to(None)
    pgfx.draw(target_spr, right_x, top_y)

    pgfx.text(font, "MSAA 4x — screen", left_x + PANEL_W / 2, 30, pgfx.WHITE, align="center")
    pgfx.text(font, "no MSAA — via render target", right_x + PANEL_W / 2, 30, pgfx.WHITE,
              align="center")
    pgfx.text(font, "ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center")


pgfx.run(update, render, on_ready=on_ready)
