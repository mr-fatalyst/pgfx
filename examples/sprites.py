"""Sprites example - loading and drawing sprites with transformations and z-order."""

import os

import pgfx

pgfx.init(800, 600, "Sprites Example")

spr = None
angle = 0


def on_ready():
    global spr
    spr = pgfx.sprite_load(os.path.join(os.path.dirname(__file__), "assets/test.png"))


def update(dt):
    global angle
    angle += dt * 2
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(30, 30, 50))

    if spr:
        # Basic sprites
        pgfx.draw(spr, 100, 100)
        pgfx.draw_ex(spr, 300, 200, rot=angle, scale=1.0)
        pgfx.draw_ex(spr, 500, 200, flip_x=True)
        pgfx.draw_ex(spr, 400, 400, alpha=0.5, scale=1.5)

        # Z-order demo: three overlapping sprites
        # Drawn in code order: back, middle, front
        # But z controls actual layering
        pgfx.draw_ex(spr, 600, 350, z=0, scale=1.2)  # Back layer
        pgfx.draw_ex(spr, 620, 370, z=1, scale=1.2)  # Middle layer
        pgfx.draw_ex(spr, 640, 390, z=2, scale=1.2)  # Front layer


pgfx.run(update, render, on_ready=on_ready)
