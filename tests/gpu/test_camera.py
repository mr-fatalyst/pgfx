"""View transform (camera): set_view/reset_view across draw kinds."""

CAMERA = """
import math
import os

import pgfx

pgfx.init(320, 240, "camera", fps_limit=0)
state = {}

def on_ready():
    state["font"] = pgfx.font_load(@FONT@, 14)
    state["ps"] = pgfx.particles_create(primitive="square", emission_rate=100)
    state["light"] = pgfx.light_create(80, pgfx.Color(255, 200, 100))
    pgfx.particles_fire(state["ps"], 160, 120)

frames = [0]

def update(dt):
    frames[0] += 1
    if "ps" in state:
        pgfx.particles_update(state["ps"], 0.016)
    return frames[0] < 12

def render():
    pgfx.clear(pgfx.Color(20, 20, 30))
    if not state:
        return
    t = frames[0] / 12
    # camera pans, zooms and rotates over the frames; every draw kind goes
    # through the view: primitives, sprites-as-lines, text, particles, light
    pgfx.set_view(160 + 40 * t, 120, zoom=0.5 + t, rot=t * math.pi / 4)
    pgfx.rect_fill(100, 100, 80, 40, pgfx.RED)
    pgfx.circle_fill(160, 120, 20, pgfx.GREEN)
    pgfx.line(0, 0, 320, 240, pgfx.WHITE, width=3)
    pgfx.text(state["font"], "world", 120, 60, pgfx.WHITE)
    pgfx.particles_render(state["ps"])
    pgfx.light_draw(state["light"], 160, 120)
    pgfx.set_ambient(pgfx.Color(120, 120, 140))
    pgfx.reset_view()
    pgfx.text(state["font"], "hud", 4, 4, pgfx.YELLOW)

pgfx.run(update, render, on_ready=on_ready)
print("CAMERA OK")
"""


def test_view_transform_renders(run_script):
    import os

    font = os.path.join(os.path.dirname(__file__), "..", "..", "examples", "assets", "font.ttf")
    p = run_script(CAMERA.replace("@FONT@", repr(font)))
    assert p.returncode == 0, p.stderr
    assert "CAMERA OK" in p.stdout
