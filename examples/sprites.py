"""Sprite drawing: transforms, origin, flipping, alpha and z-order."""

import os

import pgfx

SCREEN_W, SCREEN_H = 960, 600
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx sprites")

font = spr = spr_centered = None


def on_ready():
    global font, spr, spr_centered
    here = os.path.dirname(__file__)
    font = pgfx.font_load(os.path.join(here, "assets/font.ttf"), 17)
    spr = pgfx.sprite_load(os.path.join(here, "assets/test.png"))

    # same image with the origin moved to the center: rotates in place
    spr_centered = pgfx.sprite_load(os.path.join(here, "assets/test.png"))
    w, h = pgfx.sprite_rect(spr_centered, 0, 0)[2:]
    pgfx.sprite_set_origin(spr_centered, w / 2, h / 2)


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))
    if not font:
        return
    t = pgfx.time()

    # row 1: plain draw, rotation around two origins, z-order
    pgfx.text(font, "draw", 60, 40, DIM)
    pgfx.draw(spr, 60, 80)

    pgfx.text(font, "rot — origin top-left vs center", 200, 40, DIM)
    pgfx.draw_ex(spr, 260, 170, rot=t)
    pgfx.draw_ex(spr_centered, 430, 170, rot=t)

    pgfx.text(font, "z: first call on top", 640, 40, DIM)
    pgfx.draw_ex(spr, 640, 80, z=2, scale=2)  # drawn first, shown on top
    pgfx.draw_ex(spr, 665, 105, z=1, scale=2)
    pgfx.draw_ex(spr, 690, 130, z=0, scale=2)  # drawn last, shown at the back

    # row 2: scaling and appearance
    pgfx.text(font, "scale=2 / scale_y=0.75", 60, 300, DIM)
    pgfx.draw_ex(spr, 60, 330, scale=2)
    pgfx.draw_ex(spr, 240, 330, scale=2, scale_y=0.75)

    pgfx.text(font, "flip_x / alpha=0.4", 460, 300, DIM)
    pgfx.draw_ex(spr, 460, 330, flip_x=True, scale=2)
    pgfx.draw_ex(spr, 640, 330, alpha=0.4, scale=2)

    pgfx.text(font, "ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center")


pgfx.run(update, render, on_ready=on_ready)
