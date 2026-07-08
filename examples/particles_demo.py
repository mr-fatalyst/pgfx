"""Particle systems: fire, smoke, sparks and explosions under the mouse.

Fire, sparks and the explosion use blend="add" — overlapping particles
brighten into a glow. Smoke stays alpha-blended: it should occlude.

Controls:
    1-4     switch effect
    LMB     hold to emit (explosion: click for a burst)
    Esc     quit
"""

import math
import os

import pgfx

SCREEN_W, SCREEN_H = 1280, 720
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx particles")

font = None
effects = []  # (name, ps, continuous)
current = 0


def on_ready():
    global font
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 18)

    fire = pgfx.particles_create(
        primitive="circle_soft",
        emission_rate=80,
        lifetime_min=0.4,
        lifetime_max=1.0,
        speed_min=60,
        speed_max=120,
        direction=-math.pi / 2,
        spread=math.pi / 5,
        gravity=(0, -80),
        start_color=(255, 180, 50, 255),
        end_color=(200, 30, 0, 0),
        start_size=18,
        end_size=4,
        max_particles=400,
        blend="add",
    )
    smoke = pgfx.particles_create(
        primitive="circle_soft",
        emission_rate=30,
        lifetime_min=1.5,
        lifetime_max=3.0,
        speed_min=20,
        speed_max=50,
        direction=-math.pi / 2,
        spread=math.pi / 4,
        gravity=(20, -30),
        start_color=(100, 100, 100, 200),
        end_color=(50, 50, 50, 0),
        start_size=8,
        end_size=30,
        max_particles=200,
    )
    sparks = pgfx.particles_create(
        primitive="circle",
        emission_rate=120,
        lifetime_min=0.4,
        lifetime_max=1.2,
        speed_min=150,
        speed_max=380,
        direction=-math.pi / 2,
        spread=math.pi * 0.7,
        gravity=(0, 420),
        start_color=(255, 240, 180, 255),
        end_color=(255, 60, 0, 0),
        start_size=2,
        end_size=1,
        max_particles=500,
        blend="add",
    )
    explosion = pgfx.particles_create(
        primitive="circle_soft",
        emission_rate=0,  # burst only
        lifetime_min=0.3,
        lifetime_max=1.0,
        speed_min=80,
        speed_max=350,
        spread=math.pi * 2,
        gravity=(0, 180),
        start_color=(255, 220, 100, 255),
        end_color=(80, 20, 0, 0),
        start_size=35,
        end_size=8,
        max_particles=400,
        blend="add",
    )

    effects.append(("Fire", fire, True))
    effects.append(("Smoke", smoke, True))
    effects.append(("Sparks", sparks, True))
    effects.append(("Explosion", explosion, False))


def update(dt):
    global current
    if not effects:
        return True

    for i, key in enumerate((pgfx.KEY_1, pgfx.KEY_2, pgfx.KEY_3, pgfx.KEY_4)):
        if pgfx.key_pressed(key):
            current = i

    mx, my = pgfx.mouse_pos()
    name, ps, continuous = effects[current]

    for _, other, _ in effects:
        if other is not ps:
            pgfx.particles_stop(other)

    if continuous:
        if pgfx.mouse_down(pgfx.MOUSE_LEFT):
            pgfx.particles_fire(ps, mx, my)
        else:
            pgfx.particles_stop(ps)
    elif pgfx.mouse_pressed(pgfx.MOUSE_LEFT):
        pgfx.particles_emit(ps, mx, my, 100)

    for _, other, _ in effects:
        pgfx.particles_update(other, dt)

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(15, 15, 25))
    if not effects:
        return

    for _, ps, _ in effects:
        pgfx.particles_render(ps)

    total = sum(pgfx.particles_count(ps) for _, ps, _ in effects)
    stats = f"FPS: {pgfx.fps()}   particles: {total}"
    labels = [f"{i + 1}: {name}" for i, (name, _, _) in enumerate(effects)]
    hint = "hold LMB to emit (4: click)"

    # panel sized to its content, labels laid out by measured width
    gap = 26
    row_w = sum(pgfx.text_size(font, s)[0] for s in labels) + gap * (len(labels) - 1)
    panel_w = max(row_w, pgfx.text_size(font, stats)[0], pgfx.text_size(font, hint)[0]) + 20
    pgfx.rect_fill(8, 8, panel_w, 88, pgfx.Color(0, 0, 0, 180), z=1)

    pgfx.text(font, stats, 18, 16, pgfx.WHITE, z=2)
    x = 18
    for i, label in enumerate(labels):
        pgfx.text(font, label, x, 42, pgfx.YELLOW if i == current else DIM, z=2)
        x += pgfx.text_size(font, label)[0] + gap
    pgfx.text(font, hint, 18, 68, DIM, z=2)

    pgfx.text(font, "ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center")


pgfx.run(update, render, on_ready=on_ready)
