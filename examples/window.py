"""The smallest pgfx program: a window, a clear color, an exit key."""

import pgfx

pgfx.init(800, 600, "pgfx window")


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))


pgfx.run(update, render)
