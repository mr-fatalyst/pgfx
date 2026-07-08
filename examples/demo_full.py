"""Several features together: sprites, movement, collision and a HUD.

Drive the player between three bouncing enemies; anyone you touch lights
up red (collide_circles). WASD/arrows to move, ESC to quit.
"""

import math
import os

import pgfx

SCREEN_W, SCREEN_H = 800, 600
GRID = pgfx.Color(40, 40, 60)
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx full demo")

font = sprite = None
player_x, player_y = 400.0, 300.0
facing = 0.0

enemies = [
    {"x": 200.0, "y": 150.0, "vx": 50.0, "vy": 30.0},
    {"x": 600.0, "y": 400.0, "vx": -70.0, "vy": -40.0},
    {"x": 100.0, "y": 500.0, "vx": 60.0, "vy": -50.0},
]


def on_ready():
    global font, sprite
    here = os.path.dirname(__file__)
    font = pgfx.font_load(os.path.join(here, "assets/font.ttf"), 16)
    sprite = pgfx.sprite_load(os.path.join(here, "assets/test.png"))
    w, h = pgfx.sprite_rect(sprite, 0, 0)[2:]
    pgfx.sprite_set_origin(sprite, w / 2, h / 2)


def update(dt):
    global player_x, player_y, facing

    dx, dy = 0.0, 0.0
    if pgfx.key_down(pgfx.KEY_LEFT) or pgfx.key_down(pgfx.KEY_A):
        dx -= 1
    if pgfx.key_down(pgfx.KEY_RIGHT) or pgfx.key_down(pgfx.KEY_D):
        dx += 1
    if pgfx.key_down(pgfx.KEY_UP) or pgfx.key_down(pgfx.KEY_W):
        dy -= 1
    if pgfx.key_down(pgfx.KEY_DOWN) or pgfx.key_down(pgfx.KEY_S):
        dy += 1
    if dx or dy:
        length = math.hypot(dx, dy)
        player_x += dx / length * 220 * dt
        player_y += dy / length * 220 * dt
        facing = math.atan2(dy, dx)
    player_x = max(30, min(SCREEN_W - 30, player_x))
    player_y = max(30, min(SCREEN_H - 30, player_y))

    for e in enemies:
        e["x"] += e["vx"] * dt
        e["y"] += e["vy"] * dt
        if not 20 < e["x"] < SCREEN_W - 20:
            e["vx"] = -e["vx"]
            e["x"] = max(20, min(SCREEN_W - 20, e["x"]))
        if not 20 < e["y"] < SCREEN_H - 20:
            e["vy"] = -e["vy"]
            e["y"] = max(20, min(SCREEN_H - 20, e["y"]))

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(20, 20, 40))
    if not font:
        return

    for x in range(0, SCREEN_W + 1, 50):
        pgfx.line(x, 0, x, SCREEN_H, GRID)
    for y in range(0, SCREEN_H + 1, 50):
        pgfx.line(0, y, SCREEN_W, y, GRID)

    touching = 0
    for e in enemies:
        hit = pgfx.collide_circles(player_x, player_y, 20, e["x"], e["y"], 16)
        touching += hit
        pgfx.draw_ex(sprite, e["x"], e["y"], rot=pgfx.time() * 2, scale=0.8)
        if hit:
            pgfx.circle_fill(e["x"], e["y"], 24, pgfx.Color(255, 60, 50, 90), z=1)

    pgfx.draw_ex(sprite, player_x, player_y, rot=facing, z=2)

    pgfx.rect_fill(8, 8, 230, 66, pgfx.Color(0, 0, 0, 200), z=3)
    pgfx.text(font, f"FPS: {pgfx.fps()}   time: {pgfx.time():.0f}s", 18, 16, pgfx.WHITE, z=4)
    status = f"touching: {touching}" if touching else "touching: none"
    pgfx.text(font, status, 18, 44, pgfx.CYAN if touching else pgfx.WHITE, z=4)
    pgfx.text(font, "WASD/ARROWS — ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 30, DIM, z=4,
              align="center")


pgfx.run(update, render, on_ready=on_ready)
