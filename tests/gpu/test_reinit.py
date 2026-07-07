"""init -> run -> quit -> init -> run cycles in a single process."""

TWO_CYCLES = """
import pgfx

for cycle in range(2):
    pgfx.init(320, 240, f"cycle {cycle}", fps_limit=0)
    frames = [0]

    def update(dt):
        frames[0] += 1
        return frames[0] < 5

    pgfx.run(update, lambda: None)
    assert frames[0] == 5, frames
print("CYCLES OK")
"""


def test_init_run_twice_in_one_process(run_script):
    p = run_script(TWO_CYCLES)
    assert p.returncode == 0, p.stderr
    assert "CYCLES OK" in p.stdout


QUIT_BEFORE_RUN = """
import pgfx

pgfx.init(320, 240, "first")
pgfx.quit()
pgfx.init(320, 240, "second", fps_limit=0)

frames = [0]
def update(dt):
    frames[0] += 1
    return frames[0] < 3
pgfx.run(update, lambda: None)
print("QUIT-REINIT OK")
"""


def test_quit_before_run_allows_reinit(run_script):
    p = run_script(QUIT_BEFORE_RUN)
    assert p.returncode == 0, p.stderr
    assert "QUIT-REINIT OK" in p.stdout


QUIT_FROM_UPDATE = """
import pgfx

pgfx.init(320, 240, "quit from update", fps_limit=0)

def update(dt):
    pgfx.quit()
    return True  # quit() must stop the loop even if update returns True

pgfx.run(update, lambda: None)

# After run() returns the engine is deinitialized
try:
    pgfx.dt()
except RuntimeError as e:
    assert "not initialized" in str(e), e
    print("QUIT-FROM-UPDATE OK")
"""


def test_quit_from_update_stops_loop_and_deinitializes(run_script):
    p = run_script(QUIT_FROM_UPDATE)
    assert p.returncode == 0, p.stderr
    assert "QUIT-FROM-UPDATE OK" in p.stdout


RESOURCES_FRESH = """
import os
import pgfx

assets = os.environ["PGFX_TEST_ASSETS"]

pgfx.init(320, 240, "resources", fps_limit=0)
ids = []

def on_ready():
    ids.append(pgfx.sprite_load(os.path.join(assets, "ball.png")))

frames = [0]
def update(dt):
    frames[0] += 1
    return frames[0] < 3

pgfx.run(update, lambda: None, on_ready=on_ready)

# Second cycle: old sprite ID must be invalid, loading works again
pgfx.init(320, 240, "resources 2", fps_limit=0)

def on_ready2():
    try:
        pgfx.sprite_rect(ids[0], 0, 0)
        raise AssertionError("stale sprite ID resolved after re-init")
    except ValueError:
        pass
    ids.append(pgfx.sprite_load(os.path.join(assets, "ball.png")))

frames[0] = 0
pgfx.run(update, lambda: None, on_ready=on_ready2)
assert len(ids) == 2
print("RESOURCES OK")
"""


def test_resources_do_not_leak_across_reinit(run_script):
    import os

    assets = os.path.join(os.path.dirname(__file__), "..", "..", "examples", "assets")
    p = run_script(RESOURCES_FRESH, env={"PGFX_TEST_ASSETS": os.path.abspath(assets)})
    assert p.returncode == 0, p.stderr
    assert "RESOURCES OK" in p.stdout
