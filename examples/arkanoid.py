import math
import os
import random

import pgfx

SCREEN_W, SCREEN_H = 800, 600

pgfx.init(SCREEN_W, SCREEN_H, "Arkanoid")

# Paddle
PADDLE_W, PADDLE_H = 100, 15
PADDLE_SPEED = 400
paddle_x = SCREEN_W // 2 - PADDLE_W // 2
paddle_y = SCREEN_H - 40

# Ball
BALL_R = 8
BALL_SPEED = 350
ball_x = SCREEN_W // 2
ball_y = SCREEN_H // 2
ball_vx = BALL_SPEED * 0.7
ball_vy = BALL_SPEED * 0.7

# Bricks
BRICK_W, BRICK_H = 60, 20
BRICK_COLS = 11
BRICK_ROWS = 6
BRICK_GAP = 4
BRICK_OFFSET_X = (SCREEN_W - BRICK_COLS * (BRICK_W + BRICK_GAP)) // 2
BRICK_OFFSET_Y = 60

bricks = []
score = 0
lives = 3
game_over = False
game_won = False
ball_stuck = True  # Ball stuck to paddle at start
paused = False

# Particle systems
brick_particles = None
paddle_particles = None

# Colors for brick rows
BRICK_COLORS = [
    pgfx.Color(255, 50, 50),  # Red
    pgfx.Color(255, 150, 50),  # Orange
    pgfx.Color(255, 255, 50),  # Yellow
    pgfx.Color(50, 255, 50),  # Green
    pgfx.Color(50, 150, 255),  # Blue
    pgfx.Color(150, 50, 255),  # Purple
]

font = None


def reset_bricks():
    global bricks
    bricks = []
    for row in range(BRICK_ROWS):
        for col in range(BRICK_COLS):
            x = BRICK_OFFSET_X + col * (BRICK_W + BRICK_GAP)
            y = BRICK_OFFSET_Y + row * (BRICK_H + BRICK_GAP)
            bricks.append(
                {
                    "x": x,
                    "y": y,
                    "color": BRICK_COLORS[row % len(BRICK_COLORS)],
                    "points": (BRICK_ROWS - row) * 10,
                }
            )


def reset_ball():
    global ball_x, ball_y, ball_vx, ball_vy, ball_stuck
    ball_x = paddle_x + PADDLE_W // 2
    ball_y = paddle_y - BALL_R - 2
    angle = random.uniform(-0.3, 0.3) - math.pi / 2
    ball_vx = math.cos(angle) * BALL_SPEED
    ball_vy = math.sin(angle) * BALL_SPEED
    ball_stuck = True


def on_ready():
    global font, brick_particles, paddle_particles
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 18)

    # Brick destruction particles - colorful burst
    brick_particles = pgfx.particles_create(
        primitive="square",
        emission_rate=0,  # Manual burst only
        lifetime_min=0.3,
        lifetime_max=0.8,
        speed_min=100,
        speed_max=300,
        direction=0,
        spread=math.pi * 2,
        gravity=(0, 400),
        start_color=(255, 255, 255, 255),
        end_color=(255, 255, 255, 0),
        start_size=6,
        end_size=2,
        max_particles=500,
    )

    # Paddle bounce particles - small spark
    paddle_particles = pgfx.particles_create(
        primitive="circle",
        emission_rate=0,
        lifetime_min=0.2,
        lifetime_max=0.5,
        speed_min=50,
        speed_max=150,
        direction=-math.pi / 2,
        spread=math.pi * 0.8,
        gravity=(0, 200),
        start_color=(255, 255, 200, 255),
        end_color=(255, 200, 100, 0),
        start_size=4,
        end_size=1,
        max_particles=200,
    )

    reset_bricks()
    reset_ball()


def spawn_brick_particles(x, y, color):
    """Spawn particles at brick position with brick color."""
    # Set particle color to match brick
    pgfx.particles_set(
        brick_particles,
        start_color=(color.r, color.g, color.b, 255),
        end_color=(color.r, color.g, color.b, 0),
    )
    pgfx.particles_emit(brick_particles, x + BRICK_W // 2, y + BRICK_H // 2, 15)


def update(dt):
    global paddle_x, ball_x, ball_y, ball_vx, ball_vy
    global score, lives, game_over, game_won, ball_stuck, paused

    # Always update particles (even when paused for visual effect)
    pgfx.particles_update(brick_particles, dt)
    pgfx.particles_update(paddle_particles, dt)

    # Pause toggle
    if pgfx.key_pressed(pgfx.KEY_P):
        paused = not paused

    if paused:
        return not pgfx.key_pressed(pgfx.KEY_ESCAPE)

    if game_over or game_won:
        if pgfx.key_pressed(pgfx.KEY_SPACE):
            score = 0
            lives = 3
            game_over = False
            game_won = False
            reset_bricks()
            reset_ball()
        return not pgfx.key_pressed(pgfx.KEY_ESCAPE)

    # Paddle movement
    if pgfx.key_down(pgfx.KEY_LEFT) or pgfx.key_down(pgfx.KEY_A):
        paddle_x -= PADDLE_SPEED * dt
    if pgfx.key_down(pgfx.KEY_RIGHT) or pgfx.key_down(pgfx.KEY_D):
        paddle_x += PADDLE_SPEED * dt

    paddle_x = max(0, min(SCREEN_W - PADDLE_W, paddle_x))

    # Ball stuck to paddle
    if ball_stuck:
        ball_x = paddle_x + PADDLE_W // 2
        ball_y = paddle_y - BALL_R - 2
        if pgfx.key_pressed(pgfx.KEY_SPACE):
            ball_stuck = False
            angle = random.uniform(-0.3, 0.3) - math.pi / 2
            ball_vx = math.cos(angle) * BALL_SPEED
            ball_vy = math.sin(angle) * BALL_SPEED
        return not pgfx.key_pressed(pgfx.KEY_ESCAPE)

    # Ball movement
    ball_x += ball_vx * dt
    ball_y += ball_vy * dt

    # Wall collisions
    if ball_x - BALL_R < 0:
        ball_x = BALL_R
        ball_vx = abs(ball_vx)
    elif ball_x + BALL_R > SCREEN_W:
        ball_x = SCREEN_W - BALL_R
        ball_vx = -abs(ball_vx)

    if ball_y - BALL_R < 0:
        ball_y = BALL_R
        ball_vy = abs(ball_vy)

    # Ball fell off bottom
    if ball_y > SCREEN_H + BALL_R:
        lives -= 1
        if lives <= 0:
            game_over = True
        else:
            reset_ball()
        return not pgfx.key_pressed(pgfx.KEY_ESCAPE)

    # Paddle collision
    if (
        ball_vy > 0
        and ball_y + BALL_R >= paddle_y
        and ball_y - BALL_R <= paddle_y + PADDLE_H
        and ball_x >= paddle_x
        and ball_x <= paddle_x + PADDLE_W
    ):
        ball_y = paddle_y - BALL_R
        # Angle based on where ball hit paddle
        hit_pos = (ball_x - paddle_x) / PADDLE_W  # 0 to 1
        angle = (hit_pos - 0.5) * math.pi * 0.7 - math.pi / 2
        speed = math.sqrt(ball_vx**2 + ball_vy**2)
        ball_vx = math.cos(angle) * speed
        ball_vy = math.sin(angle) * speed

        # Spawn paddle particles
        pgfx.particles_emit(paddle_particles, ball_x, paddle_y, 8)

    # Brick collisions
    for brick in bricks[:]:
        bx, by = brick["x"], brick["y"]
        # Check collision
        if (
            ball_x + BALL_R > bx
            and ball_x - BALL_R < bx + BRICK_W
            and ball_y + BALL_R > by
            and ball_y - BALL_R < by + BRICK_H
        ):
            # Determine collision side
            dx_left = abs(ball_x - bx)
            dx_right = abs(ball_x - (bx + BRICK_W))
            dy_top = abs(ball_y - by)
            dy_bottom = abs(ball_y - (by + BRICK_H))

            min_d = min(dx_left, dx_right, dy_top, dy_bottom)

            if min_d == dx_left or min_d == dx_right:
                ball_vx = -ball_vx
            else:
                ball_vy = -ball_vy

            # Spawn particles with brick color
            spawn_brick_particles(bx, by, brick["color"])

            bricks.remove(brick)
            score += brick["points"]
            break

    # Check win
    if not bricks:
        game_won = True

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(20, 20, 35))

    if not font:
        return

    # Draw bricks
    for brick in bricks:
        pgfx.rect_fill(brick["x"], brick["y"], BRICK_W, BRICK_H, brick["color"])
        # Highlight
        pgfx.rect_fill(brick["x"], brick["y"], BRICK_W, 3, pgfx.Color(255, 255, 255, 80))

    # Draw paddle
    pgfx.rect_fill(paddle_x, paddle_y, PADDLE_W, PADDLE_H, pgfx.Color(200, 200, 220))
    pgfx.rect_fill(paddle_x, paddle_y, PADDLE_W, 3, pgfx.Color(255, 255, 255, 100))

    # Draw ball
    pgfx.circle_fill(ball_x, ball_y, BALL_R, pgfx.WHITE)

    # Particles
    pgfx.particles_render(brick_particles)
    pgfx.particles_render(paddle_particles)

    # UI
    pgfx.text(font, f"Score: {score}", 10, 10, pgfx.WHITE)
    pgfx.text(font, f"Lives: {lives}", SCREEN_W - 80, 10, pgfx.WHITE)

    if paused:
        pgfx.rect_fill(SCREEN_W // 2 - 80, SCREEN_H // 2 - 30, 160, 60, pgfx.Color(0, 0, 0, 200))
        pgfx.text(font, "PAUSED", SCREEN_W // 2 - 40, SCREEN_H // 2 - 15, pgfx.YELLOW)
        pgfx.text(font, "Press P to resume", SCREEN_W // 2 - 80, SCREEN_H // 2 + 10, pgfx.WHITE)

    if ball_stuck and not paused:
        pgfx.text(font, "Press SPACE to launch", SCREEN_W // 2 - 100, SCREEN_H // 2, pgfx.YELLOW)

    if game_over:
        pgfx.rect_fill(SCREEN_W // 2 - 120, SCREEN_H // 2 - 50, 240, 100, pgfx.Color(0, 0, 0, 200))
        pgfx.text(font, "GAME OVER", SCREEN_W // 2 - 60, SCREEN_H // 2 - 30, pgfx.RED)
        pgfx.text(font, f"Final Score: {score}", SCREEN_W // 2 - 70, SCREEN_H // 2, pgfx.WHITE)
        pgfx.text(
            font, "Press SPACE to restart", SCREEN_W // 2 - 100, SCREEN_H // 2 + 25, pgfx.YELLOW
        )

    if game_won:
        pgfx.rect_fill(SCREEN_W // 2 - 120, SCREEN_H // 2 - 50, 240, 100, pgfx.Color(0, 0, 0, 200))
        pgfx.text(font, "YOU WIN!", SCREEN_W // 2 - 50, SCREEN_H // 2 - 30, pgfx.GREEN)
        pgfx.text(font, f"Final Score: {score}", SCREEN_W // 2 - 70, SCREEN_H // 2, pgfx.WHITE)
        pgfx.text(
            font, "Press SPACE to restart", SCREEN_W // 2 - 100, SCREEN_H // 2 + 25, pgfx.YELLOW
        )


pgfx.run(update, render, on_ready=on_ready)
