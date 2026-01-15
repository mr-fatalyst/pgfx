"""Z-order test - demonstrates layer sorting for sprites."""

import os

import pgfx

pgfx.init(800, 600, "Z-Order Test - Sprites")

spr1 = None
spr2 = None
spr3 = None
font = None


def on_ready():
    global spr1, spr2, spr3, font
    # Load same sprite 3 times (in real game would be different sprites)
    spr1 = pgfx.sprite_load(os.path.join(os.path.dirname(__file__), "assets/ball.png"))
    spr2 = pgfx.sprite_load(os.path.join(os.path.dirname(__file__), "assets/ball.png"))
    spr3 = pgfx.sprite_load(os.path.join(os.path.dirname(__file__), "assets/ball.png"))
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 20)


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(30, 30, 50))

    if not spr1:
        return

    # Draw in WRONG order in code, but z fixes it
    # Code order: spr1 first, spr3 last
    # Expected visual: spr1 (z=2) on top, spr3 (z=0) at back

    # First in code, but z=2 - should be ON TOP
    pgfx.draw_ex(spr1, 300, 250, z=2, scale=3)

    # Second in code, z=1 - should be MIDDLE
    pgfx.draw_ex(spr2, 340, 290, z=1, scale=3)

    # Last in code, but z=0 - should be at BACK
    pgfx.draw_ex(spr3, 380, 330, z=0, scale=3)

    # Without z-order: spr3 would be on top (drawn last)
    # With z-order: spr1 on top (highest z=2)

    if font:
        pgfx.text(font, "Z-Order Test", 20, 20, pgfx.WHITE)
        pgfx.text(font, "Top-left sprite: z=2 (drawn FIRST in code)", 20, 60, pgfx.YELLOW)
        pgfx.text(font, "Middle sprite: z=1", 20, 90, pgfx.WHITE)
        pgfx.text(font, "Bottom-right sprite: z=0 (drawn LAST in code)", 20, 120, pgfx.WHITE)
        pgfx.text(font, "", 20, 160, pgfx.WHITE)
        pgfx.text(font, "If z-order works: top-left sprite is visible on top", 20, 180, pgfx.GREEN)
        pgfx.text(font, "If broken: bottom-right sprite would be on top", 20, 210, pgfx.RED)
        pgfx.text(font, "ESC to exit", 20, 550, pgfx.Color(100, 100, 100))


pgfx.run(update, render, on_ready=on_ready)
