"""Text rendering example."""

import os

import pgfx

pgfx.init(800, 600, "Text Example")

font = None
font_big = None


def on_ready():
    global font, font_big
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 24)
    font_big = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 48)


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(30, 30, 50))

    if font:
        pgfx.text(font, "Hello, pgfx!", 50, 50, pgfx.WHITE)
        pgfx.text(font, f"FPS: {pgfx.fps()}", 50, 100, pgfx.GREEN)
        pgfx.text(font_big, "Big Text", 50, 200, pgfx.YELLOW)
        pgfx.text(font, "Press ESC to quit", 50, 500, pgfx.Color(150, 150, 150))


pgfx.run(update, render, on_ready=on_ready)
