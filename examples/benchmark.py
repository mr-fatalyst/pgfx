"""Sprite throughput benchmark: bouncing balls, add more until FPS drops.

Controls:
    Up / Down       +/- 100 sprites
    Right / Left    +/- 1000 sprites
    Esc             quit
"""

import os
import random

import pgfx

SCREEN_W, SCREEN_H = 1280, 720
BALL = 32  # sprite size, px
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx benchmark")

font = sprite = None
balls = []  # [x, y, vx, vy]


def add_balls(count):
    for _ in range(count):
        balls.append([
            random.uniform(50, SCREEN_W - 50),
            random.uniform(50, SCREEN_H - 50),
            random.choice((-1, 1)) * random.uniform(60, 220),
            random.choice((-1, 1)) * random.uniform(60, 220),
        ])


def on_ready():
    global font, sprite
    here = os.path.dirname(__file__)
    font = pgfx.font_load(os.path.join(here, "assets/font.ttf"), 20)
    sprite = pgfx.sprite_load(os.path.join(here, "assets/ball.png"))
    add_balls(100)


def update(dt):
    if pgfx.key_pressed(pgfx.KEY_UP):
        add_balls(100)
    if pgfx.key_pressed(pgfx.KEY_DOWN):
        del balls[-100:]
    if pgfx.key_pressed(pgfx.KEY_RIGHT):
        add_balls(1000)
    if pgfx.key_pressed(pgfx.KEY_LEFT):
        del balls[-1000:]

    for b in balls:
        b[0] += b[2] * dt
        b[1] += b[3] * dt
        if b[0] < 0 or b[0] > SCREEN_W - BALL:
            b[2] = -b[2]
            b[0] = max(0, min(SCREEN_W - BALL, b[0]))
        if b[1] < 0 or b[1] > SCREEN_H - BALL:
            b[3] = -b[3]
            b[1] = max(0, min(SCREEN_H - BALL, b[1]))

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(20, 20, 30))
    if not font:
        return

    for b in balls:
        pgfx.draw(sprite, b[0], b[1])

    pgfx.rect_fill(8, 8, 210, 62, pgfx.Color(0, 0, 0, 180), z=1)
    pgfx.text(font, f"FPS: {pgfx.fps()}", 18, 16, pgfx.WHITE, z=2)
    pgfx.text(font, f"Sprites: {len(balls)}", 18, 42, pgfx.WHITE, z=2)
    pgfx.text(font, "UP/DOWN +-100   RIGHT/LEFT +-1000   ESC QUIT",
              SCREEN_W / 2, SCREEN_H - 34, DIM, z=2, align="center")


pgfx.run(update, render, on_ready=on_ready)
