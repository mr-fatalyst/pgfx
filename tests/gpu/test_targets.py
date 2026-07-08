"""Render targets: draw into a texture, use it as a sprite, error cases."""

TARGETS = """
import math

import pgfx

pgfx.init(320, 240, "targets", fps_limit=0)
state = {}

def on_ready():
    state["t"] = pgfx.target_create(64, 64)
    state["spr"] = pgfx.target_sprite(state["t"])
    assert pgfx.target_size(state["t"]) == (64, 64)
    state["ps"] = pgfx.particles_create(primitive="square", emission_rate=100)
    pgfx.particles_fire(state["ps"], 32, 32)

frames = [0]

def update(dt):
    frames[0] += 1
    if "ps" in state:
        pgfx.particles_update(state["ps"], 0.016)
    return frames[0] < 10

def render():
    pgfx.clear(pgfx.Color(20, 20, 30))
    if "t" not in state:
        return
    t = frames[0] / 10

    # draw a little scene into the target: primitives, camera, particles
    pgfx.render_to(state["t"])
    pgfx.clear(pgfx.Color(60, 20, 20))
    pgfx.set_view(32, 32, zoom=1 + t)   # centers on the target, not the screen
    pgfx.circle_fill(32, 32, 16, pgfx.GREEN)
    pgfx.rect_fill_ex(32, 32, 40, 8, pgfx.YELLOW, rot=t * math.pi)
    pgfx.particles_render(state["ps"])
    pgfx.render_to(None)

    # same frame: the freshly drawn target is shown on screen, transformed
    pgfx.draw(state["spr"], 20, 20)
    pgfx.draw_ex(state["spr"], 200, 120, rot=t, scale=1.5, z=1)
    pgfx.draw_ex(state["spr"], 120, 180, scale=0.5, alpha=0.7, blend="add", z=2)

pgfx.run(update, render, on_ready=on_ready)
print("TARGETS OK")
"""


FEEDBACK = """
import pgfx

pgfx.init(320, 240, "feedback", fps_limit=0)
state = {}

def on_ready():
    state["t"] = pgfx.target_create(64, 64)
    state["spr"] = pgfx.target_sprite(state["t"])

def update(dt):
    return "done" not in state

def render():
    pgfx.clear(pgfx.BLACK)
    if "t" not in state:
        return
    try:
        pgfx.render_to(state["t"])
        pgfx.draw(state["spr"], 0, 0)  # a target into itself
        pgfx.render_to(None)
    finally:
        state["done"] = True

pgfx.run(update, render, on_ready=on_ready)
print("never reached")
"""


def test_render_targets(run_script):
    p = run_script(TARGETS)
    assert p.returncode == 0, p.stderr
    assert "TARGETS OK" in p.stdout


def test_target_feedback_loop_is_an_error(run_script):
    p = run_script(FEEDBACK)
    assert p.returncode != 0
    assert "into itself" in p.stderr
