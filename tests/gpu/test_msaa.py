"""MSAA screen pass: resolve, blending, lighting and targets together."""

MSAA = """
import pgfx

pgfx.init(320, 240, "msaa", fps_limit=0, msaa=4)
state = {}

def on_ready():
    state["t"] = pgfx.target_create(64, 64)
    state["spr"] = pgfx.target_sprite(state["t"])
    state["light"] = pgfx.light_create(90, pgfx.Color(255, 200, 120))
    state["ps"] = pgfx.particles_create(primitive="square", emission_rate=150, blend="add")
    pgfx.particles_fire(state["ps"], 160, 120)
    pgfx.set_ambient(pgfx.Color(140, 140, 160))

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
    # 1x target pass alongside the 4x screen pass
    pgfx.render_to(state["t"])
    pgfx.rect_fill_ex(32, 32, 40, 16, pgfx.GREEN, rot=t)
    pgfx.render_to(None)
    pgfx.draw(state["spr"], 10, 10)
    # rotated geometry, additive blend and lighting through the resolve
    pgfx.rect_fill_ex(160, 120, 100, 40, pgfx.RED, rot=t * 3)
    pgfx.line(0, 0, 320, 240, pgfx.WHITE, width=5)
    pgfx.particles_render(state["ps"])
    pgfx.light_draw(state["light"], 160, 120)

pgfx.run(update, render, on_ready=on_ready)
print("MSAA OK")
"""


def test_msaa_renders(run_script):
    p = run_script(MSAA)
    assert p.returncode == 0, p.stderr
    assert "MSAA OK" in p.stdout
