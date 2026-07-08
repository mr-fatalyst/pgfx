"""Rotated rectangles and non-uniform sprite scaling.

rect_fill_ex(x, y, w, h, color, rot) draws a rectangle CENTERED on (x, y) —
same anchor rule as circle_fill — rotated around that center: here a working
clock and a ring of spinning cards. draw_ex(..., scale, scale_y) squashes
and stretches the bouncing ball.
"""

import math
import os
import time as systime

import pgfx

SCREEN_W, SCREEN_H = 960, 600
BG = pgfx.Color(24, 26, 34)
FACE = pgfx.Color(238, 235, 228)
DARK = pgfx.Color(40, 42, 52)
ACCENT = pgfx.Color(226, 60, 54)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx rotated rects")

font = None
ball = None
ball_h = 0


def on_ready():
    global font, ball, ball_h
    here = os.path.dirname(__file__)
    font = pgfx.font_load(os.path.join(here, "assets/font.ttf"), 17)
    ball = pgfx.sprite_load(os.path.join(here, "assets/ball.png"))
    bw, ball_h = pgfx.sprite_rect(ball, 0, 0)[2:]
    # anchor at bottom-center so the ball squashes into the floor
    pgfx.sprite_set_origin(ball, bw / 2, ball_h)


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def hand(cx, cy, angle, length, width, color, z):
    """A clock hand: a rect whose center sits halfway along the hand, so it
    rotates around the clock center."""
    hx = cx + math.cos(angle) * length / 2
    hy = cy + math.sin(angle) * length / 2
    pgfx.rect_fill_ex(hx, hy, length, width, color, rot=angle, z=z)


def draw_clock(cx, cy, r):
    pgfx.circle_fill(cx + 4, cy + 5, r, pgfx.Color(0, 0, 0, 70))
    pgfx.circle_fill(cx, cy, r, FACE)
    for i in range(12):
        a = i * math.pi / 6
        big = i % 3 == 0
        tick_r = r - (14 if big else 10)
        tx, ty = cx + math.cos(a) * tick_r, cy + math.sin(a) * tick_r
        pgfx.rect_fill_ex(tx, ty, 16 if big else 9, 4 if big else 2.5, DARK, rot=a, z=1)

    t = systime.localtime()
    sec = t.tm_sec + systime.time() % 1
    minute = t.tm_min + sec / 60
    hour = t.tm_hour % 12 + minute / 60
    half = math.pi / 2
    hand(cx, cy, hour * math.pi / 6 - half, r * 0.5, 7, DARK, z=2)
    hand(cx, cy, minute * math.pi / 30 - half, r * 0.75, 5, DARK, z=2)
    hand(cx, cy, sec * math.pi / 30 - half, r * 0.85, 2.5, ACCENT, z=3)
    pgfx.circle_fill(cx, cy, 5, ACCENT, z=4)


def draw_cards(cx, cy, t):
    for i in range(10):
        a = t * 0.6 + i * math.pi / 5
        x = cx + math.cos(a) * 130
        y = cy + math.sin(a) * 130
        hue = pgfx.Color(120 + i * 13, 200 - i * 12, 90 + i * 16)
        pgfx.rect_fill_ex(x, y, 46, 30, hue, rot=a + t * 2, z=1)


def draw_ball(t):
    floor_y = SCREEN_H - 60
    phase = abs(math.sin(t * 2.4))
    x = 480 + math.sin(t * 0.9) * 90
    y = floor_y - phase * 180
    on_ground = phase < 0.12
    # classic squash and stretch: wide on impact, narrow in flight
    squash = 0.55 + 0.45 * phase if on_ground else 1 - 0.18 * phase
    stretch = 2 - squash
    pgfx.line(320, floor_y, 640, floor_y, pgfx.Color(255, 255, 255, 40), width=3)
    pgfx.draw_ex(ball, x, y, scale=stretch, scale_y=squash, z=2)


def render():
    pgfx.clear(BG)
    if not font:
        return
    t = pgfx.time()
    draw_clock(230, 260, 150)
    draw_cards(710, 260, t)
    draw_ball(t)
    pgfx.text(
        font,
        "rect_fill_ex: clock + cards    draw_ex scale_y: the ball    ESC to quit",
        SCREEN_W / 2,
        SCREEN_H - 30,
        pgfx.Color(150, 155, 165),
        align="center",
    )


pgfx.run(update, render, on_ready=on_ready)
