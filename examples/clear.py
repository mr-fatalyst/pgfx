"""clear() sets the frame's background color; the last call in a frame wins.

The background cycles through a small palette once a second. ESC to quit.
"""

import pgfx

PALETTE = [
    pgfx.Color(170, 50, 40),
    pgfx.Color(40, 130, 60),
    pgfx.Color(40, 70, 160),
    pgfx.Color(120, 50, 140),
]

pgfx.init(800, 600, "pgfx clear")


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(PALETTE[int(pgfx.time()) % len(PALETTE)])


pgfx.run(update, render)
