"""Particle systems demo - fire, smoke, sparks, explosions"""

import math
import os

import pgfx

SCREEN_W, SCREEN_H = 1280, 720

pgfx.init(SCREEN_W, SCREEN_H, "Particles Demo - Click to spawn effects, 1-4 to switch type")

# Resources (loaded in on_ready)
font = None
fire_ps = None
smoke_ps = None
sparks_ps = None
explosion_ps = None

# Current effect type
current_effect = 1  # 1=fire, 2=smoke, 3=sparks, 4=explosion


def on_ready():
    """Called once when GPU is ready"""
    global font, fire_ps, smoke_ps, sparks_ps, explosion_ps

    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 18)

    # Create fire particle system (soft circles)
    fire_ps = pgfx.particles_create(
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
    )

    # Create smoke particle system
    smoke_ps = pgfx.particles_create(
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

    # Create sparks particle system
    sparks_ps = pgfx.particles_create(
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
    )

    # Create explosion particle system
    explosion_ps = pgfx.particles_create(
        primitive="circle_soft",
        emission_rate=0,
        lifetime_min=0.3,
        lifetime_max=1.0,
        speed_min=80,
        speed_max=350,
        direction=0,
        spread=math.pi * 2,
        gravity=(0, 180),
        start_color=(255, 220, 100, 255),
        end_color=(80, 20, 0, 0),
        start_size=35,
        end_size=8,
        max_particles=400,
    )


def update(dt):
    global current_effect

    if not fire_ps:
        return True

    # Switch effect type
    if pgfx.key_pressed(pgfx.KEY_1):
        current_effect = 1
        pgfx.particles_stop(smoke_ps)
        pgfx.particles_stop(sparks_ps)
    elif pgfx.key_pressed(pgfx.KEY_2):
        current_effect = 2
        pgfx.particles_stop(fire_ps)
        pgfx.particles_stop(sparks_ps)
    elif pgfx.key_pressed(pgfx.KEY_3):
        current_effect = 3
        pgfx.particles_stop(fire_ps)
        pgfx.particles_stop(smoke_ps)
    elif pgfx.key_pressed(pgfx.KEY_4):
        current_effect = 4
        pgfx.particles_stop(fire_ps)
        pgfx.particles_stop(smoke_ps)
        pgfx.particles_stop(sparks_ps)

    mouse_x, mouse_y = pgfx.mouse_pos()

    if pgfx.mouse_down(pgfx.MOUSE_LEFT):
        if current_effect == 1:
            pgfx.particles_fire(fire_ps, mouse_x, mouse_y)
        elif current_effect == 2:
            pgfx.particles_fire(smoke_ps, mouse_x, mouse_y)
        elif current_effect == 3:
            pgfx.particles_fire(sparks_ps, mouse_x, mouse_y)
    else:
        if current_effect != 4:
            pgfx.particles_stop(fire_ps)
            pgfx.particles_stop(smoke_ps)
            pgfx.particles_stop(sparks_ps)

    if current_effect == 4 and pgfx.mouse_pressed(pgfx.MOUSE_LEFT):
        pgfx.particles_emit(explosion_ps, mouse_x, mouse_y, 100)

    pgfx.particles_move_to(fire_ps, mouse_x, mouse_y)
    pgfx.particles_move_to(smoke_ps, mouse_x, mouse_y)
    pgfx.particles_move_to(sparks_ps, mouse_x, mouse_y)

    pgfx.particles_update(fire_ps, dt)
    pgfx.particles_update(smoke_ps, dt)
    pgfx.particles_update(sparks_ps, dt)
    pgfx.particles_update(explosion_ps, dt)

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(15, 15, 25))

    if not fire_ps:
        return

    pgfx.particles_render(fire_ps)
    pgfx.particles_render(smoke_ps)
    pgfx.particles_render(sparks_ps)
    pgfx.particles_render(explosion_ps)

    pgfx.rect_fill(5, 5, 300, 130, pgfx.Color(0, 0, 0, 180))
    pgfx.text(font, f"FPS: {pgfx.fps()}", 15, 12, pgfx.Color(255, 255, 255))

    total = (
        pgfx.particles_count(fire_ps)
        + pgfx.particles_count(smoke_ps)
        + pgfx.particles_count(sparks_ps)
        + pgfx.particles_count(explosion_ps)
    )
    pgfx.text(font, f"Particles: {total}", 15, 35, pgfx.Color(255, 255, 255))

    effects = ["1: Fire", "2: Smoke", "3: Sparks", "4: Explosion"]
    for i, name in enumerate(effects):
        color = pgfx.Color(255, 255, 0) if current_effect == i + 1 else pgfx.Color(150, 150, 150)
        pgfx.text(font, name, 15 + i * 75, 60, color)

    pgfx.text(font, "Hold LMB to emit, ESC to exit", 15, 85, pgfx.Color(100, 100, 100))
    pgfx.text(font, "Press 1-4 to switch effect", 15, 108, pgfx.Color(100, 100, 100))


pgfx.run(update, render, on_ready=on_ready)
