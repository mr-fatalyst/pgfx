"""Render targets: draw a scene once, show it on many "screens".

target_create() gives a texture you can draw into with render_to(); the
result is an ordinary sprite. One animated 320x240 scene below is rendered
per frame and displayed five times — scaled, rotated, tinted and additive —
all in the same frame.
"""

import math
import os

import pgfx

SCREEN_W, SCREEN_H = 960, 600
SCENE_W, SCENE_H = 320, 240

BG = pgfx.Color(18, 18, 26)
FRAME = pgfx.Color(70, 74, 90)
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx render targets")

font = scene = scene_spr = None


def on_ready():
    global font, scene, scene_spr
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 17)
    scene = pgfx.target_create(SCENE_W, SCENE_H)
    scene_spr = pgfx.target_sprite(scene)
    pgfx.sprite_set_origin(scene_spr, SCENE_W / 2, SCENE_H / 2)


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def draw_scene(t):
    """The picture on every TV: three orbiting balls over a dark backdrop."""
    pgfx.render_to(scene)
    pgfx.clear(pgfx.Color(30, 24, 48))
    for gx in range(0, SCENE_W + 1, 40):
        pgfx.line(gx, 0, gx, SCENE_H, pgfx.Color(255, 255, 255, 14))
    colors = [pgfx.Color(226, 60, 54), pgfx.Color(240, 200, 60), pgfx.Color(90, 200, 110)]
    for i, color in enumerate(colors):
        a = t * (1.0 + i * 0.3) + i * 2.1
        x = SCENE_W / 2 + math.cos(a) * (60 + i * 30)
        y = SCENE_H / 2 + math.sin(a) * (40 + i * 20)
        pgfx.circle_fill(x, y, 18 - i * 4, color, z=1)
    pgfx.text(font, "LIVE", 8, 6, pgfx.Color(255, 80, 70), z=2)
    pgfx.render_to(None)


def tv(x, y, scale, rot=0.0, alpha=1.0, blend="alpha"):
    w, h = SCENE_W * scale + 12, SCENE_H * scale + 12
    pgfx.rect_fill_ex(x, y, w, h, FRAME, rot=rot, z=3)
    pgfx.draw_ex(scene_spr, x, y, rot=rot, scale=scale, alpha=alpha, z=4, blend=blend)


def render():
    pgfx.clear(BG)
    if scene is None:
        return
    t = pgfx.time()

    draw_scene(t)  # rendered before the screen pass, ready this same frame

    # a huge dim copy as the backdrop, then a wall of TVs
    pgfx.draw_ex(scene_spr, SCREEN_W / 2, SCREEN_H / 2, scale=3.2, alpha=0.12)
    tv(SCREEN_W / 2, 280, 1.0)
    tv(180, 170, 0.42, rot=math.sin(t * 0.8) * 0.15)
    tv(SCREEN_W - 180, 170, 0.42, rot=-math.sin(t * 0.8) * 0.15)
    tv(180, 440, 0.42, rot=0.1, blend="add", alpha=0.9)
    tv(SCREEN_W - 180, 440, 0.42, rot=-0.1, alpha=0.55)

    pgfx.text(
        font,
        "ONE SCENE, FIVE SCREENS — ESC TO QUIT",
        SCREEN_W / 2,
        SCREEN_H - 32,
        DIM,
        align="center",
    )


pgfx.run(update, render, on_ready=on_ready)
