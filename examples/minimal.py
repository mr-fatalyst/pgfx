"""Minimal pgfx example."""

import pgfx

pgfx.init(800, 600, "Minimal Example")

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
    pgfx.clear(pgfx.Color(30, 30, 50))
    pgfx.circle_fill(x, y, 20, pgfx.WHITE)
    pgfx.rect_fill(10, 10, 100, 20, pgfx.RED)


pgfx.run(update, render)
