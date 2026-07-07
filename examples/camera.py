"""Camera demo: drive a buggy around a big world with set_view()/reset_view().

set_view(x, y, zoom, rot) places world point (x, y) at the screen center for
everything drawn after it; reset_view() switches back to screen space, which
is where the HUD lives. The view resets every frame, so both are called from
render().

Controls:
    Up / Down      drive          Left / Right   turn
    Q / E          zoom out / in  R (hold)       rotate camera behind the buggy
    Esc            quit
"""

import math
import random

import pgfx

SCREEN_W, SCREEN_H = 1280, 720
WORLD_W, WORLD_H = 4000, 3000

GRASS = pgfx.Color(50, 104, 58)
GRID = pgfx.Color(255, 255, 255, 14)
TREE_DARK = pgfx.Color(30, 78, 40)
TREE_LIGHT = pgfx.Color(48, 116, 60)
ROCK = pgfx.Color(130, 128, 122)
WALL = pgfx.Color(90, 70, 50)
HUD = pgfx.Color(255, 255, 255)
HUD_DIM = pgfx.Color(190, 195, 205)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx camera demo")

font = None

# Player buggy
px, py = WORLD_W / 2, WORLD_H / 2
heading = -math.pi / 2
speed = 0.0

# Camera state
cam_x, cam_y = px, py
zoom = 1.0
cam_rot = 0.0

# Scenery: (kind, x, y, size), generated once with a fixed seed
_rng = random.Random(11)


def _scatter(kind, count, size_min, size_max):
    return [
        (kind, _rng.uniform(0, WORLD_W), _rng.uniform(0, WORLD_H), _rng.uniform(size_min, size_max))
        for _ in range(count)
    ]


scenery = _scatter("tree", 130, 10, 24) + _scatter("rock", 60, 5, 12)


def on_ready():
    global font
    import os

    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 17)


def update(dt):
    global px, py, heading, speed, cam_x, cam_y, zoom, cam_rot

    # tank-style driving
    accel = 0.0
    if pgfx.key_down(pgfx.KEY_UP) or pgfx.key_down(pgfx.KEY_W):
        accel += 500.0
    if pgfx.key_down(pgfx.KEY_DOWN) or pgfx.key_down(pgfx.KEY_S):
        accel -= 350.0
    speed += accel * dt
    speed -= speed * 1.5 * dt  # drag
    turn = 0.0
    if pgfx.key_down(pgfx.KEY_LEFT) or pgfx.key_down(pgfx.KEY_A):
        turn -= 1.0
    if pgfx.key_down(pgfx.KEY_RIGHT) or pgfx.key_down(pgfx.KEY_D):
        turn += 1.0
    heading += turn * 2.4 * dt * min(1.0, abs(speed) / 120.0) * (1 if speed >= 0 else -1)
    px = max(20, min(WORLD_W - 20.0, px + math.cos(heading) * speed * dt))
    py = max(20, min(WORLD_H - 20.0, py + math.sin(heading) * speed * dt))

    # camera: smooth follow, Q/E zoom, R rotates the world behind the buggy
    if pgfx.key_down(pgfx.KEY_Q):
        zoom = max(0.35, zoom / (1 + 1.5 * dt))
    if pgfx.key_down(pgfx.KEY_E):
        zoom = min(3.0, zoom * (1 + 1.5 * dt))
    rot_target = -heading - math.pi / 2 if pgfx.key_down(pgfx.KEY_R) else 0.0
    k = 1 - math.exp(-6.0 * dt)
    cam_x += (px - cam_x) * k
    cam_y += (py - cam_y) * k
    d = (rot_target - cam_rot + math.pi) % (2 * math.pi) - math.pi
    cam_rot += d * k

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def draw_world():
    for gx in range(0, WORLD_W + 1, 250):
        pgfx.line(gx, 0, gx, WORLD_H, GRID, width=2)
    for gy in range(0, WORLD_H + 1, 250):
        pgfx.line(0, gy, WORLD_W, gy, GRID, width=2)

    # world border
    pgfx.rect_fill(-12, -12, WORLD_W + 24, 12, WALL)
    pgfx.rect_fill(-12, WORLD_H, WORLD_W + 24, 12, WALL)
    pgfx.rect_fill(-12, 0, 12, WORLD_H, WALL)
    pgfx.rect_fill(WORLD_W, 0, 12, WORLD_H, WALL)

    for kind, x, y, size in scenery:
        if kind == "tree":
            pgfx.circle_fill(x + 2, y + 3, size, pgfx.Color(0, 0, 0, 45))
            pgfx.circle_fill(x, y, size, TREE_DARK)
            pgfx.circle_fill(x - size * 0.25, y - size * 0.25, size * 0.55, TREE_LIGHT)
        else:
            pgfx.circle_fill(x, y, size, ROCK)

    # the buggy: thick-line body, four wheels, windshield
    ch, sh = math.cos(heading), math.sin(heading)

    def at(lx, ly):  # local buggy coords -> world
        return px + lx * ch - ly * sh, py + lx * sh + ly * ch

    tire = pgfx.Color(25, 25, 28)
    body = pgfx.Color(226, 60, 54)
    for lx, ly in ((10, -9), (10, 9), (-11, -9), (-11, 9)):
        wx, wy = at(lx, ly)
        pgfx.line(wx - ch * 5, wy - sh * 5, wx + ch * 5, wy + sh * 5, tire, z=1, width=5)
    tail, nose = at(-16, 0), at(12, 0)
    pgfx.line(tail[0], tail[1], nose[0], nose[1], body, z=2, width=16)
    pgfx.circle_fill(nose[0], nose[1], 8, body, z=2)
    wl, wr = at(4, -5), at(4, 5)
    pgfx.line(wl[0], wl[1], wr[0], wr[1], pgfx.Color(96, 126, 150), z=3, width=4)


def render():
    pgfx.clear(GRASS)

    pgfx.set_view(cam_x, cam_y, zoom=zoom, rot=cam_rot)
    draw_world()

    pgfx.reset_view()  # HUD below is in screen space
    if font:
        pgfx.text(font, f"POS {int(px)}, {int(py)}   ZOOM {zoom:.2f}", 12, 10, HUD, z=10)
        hint = "ARROWS/WASD DRIVE   Q/E ZOOM   HOLD R FOR CHASE CAM"
        pgfx.text(font, hint, 12, SCREEN_H - 30, HUD_DIM, z=10)
        pgfx.text(font, f"{pgfx.fps()} FPS", SCREEN_W - 70, 10, HUD_DIM, z=10)


pgfx.run(update, render, on_ready=on_ready)
