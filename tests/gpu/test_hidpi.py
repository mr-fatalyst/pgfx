"""HiDPI: the user API works in logical pixels regardless of display scale.

Uses winit's WINIT_X11_SCALE_FACTOR override, so these tests need the X11
backend (WAYLAND_DISPLAY is popped inside the scripts).
"""

HIDPI_LOGICAL_SIZE = """
import os
os.environ.pop("WAYLAND_DISPLAY", None)  # force the X11 backend
import pgfx

pgfx.init(320, 240, "hidpi", fps_limit=0)
sizes = []
frames = [0]

def update(dt):
    frames[0] += 1
    if frames[0] == 3:
        sizes.append(pgfx.screen_size())
    return frames[0] < 5

pgfx.run(update, lambda: None)
assert sizes and sizes[0] == (320, 240), f"expected logical (320, 240), got {sizes}"
print("HIDPI OK")
"""


def test_screen_size_is_logical_under_2x_scale(run_script):
    p = run_script(HIDPI_LOGICAL_SIZE, env={"WINIT_X11_SCALE_FACTOR": "2"})
    assert p.returncode == 0, p.stderr
    assert "HIDPI OK" in p.stdout


def test_screen_size_is_logical_under_fractional_scale(run_script):
    p = run_script(HIDPI_LOGICAL_SIZE, env={"WINIT_X11_SCALE_FACTOR": "1.5"})
    assert p.returncode == 0, p.stderr
    assert "HIDPI OK" in p.stdout
