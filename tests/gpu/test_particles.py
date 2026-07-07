"""Particle config merging and JSON loading."""

import json
import os
import shutil

SET_MERGE = """
import pgfx

pgfx.init(320, 240, "particles", fps_limit=0)
state = {}

def on_ready():
    state["ps"] = pgfx.particles_create(
        primitive="square",
        emission_rate=500,
        lifetime=0.5,
        speed=10,
        start_size=4,
        end_size=1,
    )

frames = [0]

def update(dt):
    frames[0] += 1
    if "ps" not in state:
        return True
    ps = state["ps"]
    if frames[0] == 1:
        pgfx.particles_fire(ps, 160, 120)
        pgfx.particles_update(ps, 0.1)
        assert pgfx.particles_count(ps) > 0, "baseline emission failed"
    if frames[0] == 2:
        # change one parameter; the rest of the config must survive
        pgfx.particles_set(ps, direction=1.0)
        pgfx.particles_update(ps, 2.0)   # everything alive dies (lifetime 0.5)
        pgfx.particles_update(ps, 0.05)  # emits again only if the rate survived
        assert pgfx.particles_count(ps) > 0, "particles_set reset emission_rate"
        print("SET_MERGE OK")
        return False
    return True

def render():
    pgfx.clear(pgfx.BLACK)
    if "ps" in state:
        pgfx.particles_render(state["ps"])

pgfx.run(update, render, on_ready=on_ready)
"""


LOAD = """
import pgfx

path = @PATH@
pgfx.init(320, 240, "particles load", fps_limit=0)
state = {}

def on_ready():
    state["ps"] = pgfx.particles_load(path)

frames = [0]

def update(dt):
    frames[0] += 1
    if "ps" not in state:
        return True
    ps = state["ps"]
    if frames[0] == 1:
        pgfx.particles_fire(ps, 160, 120)
        pgfx.particles_update(ps, 0.1)
        assert pgfx.particles_count(ps) > 0, "loaded system does not emit"
        print("LOAD OK")
        return False
    return True

def render():
    pgfx.clear(pgfx.BLACK)
    if "ps" in state:
        pgfx.particles_render(state["ps"])

pgfx.run(update, render, on_ready=on_ready)
"""

# What the particle editor saves: flat keys, separate gravity axes,
# colors as lists (possibly floats from the sliders).
EDITOR_CONFIG = {
    "primitive": "circle_soft",
    "max_particles": 500,
    "emission_rate": 300,
    "lifetime_min": 0.5,
    "lifetime_max": 1.0,
    "speed_min": 40,
    "speed_max": 80,
    "direction": -1.57,
    "spread": 0.6,
    "gravity_x": 0,
    "gravity_y": 50,
    "start_color": [255.0, 200.0, 50.0, 255.0],
    "end_color": [255.0, 50.0, 0.0, 0.0],
    "start_size": 10,
    "end_size": 2,
}


def test_particles_set_merges_config(run_script):
    p = run_script(SET_MERGE)
    assert p.returncode == 0, p.stderr
    assert "SET_MERGE OK" in p.stdout


def test_particles_load_editor_format(run_script, tmp_path):
    path = tmp_path / "effect.json"
    path.write_text(json.dumps(EDITOR_CONFIG))
    p = run_script(LOAD.replace("@PATH@", repr(str(path))))
    assert p.returncode == 0, p.stderr
    assert "LOAD OK" in p.stdout


def test_particles_load_with_sprite(run_script, tmp_path):
    assets = os.path.join(os.path.dirname(__file__), "..", "..", "examples", "assets")
    shutil.copy(os.path.join(assets, "test.png"), tmp_path / "puff.png")
    cfg = {k: v for k, v in EDITOR_CONFIG.items() if k != "primitive"}
    cfg["sprite"] = "puff.png"  # resolved relative to the JSON file
    path = tmp_path / "effect.json"
    path.write_text(json.dumps(cfg))
    p = run_script(LOAD.replace("@PATH@", repr(str(path))))
    assert p.returncode == 0, p.stderr
    assert "LOAD OK" in p.stdout
