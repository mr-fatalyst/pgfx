"""Lighting system example."""

import pgfx

pgfx.init(800, 600, "Lighting Example")

light1 = None
light2 = None
light3 = None

light1_pos = [400, 300]
light2_pos = [200, 200]
light3_pos = [600, 400]


def on_ready():
    global light1, light2, light3

    light1 = pgfx.light_create(200, pgfx.Color(255, 100, 100))  # Orange
    light2 = pgfx.light_create(150, pgfx.Color(100, 100, 255))  # Purple
    light3 = pgfx.light_create(180, pgfx.Color(100, 255, 100))  # Green

    pgfx.light_set_intensity(light2, 0.8)
    pgfx.light_set_flicker(light3, 0.3, 2.0)
    pgfx.set_ambient(pgfx.Color(50, 50, 70))


def update(dt):
    speed = 200
    if pgfx.key_down(pgfx.KEY_LEFT):
        light1_pos[0] -= speed * dt
    if pgfx.key_down(pgfx.KEY_RIGHT):
        light1_pos[0] += speed * dt
    if pgfx.key_down(pgfx.KEY_UP):
        light1_pos[1] -= speed * dt
    if pgfx.key_down(pgfx.KEY_DOWN):
        light1_pos[1] += speed * dt

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(20, 20, 30))

    pgfx.rect_fill(100, 100, 100, 100, pgfx.Color(80, 80, 80))
    pgfx.rect_fill(300, 200, 120, 80, pgfx.Color(100, 100, 100))
    pgfx.rect_fill(500, 350, 150, 120, pgfx.Color(90, 90, 90))

    if light1:
        pgfx.light_draw(light1, light1_pos[0], light1_pos[1])
        pgfx.light_draw(light2, light2_pos[0], light2_pos[1])
        pgfx.light_draw(light3, light3_pos[0], light3_pos[1])


pgfx.run(update, render, on_ready=on_ready)
