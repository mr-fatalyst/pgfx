"""Day/night cycle: a keyframe-interpolated sky over a mountain valley.

All the timing lives in one table: each keyframe is (hour, sky top,
horizon, night factor), and everything else — sun, moon, stars, mountain
and ground shading — derives from the sampled values.

Controls:
    Right / Left    hold to fast-forward / rewind (time also flows by itself)
    Esc             quit
"""

import math
import os

import pgfx

SCREEN_W, SCREEN_H = 1280, 720
GROUND_Y = SCREEN_H - 150
DIM = pgfx.Color(150, 155, 165)

# hour -> (sky top RGB, horizon RGB, night factor 0..1)
KEYFRAMES = [
    (0.0, (10, 10, 40), (20, 20, 60), 1.0),
    (5.0, (10, 10, 40), (20, 20, 60), 1.0),
    (6.5, (255, 150, 100), (255, 200, 150), 0.5),
    (8.0, (135, 206, 235), (200, 230, 255), 0.0),
    (12.0, (100, 180, 255), (210, 235, 255), 0.0),
    (17.0, (135, 206, 235), (200, 230, 255), 0.0),
    (19.0, (255, 130, 80), (255, 180, 120), 0.45),
    (20.5, (80, 60, 120), (100, 80, 140), 0.8),
    (22.0, (10, 10, 40), (20, 20, 60), 1.0),
    (24.0, (10, 10, 40), (20, 20, 60), 1.0),
]

RIDGE_BACK = [
    (0, 480),
    (100, 430),
    (200, 460),
    (350, 410),
    (500, 450),
    (650, 400),
    (800, 440),
    (950, 420),
    (1100, 460),
    (1280, 480),
]
RIDGE_FRONT = [
    (0, 500),
    (150, 440),
    (300, 480),
    (450, 420),
    (600, 470),
    (750, 400),
    (900, 450),
    (1050, 430),
    (1200, 470),
    (1280, 500),
]
STARS = [(x * 137 % SCREEN_W, (x * 89) % 220 + 20) for x in range(40)]
CLOUDS = [(100, 80, 40), (400, 120, 30), (700, 60, 50), (1000, 140, 35), (200, 180, 32)]

pgfx.init(SCREEN_W, SCREEN_H, "pgfx day/night")

font = None
hour = 12.0
cloud_shift = 0.0


def lerp(a, b, t):
    return a + (b - a) * t


def lerp_rgb(c1, c2, t):
    return tuple(int(lerp(a, b, t)) for a, b in zip(c1, c2))


def sample(h):
    """Piecewise-linear interpolation over KEYFRAMES at hour h."""
    for (h0, top0, bot0, n0), (h1, top1, bot1, n1) in zip(KEYFRAMES, KEYFRAMES[1:]):
        if h0 <= h <= h1:
            t = (h - h0) / (h1 - h0) if h1 > h0 else 0.0
            return lerp_rgb(top0, top1, t), lerp_rgb(bot0, bot1, t), lerp(n0, n1, t)
    return KEYFRAMES[-1][1], KEYFRAMES[-1][2], KEYFRAMES[-1][3]


def darken(rgb, night, strength=0.6):
    f = 1 - night * strength
    return pgfx.Color(int(rgb[0] * f), int(rgb[1] * f), int(rgb[2] * f))


def draw_sky(top, bottom):
    strips = 20
    strip_h = GROUND_Y // strips + 1
    for i in range(strips):
        color = lerp_rgb(top, bottom, i / (strips - 1))
        pgfx.rect_fill(0, i * (GROUND_Y // strips), SCREEN_W, strip_h, pgfx.Color(*color))


def draw_ridge(points, ground_y, color):
    """Fill a mountain polyline down to ground_y with vertical strips."""
    for (x1, y1), (x2, y2) in zip(points, points[1:]):
        for x in range(int(x1), int(x2), 4):
            y = lerp(y1, y2, (x - x1) / (x2 - x1))
            pgfx.rect_fill(x, y, 4, ground_y - y, color)


def draw_sun_and_moon(night):
    # both travel a circle around the valley; the moon is opposite the sun
    angle = (hour - 12) / 24 * 2 * math.pi + math.pi / 2
    cx, cy, radius = SCREEN_W / 2, GROUND_Y, SCREEN_H - 200

    sun_alpha = int(255 * (1 - night))
    if sun_alpha > 10:
        x = cx + math.cos(angle) * radius
        y = cy - math.sin(angle) * radius
        color = lerp_rgb((255, 240, 150), (255, 120, 50), night)
        pgfx.circle_fill(x, y, 75, pgfx.Color(*color, sun_alpha // 3))
        pgfx.circle_fill(x, y, 40, pgfx.Color(*color, sun_alpha))

    moon_alpha = int(255 * night)
    if moon_alpha > 10:
        x = cx + math.cos(angle + math.pi) * radius
        y = cy - math.sin(angle + math.pi) * radius
        pgfx.circle_fill(x, y, 50, pgfx.Color(200, 200, 255, moon_alpha // 4))
        pgfx.circle_fill(x, y, 35, pgfx.Color(240, 240, 255, moon_alpha))
        pgfx.circle_fill(x - 10, y - 8, 8, pgfx.Color(200, 200, 220, moon_alpha))
        pgfx.circle_fill(x + 12, y + 5, 6, pgfx.Color(210, 210, 230, moon_alpha))


def draw_clouds(night):
    alpha = int(lerp(200, 90, night))
    for base_x, y, size in CLOUDS:
        x = (base_x + cloud_shift) % (SCREEN_W + 300) - 150
        color = pgfx.Color(255, 255, 255, alpha)
        pgfx.circle_fill(x, y, size * 0.8, color)
        pgfx.circle_fill(x - size * 0.8, y + size * 0.15, size * 0.55, color)
        pgfx.circle_fill(x + size * 0.8, y + size * 0.15, size * 0.65, color)


def draw_ground(night):
    pgfx.rect_fill(0, GROUND_Y, SCREEN_W, 50, darken((50, 120, 50), night))
    pgfx.rect_fill(0, GROUND_Y + 50, SCREEN_W, 100, darken((80, 50, 30), night))
    tuft = darken((60, 140, 60), night)
    for x in range(0, SCREEN_W, 20):
        h = 10 + (x * 7) % 15
        pgfx.rect_fill(x, GROUND_Y - h, 8, h + 5, tuft)


def update(dt):
    global hour, cloud_shift
    speed = 1 / 6  # game-hours per real second when idle
    if pgfx.key_down(pgfx.KEY_RIGHT):
        speed = 3.0
    elif pgfx.key_down(pgfx.KEY_LEFT):
        speed = -3.0
    hour = (hour + speed * dt) % 24
    cloud_shift += 12 * dt
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def on_ready():
    global font
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 20)


def render():
    top, bottom, night = sample(hour)

    draw_sky(top, bottom)

    star_alpha = int(255 * night)
    if star_alpha > 10:
        for x, y in STARS:
            pgfx.circle_fill(x, y, 2, pgfx.Color(255, 255, 255, star_alpha))

    draw_sun_and_moon(night)
    draw_clouds(night)
    draw_ridge(RIDGE_BACK, GROUND_Y, darken((80, 90, 120), night))
    draw_ridge(RIDGE_FRONT, SCREEN_H, darken((60, 70, 90), night))
    draw_ground(night)

    if font:
        pgfx.rect_fill(8, 8, 170, 62, pgfx.Color(0, 0, 0, 180))
        pgfx.text(font, f"{int(hour):02d}:{int(hour * 60) % 60:02d}", 18, 16, pgfx.WHITE)
        pgfx.text(font, f"FPS: {pgfx.fps()}", 18, 42, pgfx.WHITE)
        pgfx.text(
            font,
            "HOLD LEFT/RIGHT TO SCRUB TIME — ESC TO QUIT",
            SCREEN_W / 2,
            SCREEN_H - 34,
            DIM,
            align="center",
        )


pgfx.run(update, render, on_ready=on_ready)
