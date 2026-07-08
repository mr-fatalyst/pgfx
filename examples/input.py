"""Polling input: held keys, single presses, mouse buttons and wheel."""

import os

import pgfx

SCREEN_W, SCREEN_H = 800, 600
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx input")

font = None
space_presses = 0
wheel_total = 0.0


def on_ready():
    global font
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 20)


def update(dt):
    global space_presses, wheel_total
    if pgfx.key_pressed(pgfx.KEY_SPACE):  # fires once per press
        space_presses += 1
    wheel_total += pgfx.mouse_wheel()
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))
    if not font:
        return

    held = pgfx.key_down(pgfx.KEY_SPACE)  # true every frame while held
    pgfx.text(font, f"SPACE held: {'yes' if held else 'no'}", 40, 60, pgfx.WHITE)
    pgfx.text(font, f"SPACE presses: {space_presses}", 40, 90, pgfx.WHITE)

    mx, my = pgfx.mouse_pos()
    buttons = [
        name
        for name, btn in (("L", pgfx.MOUSE_LEFT), ("R", pgfx.MOUSE_RIGHT), ("M", pgfx.MOUSE_MIDDLE))
        if pgfx.mouse_down(btn)
    ]
    pgfx.text(font, f"mouse: {mx}, {my}   buttons: {' '.join(buttons) or '-'}", 40, 140, pgfx.WHITE)
    pgfx.text(font, f"wheel total: {wheel_total:+.1f}", 40, 170, pgfx.WHITE)

    # crosshair under the cursor
    pgfx.line(mx - 12, my, mx + 12, my, pgfx.YELLOW)
    pgfx.line(mx, my - 12, mx, my + 12, pgfx.YELLOW)

    hint = "HOLD SPACE, CLICK, SCROLL — ESC TO QUIT"
    pgfx.text(font, hint, SCREEN_W / 2, SCREEN_H - 34, DIM, align="center")


pgfx.run(update, render, on_ready=on_ready)
