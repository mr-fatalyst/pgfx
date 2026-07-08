"""Blend modes: the same effects with blend="alpha" vs blend="add".

Two identical campfires side by side — the left one alpha-blended, the right
one additive: overlapping additive particles brighten into a glow instead of
stacking as opaque blobs. Click anywhere for an additive spark burst; the
two ball rows show draw_ex(blend=...) on overlapping sprites.
"""

import math
import os

import pgfx

SCREEN_W, SCREEN_H = 960, 600
BG = pgfx.Color(16, 14, 22)
DIM = pgfx.Color(150, 155, 165)

FIRE = dict(
    primitive="circle_soft",
    emission_rate=140,
    lifetime_min=0.4,
    lifetime_max=1.1,
    speed_min=40,
    speed_max=110,
    direction=-math.pi / 2,
    spread=math.pi / 6,
    gravity=(0, -60),
    start_color=(255, 160, 40, 200),
    end_color=(180, 30, 0, 0),
    start_size=22,
    end_size=5,
    max_particles=600,
)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx blend modes")

font = ball = fire_alpha = fire_add = sparks = None


def on_ready():
    global font, ball, fire_alpha, fire_add, sparks
    here = os.path.dirname(__file__)
    font = pgfx.font_load(os.path.join(here, "assets/font.ttf"), 18)
    ball = pgfx.sprite_load(os.path.join(here, "assets/ball.png"))

    fire_alpha = pgfx.particles_create(**FIRE)  # blend defaults to "alpha"
    fire_add = pgfx.particles_create(**FIRE, blend="add")
    sparks = pgfx.particles_create(
        primitive="square",
        speed_min=120,
        speed_max=420,
        lifetime_min=0.2,
        lifetime_max=0.6,
        spread=math.pi * 2,
        gravity=(0, 500),
        start_color=(255, 230, 150, 255),
        end_color=(255, 90, 20, 0),
        start_size=3.5,
        end_size=1,
        max_particles=900,
        blend="add",
    )
    pgfx.particles_fire(fire_alpha, SCREEN_W / 2 - 220, 420)
    pgfx.particles_fire(fire_add, SCREEN_W / 2 + 220, 420)


def update(dt):
    if fire_alpha is None:
        return True
    pgfx.particles_update(fire_alpha, dt)
    pgfx.particles_update(fire_add, dt)
    pgfx.particles_update(sparks, dt)
    if pgfx.mouse_pressed(pgfx.MOUSE_LEFT):
        pgfx.particles_emit(sparks, *pgfx.mouse_pos(), 60)
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def draw_logs(cx, cy):
    for rot in (0.4, -0.4):
        pgfx.rect_fill_ex(cx, cy + 14, 90, 14, pgfx.Color(90, 60, 40), rot=rot, z=1)


def render():
    pgfx.clear(BG)
    if not font:
        return

    lx, rx = SCREEN_W / 2 - 220, SCREEN_W / 2 + 220
    draw_logs(lx, 420)
    draw_logs(rx, 420)
    pgfx.particles_render(fire_alpha, z=2)
    pgfx.particles_render(fire_add, z=2)
    pgfx.particles_render(sparks, z=3)
    pgfx.text(font, 'blend="alpha"', lx, 480, DIM, align="center")
    pgfx.text(font, 'blend="add"', rx, 480, DIM, align="center")

    # same overlapping sprites, two blend modes
    for i in range(4):
        pgfx.draw_ex(ball, lx - 60 + i * 28, 90, scale=1.6, alpha=0.6)
        pgfx.draw_ex(ball, rx - 60 + i * 28, 90, scale=1.6, alpha=0.6, blend="add")
    pgfx.text(font, "alpha stacks, add accumulates", SCREEN_W / 2, 160, DIM, align="center")

    hint = "CLICK FOR SPARKS   ESC TO QUIT"
    pgfx.text(font, hint, SCREEN_W / 2, SCREEN_H - 34, DIM, align="center")


pgfx.run(update, render, on_ready=on_ready)
