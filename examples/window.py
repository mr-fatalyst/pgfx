"""Window creation and basic event loop."""

import pgfx

pgfx.init(800, 600, "Window Example")

print("Window initialized successfully!")
print(f"Screen size: {pgfx.screen_size()}")


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(30, 30, 50))


pgfx.run(update, render)
