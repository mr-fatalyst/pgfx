"""Comprehensive pgfx demo - showcasing multiple features."""

import math
import os

import pgfx

pgfx.init(800, 600, "pgfx Full Demo")

sprite = None
font = None

player_x = 400.0
player_y = 300.0
player_speed = 200.0
rotation = 0.0

enemies = [
    {"x": 200.0, "y": 150.0, "vx": 50.0, "vy": 30.0},
    {"x": 600.0, "y": 400.0, "vx": -70.0, "vy": -40.0},
    {"x": 100.0, "y": 500.0, "vx": 60.0, "vy": -50.0},
]


def on_ready():
    global sprite, font
    try:
        sprite = pgfx.sprite_load(os.path.join(os.path.dirname(__file__), "assets/test.png"))
        pgfx.sprite_set_origin(sprite, 16, 16)
    except Exception:
        print("Could not load sprite - using primitives instead")

    try:
        font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 16)
    except Exception:
        print("Could not load font - text will be skipped")


def update(dt):
    global player_x, player_y, rotation

    dx, dy = 0.0, 0.0
    if pgfx.key_down(pgfx.KEY_LEFT) or pgfx.key_down(pgfx.KEY_A):
        dx -= 1.0
    if pgfx.key_down(pgfx.KEY_RIGHT) or pgfx.key_down(pgfx.KEY_D):
        dx += 1.0
    if pgfx.key_down(pgfx.KEY_UP) or pgfx.key_down(pgfx.KEY_W):
        dy -= 1.0
    if pgfx.key_down(pgfx.KEY_DOWN) or pgfx.key_down(pgfx.KEY_S):
        dy += 1.0

    if dx != 0.0 and dy != 0.0:
        dx *= 0.707
        dy *= 0.707

    player_x += dx * player_speed * dt
    player_y += dy * player_speed * dt

    if dx != 0.0 or dy != 0.0:
        rotation = math.atan2(dy, dx)

    player_x = max(30, min(770, player_x))
    player_y = max(30, min(570, player_y))

    for enemy in enemies:
        enemy["x"] += enemy["vx"] * dt
        enemy["y"] += enemy["vy"] * dt

        if enemy["x"] < 20 or enemy["x"] > 780:
            enemy["vx"] = -enemy["vx"]
            enemy["x"] = max(20, min(780, enemy["x"]))
        if enemy["y"] < 20 or enemy["y"] > 580:
            enemy["vy"] = -enemy["vy"]
            enemy["y"] = max(20, min(580, enemy["y"]))

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(20, 20, 40))

    for x in range(0, 800, 50):
        pgfx.line(x, 0, x, 600, pgfx.Color(40, 40, 60))
    for y in range(0, 600, 50):
        pgfx.line(0, y, 800, y, pgfx.Color(40, 40, 60))

    for enemy in enemies:
        if sprite:
            pgfx.draw_ex(sprite, enemy["x"], enemy["y"], rot=pgfx.time() * 2, scale=0.8)
        else:
            pgfx.circle_fill(enemy["x"], enemy["y"], 15, pgfx.RED)
            pgfx.circle_fill(enemy["x"] - 5, enemy["y"] - 3, 3, pgfx.YELLOW)
            pgfx.circle_fill(enemy["x"] + 5, enemy["y"] - 3, 3, pgfx.YELLOW)

    if sprite:
        pgfx.draw_ex(sprite, player_x, player_y, rot=rotation, scale=1.0)
    else:
        pgfx.circle_fill(player_x, player_y, 20, pgfx.GREEN)
        dx = math.cos(rotation) * 25
        dy = math.sin(rotation) * 25
        pgfx.circle_fill(player_x + dx, player_y + dy, 5, pgfx.YELLOW)

    pgfx.line(0, 0, 800, 0, pgfx.CYAN)
    pgfx.line(800, 0, 800, 600, pgfx.CYAN)
    pgfx.line(800, 600, 0, 600, pgfx.CYAN)
    pgfx.line(0, 600, 0, 0, pgfx.CYAN)

    if pgfx.key_down(pgfx.KEY_SPACE):
        pgfx.circle_fill(player_x, player_y, 20, pgfx.Color(0, 255, 0, 50))
        for enemy in enemies:
            pgfx.circle_fill(enemy["x"], enemy["y"], 15, pgfx.Color(255, 0, 0, 50))

    pgfx.rect_fill(5, 5, 220, 85, pgfx.Color(0, 0, 0, 200))

    if font:
        pgfx.text(font, f"FPS: {pgfx.fps()}", 10, 10, pgfx.WHITE)
        pgfx.text(font, f"Time: {pgfx.time():.1f}s", 10, 30, pgfx.WHITE)
        pgfx.text(font, f"Pos: ({player_x:.0f}, {player_y:.0f})", 10, 50, pgfx.CYAN)
        pgfx.text(font, "WASD/Arrows: Move | ESC: Quit", 10, 70, pgfx.YELLOW)
        pgfx.text(font, "SPACE: Show collision", 10, 570, pgfx.MAGENTA)


print("Controls: WASD/Arrows - Move | SPACE - Collision | ESC - Quit")
pgfx.run(update, render, on_ready=on_ready)
