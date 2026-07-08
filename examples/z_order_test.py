"""z-order: draw order in code does not matter, z does.

Three balls are drawn front-to-back in code — the OPPOSITE of how they
appear: the first call has the highest z and lands on top. Within the same
z, call order still applies.
"""

import os

import pgfx

SCREEN_W, SCREEN_H = 800, 600
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx z-order")

font = ball = None


def on_ready():
    global font, ball
    here = os.path.dirname(__file__)
    font = pgfx.font_load(os.path.join(here, "assets/font.ttf"), 18)
    ball = pgfx.sprite_load(os.path.join(here, "assets/ball.png"))


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))
    if not font:
        return

    pgfx.draw_ex(ball, 280, 180, z=2, scale=4)  # first call — on top (z=2)
    pgfx.draw_ex(ball, 340, 240, z=1, scale=4)
    pgfx.draw_ex(ball, 400, 300, z=0, scale=4)  # last call — at the back (z=0)

    pgfx.text(font, "z=2, drawn FIRST in code", 420, 190, pgfx.YELLOW)
    pgfx.text(font, "z=1", 480, 260, pgfx.WHITE)
    pgfx.text(font, "z=0, drawn LAST in code", 540, 330, pgfx.WHITE)

    pgfx.text(font, "the first-drawn ball covers the others — z wins over call order",
              SCREEN_W / 2, 470, pgfx.GREEN, align="center")
    pgfx.text(font, "ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center")


pgfx.run(update, render, on_ready=on_ready)
