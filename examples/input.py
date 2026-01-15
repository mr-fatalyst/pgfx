import pgfx

pgfx.init(800, 600, "Input Example")


def update(dt):
    if pgfx.key_pressed(pgfx.KEY_SPACE):
        print("SPACE pressed!")
    if pgfx.key_down(pgfx.KEY_A):
        print("A is held")

    if pgfx.mouse_pressed(pgfx.MOUSE_LEFT):
        print(f"Left click at {pgfx.mouse_pos()}")

    wheel = pgfx.mouse_wheel()
    if wheel != 0:
        print(f"Mouse wheel: {wheel}")

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(30, 30, 50))


pgfx.run(update, render)
