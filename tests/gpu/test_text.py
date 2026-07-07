"""Text rendering: Cyrillic, multiline, lazy atlas growth."""

import os

FONT = os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", "..", "examples", "assets", "font.ttf")
)

MULTILINE_CYRILLIC = """
import os
import pgfx

pgfx.init(480, 240, "text ru", fps_limit=0)
state = {}

def on_ready():
    state["font"] = pgfx.font_load(os.environ["PGFX_TEST_FONT"], 20)

frames = [0]
def update(dt):
    frames[0] += 1
    return frames[0] < 5

def render():
    pgfx.clear(pgfx.BLACK)
    if "font" in state:
        pgfx.text(state["font"], "Привет, мир!\\nВторая строка — кириллица", 10, 10, pgfx.WHITE)
        pgfx.text(state["font"], "ASCII over sprites", 10, 120, pgfx.GREEN, z=1)

pgfx.run(update, render, on_ready=on_ready)
print("TEXT OK")
"""


def test_multiline_cyrillic_renders(run_script):
    p = run_script(MULTILINE_CYRILLIC, env={"PGFX_TEST_FONT": FONT})
    assert p.returncode == 0, p.stderr
    assert "TEXT OK" in p.stdout
    assert "atlas is full" not in p.stderr


ATLAS_GROWTH = """
import os
import pgfx

# Big glyphs + full Cyrillic alphabet on top of pre-rasterized ASCII:
# guaranteed to overflow the initial atlas and trigger at least one growth
pgfx.init(480, 240, "atlas growth", fps_limit=0)
state = {}
ru = "".join(chr(c) for c in range(0x410, 0x450)) + "Ёё"

def on_ready():
    state["font"] = pgfx.font_load(os.environ["PGFX_TEST_FONT"], 120)

frames = [0]
def update(dt):
    frames[0] += 1
    return frames[0] < 5

def render():
    pgfx.clear(pgfx.BLACK)
    if "font" in state:
        pgfx.text(state["font"], ru, 10, 10, pgfx.WHITE)

pgfx.run(update, render, on_ready=on_ready)
print("GROWTH OK")
"""


def test_atlas_grows_without_errors(run_script):
    p = run_script(ATLAS_GROWTH, env={"PGFX_TEST_FONT": FONT})
    assert p.returncode == 0, p.stderr
    assert "GROWTH OK" in p.stdout
    assert "atlas is full" not in p.stderr


METRICS = """
import pgfx

pgfx.init(320, 240, "metrics", fps_limit=0)
state = {}

def on_ready():
    f = pgfx.font_load(@FONT@, 18)
    state["f"] = f

    w1, h1 = pgfx.text_size(f, "a")
    w2, _ = pgfx.text_size(f, "aa")
    assert 0 < w1 < w2, (w1, w2)
    assert h1 > 0

    # multiline: height is per line, width is the widest line
    w_wide, _ = pgfx.text_size(f, "the widest line")
    w, h = pgfx.text_size(f, "the widest line\\nx")
    assert w == w_wide
    assert abs(h - 2 * h1) < 1e-3

    # never-rendered glyphs measure via font metrics; after a draw the
    # cached-glyph path must give the same numbers
    state["before"] = pgfx.text_size(f, "Ы glyph cache Ю")

frames = [0]

def update(dt):
    frames[0] += 1
    if frames[0] == 3:
        after = pgfx.text_size(state["f"], "Ы glyph cache Ю")
        assert after == state["before"], (after, state["before"])
        print("METRICS OK")
        return False
    return True

def render():
    pgfx.clear(pgfx.BLACK)
    if "f" not in state:
        return
    # exercises all three alignments and rasterizes the cyrillic glyphs
    pgfx.text(state["f"], "Ы glyph cache Ю", 160, 40, pgfx.WHITE)
    pgfx.text(state["f"], "centered\\nlines", 160, 80, pgfx.WHITE, align="center")
    pgfx.text(state["f"], "right", 300, 140, pgfx.WHITE, align="right")

pgfx.run(update, render, on_ready=on_ready)
"""


def test_text_size_and_align(run_script):
    p = run_script(METRICS.replace("@FONT@", repr(FONT)))
    assert p.returncode == 0, p.stderr
    assert "METRICS OK" in p.stdout
