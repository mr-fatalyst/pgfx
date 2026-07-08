"""Text basics: sizes, colors, multiline, unicode and pixel-perfect fonts."""

import os

import pgfx

SCREEN_W, SCREEN_H = 800, 600
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx text")

font = font_big = font_pixel = None


def on_ready():
    global font, font_big, font_pixel
    path = os.path.join(os.path.dirname(__file__), "assets/font.ttf")
    font = pgfx.font_load(path, 22)
    font_big = pgfx.font_load(path, 48)
    font_pixel = pgfx.font_load(path, 22, smooth=False)  # pixel-perfect


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))
    if not font:
        return

    pgfx.text(font_big, "Hello, pgfx!", 50, 40, pgfx.YELLOW)
    pgfx.text(font, "Any unicode the font provides: Hello World! Ünïcodé", 50, 120, pgfx.WHITE)
    pgfx.text(font, "Multiline is one call:\nsecond line\nthird line", 50, 170, pgfx.CYAN)
    pgfx.text(font, "smooth=True (default)", 50, 290, pgfx.WHITE)
    pgfx.text(font_pixel, "smooth=False — pixel-perfect", 50, 320, pgfx.WHITE)
    pgfx.text(font, "align='center' and align='right' anchor x differently:", 50, 390, DIM)
    pgfx.line(400, 420, 400, 500, pgfx.Color(255, 255, 255, 60))
    pgfx.text(font, "centered on the line", 400, 430, pgfx.GREEN, align="center")
    pgfx.text(font, "ends at the line", 400, 465, pgfx.GREEN, align="right")

    pgfx.text(
        font, f"{pgfx.fps()} FPS — ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center"
    )


pgfx.run(update, render, on_ready=on_ready)
