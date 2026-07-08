"""Rotated rectangles and non-uniform sprite scaling render without errors."""

import os

TEST_PNG = os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", "..", "examples", "assets", "test.png")
)

PRIMS = """
import math

import pgfx

pgfx.init(320, 240, "prims", fps_limit=0)
state = {}

def on_ready():
    state["spr"] = pgfx.sprite_load(@PNG@)
    pgfx.sprite_set_origin(state["spr"], 8, 8)

frames = [0]

def update(dt):
    frames[0] += 1
    return frames[0] < 8

def render():
    pgfx.clear(pgfx.BLACK)
    if "spr" not in state:
        return
    t = frames[0] / 8
    # rotated rects at various angles, plain and under a camera
    for i in range(8):
        pgfx.rect_fill_ex(40 + i * 30, 60, 24, 10, pgfx.RED, rot=i * math.pi / 8)
    pgfx.rect_fill_ex(160, 120, 60, 30, pgfx.GREEN, rot=t * math.pi, z=1)
    pgfx.set_view(160, 120, zoom=1.5, rot=0.3)
    pgfx.rect_fill_ex(160, 180, 40, 20, pgfx.YELLOW, rot=-t)
    # non-uniform sprite scaling, incl. through the view
    pgfx.draw_ex(state["spr"], 80, 180, scale=2, scale_y=0.5)
    pgfx.reset_view()
    pgfx.draw_ex(state["spr"], 240, 180, rot=t, scale=1, scale_y=3, z=2)

pgfx.run(update, render, on_ready=on_ready)
print("PRIMS OK")
"""


def test_rect_fill_ex_and_scale_y(run_script):
    p = run_script(PRIMS.replace("@PNG@", repr(TEST_PNG)))
    assert p.returncode == 0, p.stderr
    assert "PRIMS OK" in p.stdout
