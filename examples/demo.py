"""Simple pgfx demo - sprite movement with keyboard input."""

import pgfx

# Initialize window
pgfx.init(800, 600, "pgfx Demo")

# Game state
player_x = 400.0
player_y = 300.0
player_speed = 200.0


def update(dt):
    """Update game logic."""
    global player_x, player_y

    # Player movement with arrow keys
    if pgfx.key_down(pgfx.KEY_LEFT):
        player_x -= player_speed * dt
    if pgfx.key_down(pgfx.KEY_RIGHT):
        player_x += player_speed * dt
    if pgfx.key_down(pgfx.KEY_UP):
        player_y -= player_speed * dt
    if pgfx.key_down(pgfx.KEY_DOWN):
        player_y += player_speed * dt

    # Keep player on screen
    player_x = max(20, min(780, player_x))
    player_y = max(20, min(580, player_y))

    # ESC to quit
    if pgfx.key_pressed(pgfx.KEY_ESCAPE):
        return False

    return True


def render():
    """Render the game."""
    # Clear screen with dark blue
    pgfx.clear(pgfx.Color(20, 20, 40))

    # Draw player as a colored rectangle (placeholder for sprite)
    pgfx.rect_fill(player_x - 20, player_y - 20, 40, 40, pgfx.GREEN)

    # Draw player direction indicator
    pgfx.circle_fill(player_x, player_y, 5, pgfx.YELLOW)

    # Draw UI border
    pgfx.line(0, 0, 800, 0, pgfx.WHITE)
    pgfx.line(800, 0, 800, 600, pgfx.WHITE)
    pgfx.line(800, 600, 0, 600, pgfx.WHITE)
    pgfx.line(0, 600, 0, 0, pgfx.WHITE)

    # Draw instructions
    w, h = pgfx.screen_size()

    # Simple text positioning
    pgfx.rect_fill(5, 5, 200, 60, pgfx.Color(0, 0, 0, 180))


# Run the game loop
pgfx.run(update, render)
