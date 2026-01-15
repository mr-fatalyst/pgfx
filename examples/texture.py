"""Texture loading example."""

import os

import pgfx

pgfx.init(800, 600, "Texture Example")

tex = None


def on_ready():
    global tex
    tex = pgfx.texture_load(os.path.join(os.path.dirname(__file__), "assets/test.png"))
    print(f"Texture loaded: {tex}")
    print(f"Size: {pgfx.texture_size(tex)}")


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(30, 30, 50))


pgfx.run(update, render, on_ready=on_ready)

if tex:
    pgfx.texture_free(tex)
    print("Texture freed")
