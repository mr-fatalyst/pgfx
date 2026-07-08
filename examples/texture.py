"""Textures vs sprites: one texture, several sprites cut from its regions.

texture_load() uploads the pixels once; sprite_create() defines a drawable
rectangle inside them — that's how atlases work.
"""

import os

import pgfx

SCREEN_W, SCREEN_H = 800, 600
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx texture")

font = None
full = None
quarters = []
tex_size = (0, 0)


def on_ready():
    global font, full, tex_size
    here = os.path.dirname(__file__)
    font = pgfx.font_load(os.path.join(here, "assets/font.ttf"), 18)

    tex = pgfx.texture_load(os.path.join(here, "assets/test.png"))
    tex_size = pgfx.texture_size(tex)
    w, h = tex_size

    # the whole texture as one sprite, and its four quarters as separate ones
    full = pgfx.sprite_create(tex, 0, 0, w, h)
    for qy in (0, h // 2):
        for qx in (0, w // 2):
            quarters.append(pgfx.sprite_create(tex, qx, qy, w // 2, h // 2))


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))
    if not font:
        return

    w, h = tex_size
    pgfx.text(font, f"texture: {w}x{h} px", 60, 50, DIM)
    pgfx.draw_ex(full, 60, 80, scale=4)

    pgfx.text(font, "split into 4 region sprites", 400, 50, DIM)
    for i, spr in enumerate(quarters):
        x = 400 + (i % 2) * (w * 2 + 20)
        y = 80 + (i // 2) * (h * 2 + 20)
        pgfx.draw_ex(spr, x, y, scale=4)

    pgfx.text(font, "ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center")


pgfx.run(update, render, on_ready=on_ready)
