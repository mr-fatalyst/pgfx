import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pgfx

SCREEN_W, SCREEN_H = 1280, 720

pgfx.init(SCREEN_W, SCREEN_H, "Benchmark - Press UP/DOWN to change count")

# Load sprite and font after init
sprite = None
font = None


class Ball:
    def __init__(self):
        self.x = random.uniform(50, SCREEN_W - 50)
        self.y = random.uniform(50, SCREEN_H - 50)
        self.vx = random.uniform(-200, 200)
        self.vy = random.uniform(-200, 200)
        # Ensure minimum speed
        if abs(self.vx) < 50:
            self.vx = 50 if self.vx >= 0 else -50
        if abs(self.vy) < 50:
            self.vy = 50 if self.vy >= 0 else -50

    def update(self, dt):
        self.x += self.vx * dt
        self.y += self.vy * dt

        # Bounce off walls
        if self.x < 0:
            self.x = 0
            self.vx = -self.vx
        elif self.x > SCREEN_W - 32:
            self.x = SCREEN_W - 32
            self.vx = -self.vx

        if self.y < 0:
            self.y = 0
            self.vy = -self.vy
        elif self.y > SCREEN_H - 32:
            self.y = SCREEN_H - 32
            self.vy = -self.vy


balls = []
target_count = 100


def add_balls(count):
    for _ in range(count):
        balls.append(Ball())


def remove_balls(count):
    for _ in range(min(count, len(balls))):
        if balls:
            balls.pop()


def on_ready():
    global sprite, font
    sprite = pgfx.sprite_load(os.path.join(os.path.dirname(__file__), "assets/ball.png"))
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 20)
    add_balls(target_count)


def update(dt):
    global target_count

    # Controls
    if pgfx.key_pressed(pgfx.KEY_UP):
        add_balls(100)
        target_count += 100
    if pgfx.key_pressed(pgfx.KEY_DOWN):
        remove_balls(100)
        target_count = max(0, target_count - 100)
    if pgfx.key_pressed(pgfx.KEY_RIGHT):
        add_balls(1000)
        target_count += 1000
    if pgfx.key_pressed(pgfx.KEY_LEFT):
        remove_balls(1000)
        target_count = max(0, target_count - 1000)

    # Update all balls
    for ball in balls:
        ball.update(dt)

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    global sprite, font

    pgfx.clear(pgfx.Color(20, 20, 30))

    # Draw all balls
    if sprite is not None:
        for ball in balls:
            pgfx.draw(sprite, ball.x, ball.y)

    # Draw stats panel
    if font is not None:
        pgfx.rect_fill(5, 5, 200, 80, pgfx.Color(0, 0, 0, 180))
        pgfx.text(font, f"FPS: {pgfx.fps()}", 15, 15, pgfx.Color(255, 255, 255))
        pgfx.text(font, f"Objects: {len(balls)}", 15, 40, pgfx.Color(255, 255, 255))
        pgfx.text(font, "UP/DOWN: +/-100", 15, 65, pgfx.Color(150, 150, 150))


print("Benchmark Controls:")
print("  UP/DOWN - Add/Remove 100 sprites")
print("  RIGHT/LEFT - Add/Remove 1000 sprites")
print("  ESC - Exit")
print()

pgfx.run(update, render, on_ready=on_ready)
