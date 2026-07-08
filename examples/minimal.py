"""Minimal interactive example: move a circle with the arrow keys."""

import pgfx

pgfx.init(800, 600, "pgfx minimal")

x, y = 400, 300


def update(dt):
    global x, y
    speed = 200 * dt

    if pgfx.key_down(pgfx.KEY_LEFT):
        x -= speed
    if pgfx.key_down(pgfx.KEY_RIGHT):
        x += speed
    if pgfx.key_down(pgfx.KEY_UP):
        y -= speed
    if pgfx.key_down(pgfx.KEY_DOWN):
        y += speed

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))
    pgfx.circle_fill(x, y, 20, pgfx.WHITE)


pgfx.run(update, render)
