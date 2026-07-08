"""Additive blend mode for particles and sprites."""

import os

TEST_PNG = os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", "..", "examples", "assets", "test.png")
)

BLEND = """
import pgfx

pgfx.init(320, 240, "blend", fps_limit=0)
state = {}

def on_ready():
    state["spr"] = pgfx.sprite_load(@PNG@)
    state["alpha"] = pgfx.particles_create(primitive="circle_soft", emission_rate=200)
    state["add"] = pgfx.particles_create(primitive="square", emission_rate=200, blend="add")
    pgfx.particles_fire(state["alpha"], 100, 120)
    pgfx.particles_fire(state["add"], 220, 120)
    # blend can be flipped live through particles_set (merge keeps the rest)
    pgfx.particles_set(state["alpha"], blend="add")
    pgfx.particles_set(state["alpha"], blend="alpha")

frames = [0]

def update(dt):
    frames[0] += 1
    if "alpha" in state:
        pgfx.particles_update(state["alpha"], 0.016)
        pgfx.particles_update(state["add"], 0.016)
    return frames[0] < 8

def render():
    pgfx.clear(pgfx.BLACK)
    if "spr" not in state:
        return
    # interleave blends and textures so batches split and pipelines switch
    pgfx.particles_render(state["alpha"])
    pgfx.particles_render(state["add"])
    pgfx.draw_ex(state["spr"], 40, 40, blend="add")
    pgfx.draw_ex(state["spr"], 80, 40)
    pgfx.rect_fill(120, 30, 30, 20, pgfx.RED)
    pgfx.draw_ex(state["spr"], 160, 40, blend="add", z=1)

pgfx.run(update, render, on_ready=on_ready)
print("BLEND OK")
"""


def test_additive_blending_renders(run_script):
    p = run_script(BLEND.replace("@PNG@", repr(TEST_PNG)))
    assert p.returncode == 0, p.stderr
    assert "BLEND OK" in p.stdout
