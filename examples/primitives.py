"""All the primitives: rectangles (plain and rotated), lines, circles."""

import os

import pgfx

SCREEN_W, SCREEN_H = 800, 600
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx primitives")

font = None


def on_ready():
    global font
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 17)


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))
    if not font:
        return
    t = pgfx.time()

    pgfx.text(font, "rect_fill — anchored at the top-left corner", 50, 40, DIM)
    pgfx.rect_fill(50, 65, 100, 60, pgfx.Color(226, 60, 54))
    pgfx.rect_fill(170, 65, 100, 60, pgfx.Color(90, 200, 110))
    pgfx.rect_fill(290, 65, 100, 60, pgfx.Color(64, 120, 230))

    pgfx.text(font, "rect_fill_ex — centered, rotates around the center", 50, 160, DIM)
    for i in range(4):
        pgfx.rect_fill_ex(100 + i * 120, 220, 80, 40, pgfx.Color(240, 200, 60), rot=t + i * 0.5)

    pgfx.text(font, "line — any width, centered on the segment", 50, 290, DIM)
    for i, width in enumerate((1, 3, 6, 12)):
        y = 325 + i * 22
        pgfx.line(50, y, 390, y, pgfx.WHITE, width=width)

    pgfx.text(font, "circle_fill — centered", 50, 440, DIM)
    pgfx.circle_fill(90, 510, 40, pgfx.CYAN)
    pgfx.circle_fill(190, 510, 25, pgfx.MAGENTA)
    pgfx.circle_fill(270, 510, 12, pgfx.Color(255, 128, 0))

    pgfx.text(font, "ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center")


pgfx.run(update, render, on_ready=on_ready)
