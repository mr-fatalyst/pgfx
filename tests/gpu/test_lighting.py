"""Lightmap pass: ambient darkening plus additive lights."""

LIGHTING = """
import pgfx

pgfx.init(320, 240, "lighting", fps_limit=0)
state = {}

def on_ready():
    state["l1"] = pgfx.light_create(100, pgfx.Color(255, 200, 100))
    state["l2"] = pgfx.light_create(80, pgfx.Color(100, 100, 255))
    pgfx.light_set_flicker(state["l2"], 0.3, 2.0)
    pgfx.set_ambient(pgfx.Color(30, 30, 50))

frames = [0]
def update(dt):
    frames[0] += 1
    return frames[0] < 10

def render():
    pgfx.clear(pgfx.Color(20, 20, 30))
    pgfx.rect_fill(50, 50, 100, 80, pgfx.Color(120, 120, 120))
    if state:
        pgfx.light_draw(state["l1"], 100, 90)
        pgfx.light_draw(state["l2"], 250, 150)

pgfx.run(update, render, on_ready=on_ready)
print("LIGHTING OK")
"""


def test_ambient_and_lights_render(run_script):
    p = run_script(LIGHTING)
    assert p.returncode == 0, p.stderr
    assert "LIGHTING OK" in p.stdout


AMBIENT_ONLY = """
import pgfx

pgfx.init(320, 240, "ambient only", fps_limit=0)

def on_ready():
    pgfx.set_ambient(pgfx.Color(80, 80, 80))

frames = [0]
def update(dt):
    frames[0] += 1
    return frames[0] < 5

def render():
    pgfx.clear(pgfx.Color(40, 40, 40))
    pgfx.rect_fill(10, 10, 50, 50, pgfx.WHITE)

pgfx.run(update, render, on_ready=on_ready)
print("AMBIENT OK")
"""


def test_ambient_without_lights(run_script):
    p = run_script(AMBIENT_ONLY)
    assert p.returncode == 0, p.stderr
    assert "AMBIENT OK" in p.stdout
