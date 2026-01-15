import pgfx

pgfx.init(800, 600, "Primitives Example")


def update(dt):
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(30, 30, 50))

    # Rectangles
    pgfx.rect_fill(50, 50, 100, 80, pgfx.RED)
    pgfx.rect_fill(200, 50, 100, 80, pgfx.GREEN)
    pgfx.rect_fill(350, 50, 100, 80, pgfx.BLUE)

    # Lines
    pgfx.line(50, 200, 450, 200, pgfx.WHITE)
    pgfx.line(50, 200, 250, 300, pgfx.YELLOW)

    # Circles
    pgfx.circle_fill(150, 400, 50, pgfx.CYAN)
    pgfx.circle_fill(300, 400, 30, pgfx.MAGENTA)
    pgfx.circle_fill(450, 400, 70, pgfx.Color(255, 128, 0))


pgfx.run(update, render)
