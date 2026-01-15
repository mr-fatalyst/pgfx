import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pgfx

SCREEN_W, SCREEN_H = 1280, 720

pgfx.init(SCREEN_W, SCREEN_H, "Day/Night Cycle - LEFT/RIGHT to change time")

font = None

# Time in minutes (0-1440, where 0 = midnight, 720 = noon)
current_time = 720  # Start at noon

# Cloud positions (x, y, speed)
clouds = [
    {"x": 100, "y": 80, "w": 120, "h": 40, "speed": 15},
    {"x": 400, "y": 120, "w": 80, "h": 30, "speed": 10},
    {"x": 700, "y": 60, "w": 150, "h": 50, "speed": 20},
    {"x": 1000, "y": 140, "w": 100, "h": 35, "speed": 12},
    {"x": 200, "y": 180, "w": 90, "h": 32, "speed": 8},
]


def lerp(a, b, t):
    """Linear interpolation."""
    return a + (b - a) * t


def lerp_color(c1, c2, t):
    """Interpolate between two colors."""
    return pgfx.Color(
        int(lerp(c1[0], c2[0], t)), int(lerp(c1[1], c2[1], t)), int(lerp(c1[2], c2[2], t))
    )


def get_sky_color(time_minutes):
    """Get sky color based on time of day."""
    hour = time_minutes / 60

    # Define key colors for different times
    midnight = (10, 10, 40)
    dawn = (255, 150, 100)
    morning = (135, 206, 235)
    noon = (100, 180, 255)
    evening = (255, 130, 80)
    dusk = (80, 60, 120)

    if hour < 5:  # Night
        return lerp_color(midnight, midnight, 0)
    elif hour < 6:  # Dawn starts
        t = (hour - 5) / 1
        return lerp_color(midnight, dawn, t)
    elif hour < 8:  # Dawn to morning
        t = (hour - 6) / 2
        return lerp_color(dawn, morning, t)
    elif hour < 12:  # Morning to noon
        t = (hour - 8) / 4
        return lerp_color(morning, noon, t)
    elif hour < 17:  # Noon to evening start
        t = (hour - 12) / 5
        return lerp_color(noon, morning, t)
    elif hour < 19:  # Evening
        t = (hour - 17) / 2
        return lerp_color(morning, evening, t)
    elif hour < 20:  # Dusk
        t = (hour - 19) / 1
        return lerp_color(evening, dusk, t)
    elif hour < 21:  # Dusk to night
        t = (hour - 20) / 1
        return lerp_color(dusk, midnight, t)
    else:  # Night
        return lerp_color(midnight, midnight, 0)


def get_sky_bottom_color(time_minutes):
    """Get horizon color (lighter than sky)."""
    hour = time_minutes / 60

    midnight = (20, 20, 60)
    dawn = (255, 200, 150)
    day = (200, 230, 255)
    evening = (255, 180, 120)
    dusk = (100, 80, 140)

    if hour < 5:
        return lerp_color(midnight, midnight, 0)
    elif hour < 6:
        t = (hour - 5) / 1
        return lerp_color(midnight, dawn, t)
    elif hour < 8:
        t = (hour - 6) / 2
        return lerp_color(dawn, day, t)
    elif hour < 17:
        return lerp_color(day, day, 0)
    elif hour < 19:
        t = (hour - 17) / 2
        return lerp_color(day, evening, t)
    elif hour < 20:
        t = (hour - 19) / 1
        return lerp_color(evening, dusk, t)
    elif hour < 21:
        t = (hour - 20) / 1
        return lerp_color(dusk, midnight, t)
    else:
        return lerp_color(midnight, midnight, 0)


def get_sun_moon_pos(time_minutes):
    """Get sun and moon positions based on time."""
    # Full circle in 24 hours, sun at top at noon (720 min)
    # Angle: 0 = right, 90 = top, 180 = left, 270 = bottom

    # Sun angle: at noon (720) sun is at top (90 degrees)
    sun_angle = ((time_minutes - 720) / 1440) * 360 + 90
    sun_angle_rad = math.radians(sun_angle)

    # Center of rotation - at ground level, smaller radius to keep sun/moon visible
    cx, cy = SCREEN_W // 2, SCREEN_H - 150  # Ground level
    radius = SCREEN_H - 200  # Smaller radius so sunrise/sunset visible on screen

    sun_x = cx + math.cos(sun_angle_rad) * radius
    sun_y = cy - math.sin(sun_angle_rad) * radius

    # Moon is opposite to sun
    moon_x = cx + math.cos(sun_angle_rad + math.pi) * radius
    moon_y = cy - math.sin(sun_angle_rad + math.pi) * radius

    return (sun_x, sun_y), (moon_x, moon_y)


def get_sun_color(time_minutes):
    """Sun color changes during sunrise/sunset."""
    hour = time_minutes / 60

    if hour < 6 or hour > 20:
        return pgfx.Color(255, 200, 100, 0)  # Invisible
    elif hour < 8:
        t = (hour - 6) / 2
        return pgfx.Color(
            255, int(lerp(100, 220, t)), int(lerp(50, 100, t)), int(lerp(200, 255, t))
        )
    elif hour < 17:
        return pgfx.Color(255, 240, 150)
    elif hour < 19:
        t = (hour - 17) / 2
        return pgfx.Color(255, int(lerp(240, 100, t)), int(lerp(150, 50, t)))
    else:
        t = (hour - 19) / 1
        return pgfx.Color(255, 100, 50, int(lerp(255, 0, t)))


def get_moon_alpha(time_minutes):
    """Moon visibility."""
    hour = time_minutes / 60

    if hour < 5:
        return 255
    elif hour < 7:
        return int(lerp(255, 0, (hour - 5) / 2))
    elif hour < 18:
        return 0
    elif hour < 20:
        return int(lerp(0, 255, (hour - 18) / 2))
    else:
        return 255


def time_to_string(time_minutes):
    """Convert minutes to HH:MM format."""
    hours = int(time_minutes // 60) % 24
    minutes = int(time_minutes % 60)
    return f"{hours:02d}:{minutes:02d}"


def get_time_of_day(time_minutes):
    """Get time of day name."""
    hour = time_minutes / 60
    if hour < 5:
        return "Night"
    elif hour < 8:
        return "Dawn"
    elif hour < 12:
        return "Morning"
    elif hour < 17:
        return "Afternoon"
    elif hour < 20:
        return "Evening"
    else:
        return "Night"


def draw_gradient_sky(top_color, bottom_color):
    """Draw sky gradient using horizontal strips."""
    strips = 20
    strip_height = (SCREEN_H - 150) // strips  # Leave space for ground

    for i in range(strips):
        t = i / (strips - 1)
        color = lerp_color(
            (top_color.r, top_color.g, top_color.b),
            (bottom_color.r, bottom_color.g, bottom_color.b),
            t,
        )
        y = i * strip_height
        pgfx.rect_fill(0, y, SCREEN_W, strip_height + 1, color)


def draw_sun(x, y, color):
    """Draw sun with glow."""
    if color.a < 10:
        return
    # Glow
    glow_color = pgfx.Color(color.r, color.g, color.b, min(color.a // 3, 80))
    pgfx.circle_fill(x, y, 80, glow_color)
    glow_color2 = pgfx.Color(color.r, color.g, color.b, min(color.a // 2, 120))
    pgfx.circle_fill(x, y, 55, glow_color2)
    # Sun body
    pgfx.circle_fill(x, y, 40, color)


def draw_moon(x, y, alpha):
    """Draw moon."""
    if alpha < 10:
        return
    # Glow
    pgfx.circle_fill(x, y, 50, pgfx.Color(200, 200, 255, alpha // 4))
    # Moon body
    pgfx.circle_fill(x, y, 35, pgfx.Color(240, 240, 255, alpha))
    # Craters (darker spots)
    pgfx.circle_fill(x - 10, y - 8, 8, pgfx.Color(200, 200, 220, alpha))
    pgfx.circle_fill(x + 12, y + 5, 6, pgfx.Color(210, 210, 230, alpha))
    pgfx.circle_fill(x - 5, y + 12, 5, pgfx.Color(205, 205, 225, alpha))


def draw_cloud(x, y, w, h, alpha):
    """Draw a fluffy cloud."""
    color = pgfx.Color(255, 255, 255, alpha)
    # Main body - overlapping circles
    pgfx.circle_fill(x, y, h * 0.8, color)
    pgfx.circle_fill(x - w * 0.25, y + h * 0.1, h * 0.6, color)
    pgfx.circle_fill(x + w * 0.25, y + h * 0.1, h * 0.7, color)
    pgfx.circle_fill(x - w * 0.4, y + h * 0.2, h * 0.4, color)
    pgfx.circle_fill(x + w * 0.4, y + h * 0.15, h * 0.5, color)


def draw_mountains(time_minutes):
    """Draw mountain silhouettes."""
    hour = time_minutes / 60

    # Mountain color changes with time of day
    if hour < 6 or hour > 20:
        color = pgfx.Color(20, 25, 40)  # Dark blue at night
    elif hour < 8:
        t = (hour - 6) / 2
        color = lerp_color((20, 25, 40), (60, 70, 90), t)
    elif hour < 18:
        color = pgfx.Color(60, 70, 90)  # Gray-blue during day
    elif hour < 20:
        t = (hour - 18) / 2
        color = lerp_color((60, 70, 90), (40, 35, 50), t)
    else:
        color = pgfx.Color(20, 25, 40)

    # Back mountains (smaller, lighter)
    back_color = pgfx.Color(min(color.r + 20, 255), min(color.g + 20, 255), min(color.b + 30, 255))

    # Draw back mountain range (lower)
    mountains_back = [
        (0, 480),
        (100, 430),
        (200, 460),
        (350, 410),
        (500, 450),
        (650, 400),
        (800, 440),
        (950, 420),
        (1100, 460),
        (1280, 480),
    ]
    for i in range(len(mountains_back) - 1):
        x1, y1 = mountains_back[i]
        x2, y2 = mountains_back[i + 1]
        # Fill triangle from peak to ground
        draw_mountain_segment(x1, y1, x2, y2, SCREEN_H - 150, back_color)

    # Draw front mountain range (darker, slightly taller)
    mountains_front = [
        (0, 500),
        (150, 440),
        (300, 480),
        (450, 420),
        (600, 470),
        (750, 400),
        (900, 450),
        (1050, 430),
        (1200, 470),
        (1280, 500),
    ]
    for i in range(len(mountains_front) - 1):
        x1, y1 = mountains_front[i]
        x2, y2 = mountains_front[i + 1]
        draw_mountain_segment(x1, y1, x2, y2, SCREEN_H, color)  # Extend to bottom of screen


def draw_mountain_segment(x1, y1, x2, y2, ground_y, color):
    """Draw a mountain segment as filled polygon."""
    # Draw as series of vertical strips (optimized - fewer draw calls)
    if x2 < x1:
        x1, y1, x2, y2 = x2, y2, x1, y1

    strip_width = 2  # Draw in 2-pixel strips for smooth edges
    for x in range(int(x1), int(x2) + 1, strip_width):
        if x2 != x1:
            t = (x - x1) / (x2 - x1)
            y = y1 + (y2 - y1) * t
        else:
            y = y1
        height = ground_y - y
        if height > 0:
            w = min(strip_width, int(x2) - x + 1)
            pgfx.rect_fill(x, int(y), w, int(height) + 1, color)


def draw_ground(time_minutes):
    """Draw grass and dirt ground."""
    hour = time_minutes / 60

    # Calculate darkness factor (0 = day, 1 = night)
    if hour < 5 or hour > 21:
        darkness = 0.6
    elif hour < 7:
        darkness = lerp(0.6, 0, (hour - 5) / 2)
    elif hour < 18:
        darkness = 0
    elif hour < 21:
        darkness = lerp(0, 0.6, (hour - 18) / 3)
    else:
        darkness = 0.6

    # Base colors
    dirt_day = (80, 50, 30)
    grass_day = (50, 120, 50)
    grass_tuft_day = (60, 140, 60)
    dirt_line_day = (60, 40, 25)

    # Darken colors for night
    def darken(color, factor):
        return pgfx.Color(
            int(color[0] * (1 - factor)), int(color[1] * (1 - factor)), int(color[2] * (1 - factor))
        )

    dirt_color = darken(dirt_day, darkness)
    grass_color = darken(grass_day, darkness)
    tuft_color = darken(grass_tuft_day, darkness)
    line_color = darken(dirt_line_day, darkness)

    # Dirt layer
    pgfx.rect_fill(0, SCREEN_H - 100, SCREEN_W, 100, dirt_color)

    # Grass layer
    pgfx.rect_fill(0, SCREEN_H - 150, SCREEN_W, 50, grass_color)

    # Grass tufts
    for i in range(0, SCREEN_W, 20):
        h = 10 + (i * 7) % 15
        pgfx.rect_fill(i, SCREEN_H - 150 - h, 8, h + 5, tuft_color)

    # Dirt texture lines
    for i in range(5):
        y = SCREEN_H - 90 + i * 20
        pgfx.rect_fill(0, y, SCREEN_W, 2, line_color)


def draw_stars(alpha):
    """Draw stars at night."""
    if alpha < 10:
        return
    star_positions = [
        (100, 50),
        (250, 80),
        (400, 30),
        (550, 90),
        (700, 45),
        (850, 70),
        (1000, 35),
        (1150, 85),
        (180, 120),
        (320, 150),
        (480, 110),
        (620, 140),
        (780, 100),
        (920, 130),
        (1080, 115),
        (60, 180),
        (220, 200),
        (380, 170),
        (540, 190),
        (700, 175),
    ]
    color = pgfx.Color(255, 255, 255, alpha)
    for x, y in star_positions:
        pgfx.circle_fill(x, y, 2, color)


def on_ready():
    global font
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 20)


def update(dt):
    global current_time

    # Time controls
    time_delta = 0
    if pgfx.key_down(pgfx.KEY_RIGHT):
        time_delta = 200 * dt  # Fast forward when held
    elif pgfx.key_down(pgfx.KEY_LEFT):
        time_delta = -200 * dt  # Rewind when held

    if pgfx.key_pressed(pgfx.KEY_RIGHT):
        time_delta = 10  # 10 minutes per press
    elif pgfx.key_pressed(pgfx.KEY_LEFT):
        time_delta = -10

    current_time = (current_time + time_delta) % 1440
    if current_time < 0:
        current_time += 1440

    # Update cloud positions
    for cloud in clouds:
        cloud["x"] += cloud["speed"] * dt
        if cloud["x"] > SCREEN_W + 100:
            cloud["x"] = -150

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    global font, current_time

    # Get colors for current time
    sky_top = get_sky_color(current_time)
    sky_bottom = get_sky_bottom_color(current_time)

    # Draw sky gradient
    draw_gradient_sky(sky_top, sky_bottom)

    # Draw stars at night
    hour = current_time / 60
    if hour < 6 or hour > 19:
        star_alpha = 255 if hour < 5 or hour > 21 else int(lerp(0, 255, abs(hour - 12) / 9))
        draw_stars(star_alpha)

    # Get celestial body positions
    (sun_x, sun_y), (moon_x, moon_y) = get_sun_moon_pos(current_time)

    # Draw sun and moon
    sun_color = get_sun_color(current_time)
    moon_alpha = get_moon_alpha(current_time)

    draw_sun(sun_x, sun_y, sun_color)
    draw_moon(moon_x, moon_y, moon_alpha)

    # Draw clouds (with varying opacity based on time)
    cloud_alpha = 200 if 6 < hour < 20 else 100
    for cloud in clouds:
        draw_cloud(cloud["x"], cloud["y"], cloud["w"], cloud["h"], cloud_alpha)

    # Draw mountains (covers sun/moon at horizon)
    draw_mountains(current_time)

    # Draw ground
    draw_ground(current_time)

    # Draw UI panel
    if font is not None:
        pgfx.rect_fill(5, 5, 180, 90, pgfx.Color(0, 0, 0, 180))
        pgfx.text(font, f"FPS: {pgfx.fps()}", 15, 15, pgfx.Color(255, 255, 255))
        pgfx.text(font, f"Time: {time_to_string(current_time)}", 15, 40, pgfx.Color(255, 255, 255))
        pgfx.text(font, f"{get_time_of_day(current_time)}", 15, 65, pgfx.Color(200, 200, 200))


print("Day/Night Cycle Demo")
print("  LEFT/RIGHT - Change time (hold to fast forward)")
print("  ESC - Exit")
print()

pgfx.run(update, render, on_ready=on_ready)
