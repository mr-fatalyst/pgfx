"""A tiny complete game: move the square, collect coins, watch the score.

The smallest example with the full game-loop shape: input, state, simple
collision and a HUD — a template to start a game from.
"""

import os
import random

import pgfx

SCREEN_W, SCREEN_H = 800, 600
PLAYER_SIZE = 36
COIN_R = 12
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx demo")

font = None
px, py = SCREEN_W / 2, SCREEN_H / 2
coin_x, coin_y = 200.0, 200.0
score = 0


def place_coin():
    global coin_x, coin_y
    coin_x = random.uniform(40, SCREEN_W - 40)
    coin_y = random.uniform(40, SCREEN_H - 40)


def on_ready():
    global font
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 20)
    place_coin()


def update(dt):
    global px, py, score

    speed = 260 * dt
    if pgfx.key_down(pgfx.KEY_LEFT) or pgfx.key_down(pgfx.KEY_A):
        px -= speed
    if pgfx.key_down(pgfx.KEY_RIGHT) or pgfx.key_down(pgfx.KEY_D):
        px += speed
    if pgfx.key_down(pgfx.KEY_UP) or pgfx.key_down(pgfx.KEY_W):
        py -= speed
    if pgfx.key_down(pgfx.KEY_DOWN) or pgfx.key_down(pgfx.KEY_S):
        py += speed
    px = max(PLAYER_SIZE / 2, min(SCREEN_W - PLAYER_SIZE / 2, px))
    py = max(PLAYER_SIZE / 2, min(SCREEN_H - PLAYER_SIZE / 2, py))

    half = PLAYER_SIZE / 2
    if pgfx.collide_circle_rect(
        coin_x, coin_y, COIN_R, px - half, py - half, PLAYER_SIZE, PLAYER_SIZE
    ):
        score += 1
        place_coin()

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))
    if not font:
        return

    pgfx.circle_fill(coin_x, coin_y, COIN_R, pgfx.Color(240, 200, 60))
    pgfx.circle_fill(coin_x - 3, coin_y - 3, COIN_R / 3, pgfx.Color(255, 235, 150))

    half = PLAYER_SIZE / 2
    pgfx.rect_fill(px - half, py - half, PLAYER_SIZE, PLAYER_SIZE, pgfx.Color(90, 200, 110))

    pgfx.text(font, f"SCORE {score}", 14, 10, pgfx.WHITE)
    pgfx.text(font, "WASD/ARROWS — ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center")


pgfx.run(update, render, on_ready=on_ready)
