import pgfx

pgfx.init(800, 600, "Clear Example")

frame = 0


def update(dt):
    global frame
    frame += 1
    return frame < 300  # 5 seconds at 60fps


def render():
    # Change color every 60 frames
    if frame < 60:
        pgfx.clear(pgfx.RED)
    elif frame < 120:
        pgfx.clear(pgfx.GREEN)
    elif frame < 180:
        pgfx.clear(pgfx.BLUE)
    else:
        pgfx.clear(pgfx.Color(128, 0, 128))  # Purple


pgfx.run(update, render)
