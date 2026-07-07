"""Text metrics demo: text_size() and the align= parameter of text().

Four things on screen:
  - centered multiline title (align="center" — each line centers on x)
  - a speech bubble auto-sized around its text with text_size()
  - a typewriter line with a blinking caret placed via text_size()
  - a right-aligned scoreboard column (align="right")
"""

import math
import os

import pgfx

SCREEN_W, SCREEN_H = 960, 600

BG = pgfx.Color(24, 26, 34)
PANEL = pgfx.Color(40, 44, 58)
ACCENT = pgfx.Color(255, 210, 70)
DIM = pgfx.Color(150, 155, 165)

PHRASES = [
    "Hi!",
    "text_size() measures me",
    "so this bubble always fits,",
    "short or really, really long lines alike.",
]
TYPED = "Typing with a caret that follows text_size()..."

pgfx.init(SCREEN_W, SCREEN_H, "pgfx text metrics")

font = font_big = None


def on_ready():
    global font, font_big
    path = os.path.join(os.path.dirname(__file__), "assets/font.ttf")
    font = pgfx.font_load(path, 20)
    font_big = pgfx.font_load(path, 44)


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def draw_bubble(cx, cy, phrase):
    """A speech bubble sized to its text: the whole point of text_size()."""
    w, h = pgfx.text_size(font, phrase)
    pad = 12
    bx, by = cx - w / 2 - pad, cy - h - 2 * pad - 18
    pgfx.rect_fill(bx + 3, by + 4, w + 2 * pad, h + 2 * pad, pgfx.Color(0, 0, 0, 80), z=4)
    pgfx.rect_fill(bx, by, w + 2 * pad, h + 2 * pad, pgfx.WHITE, z=5)
    pgfx.line(cx - 6, by + h + 2 * pad, cx, cy - 4, pgfx.WHITE, z=5, width=10)
    pgfx.text(font, phrase, cx, by + pad, pgfx.Color(30, 30, 40), z=6, align="center")


def render():
    pgfx.clear(BG)
    if not font:
        return
    t = pgfx.time()

    # 1. centered multiline block: one call, both lines center on SCREEN_W/2
    pgfx.text(
        font_big,
        "TEXT METRICS\nfinally measurable",
        SCREEN_W / 2,
        40,
        pgfx.WHITE,
        align="center",
    )

    # 2. bouncing character with an auto-sized speech bubble
    cx = SCREEN_W / 2 + math.sin(t * 0.7) * 260
    cy = 380 + abs(math.sin(t * 3)) * -40
    pgfx.circle_fill(cx + 3, 384, 24, pgfx.Color(0, 0, 0, 80))
    pgfx.circle_fill(cx, cy, 24, ACCENT)
    pgfx.circle_fill(cx - 8, cy - 6, 3.5, pgfx.Color(30, 30, 40))
    pgfx.circle_fill(cx + 8, cy - 6, 3.5, pgfx.Color(30, 30, 40))
    draw_bubble(cx, cy - 24, PHRASES[int(t / 2.5) % len(PHRASES)])

    # 3. typewriter: the caret sits exactly after the typed prefix
    shown = TYPED[: int(t * 12) % (len(TYPED) + 20)]  # types, then holds
    tx, ty = 60, 480
    pgfx.rect_fill(tx - 16, ty - 12, SCREEN_W - 2 * (tx - 16), 46, PANEL)
    pgfx.text(font, shown, tx, ty, pgfx.Color(120, 220, 130))
    if int(t * 2.5) % 2 == 0:
        w, h = pgfx.text_size(font, shown)
        pgfx.rect_fill(tx + w + 2, ty, 10, h, pgfx.Color(120, 220, 130))

    # 4. right-aligned column: numbers anchor to the right edge
    rx = SCREEN_W - 40
    pgfx.text(font, "SCORE", rx, 100, DIM, align="right")
    for i, points in enumerate((125_900, 8_420, 310)):
        pgfx.text(font, f"{points:,}", rx, 130 + i * 26, pgfx.WHITE, align="right")

    pgfx.text(font, "ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 36, DIM, align="center")


pgfx.run(update, render, on_ready=on_ready)
