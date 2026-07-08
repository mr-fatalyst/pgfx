"""Lighting: ambient darkness plus moving, flickering and pulsing lights.

set_ambient() darkens the whole frame; light_draw() adds light on top.
The warm lamp follows the mouse, the torch flickers, the beacon pulses.
"""

import math
import os

import pgfx

SCREEN_W, SCREEN_H = 800, 600
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx lighting")

font = lamp = torch = beacon = None


def on_ready():
    global font, lamp, torch, beacon
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 17)

    lamp = pgfx.light_create(220, pgfx.Color(255, 190, 120))
    torch = pgfx.light_create(150, pgfx.Color(255, 130, 60))
    pgfx.light_set_flicker(torch, 0.35, 2.5)
    beacon = pgfx.light_create(180, pgfx.Color(110, 160, 255))

    pgfx.set_ambient(pgfx.Color(40, 40, 60))  # how dark the unlit scene is


def update(dt):
    if beacon:
        pulse = 0.5 + 0.5 * math.sin(pgfx.time() * 2.0)
        pgfx.light_set_intensity(beacon, 0.3 + 0.7 * pulse)
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(30, 32, 40))
    if not font:
        return

    # something to light: a grid of pillars
    for gx in range(80, SCREEN_W, 160):
        for gy in range(100, SCREEN_H - 100, 140):
            pgfx.rect_fill(gx, gy, 60, 60, pgfx.Color(110, 105, 95))
            pgfx.rect_fill(gx, gy, 60, 8, pgfx.Color(140, 135, 120))

    mx, my = pgfx.mouse_pos()
    pgfx.light_draw(lamp, mx, my)
    pgfx.light_draw(torch, 150, 460)
    pgfx.light_draw(beacon, 650, 150)

    pgfx.text(
        font, "MOVE THE MOUSE — ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center"
    )


pgfx.run(update, render, on_ready=on_ready)
