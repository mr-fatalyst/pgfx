# pgfx

Lightweight 2D game library for Python, inspired by Haaf's Game Engine.

## Installation

```bash
pip install pgfx
```

Prebuilt wheels are available for Linux, macOS and Windows — no extra
dependencies needed.

### Building from source

Building (or installing from sdist) requires a Rust toolchain, `pkg-config`
and system audio/input headers on Linux:

```bash
# Debian/Ubuntu
sudo apt-get install libasound2-dev libudev-dev pkg-config

# Fedora/RHEL
sudo dnf install alsa-lib-devel systemd-devel pkgconf-pkg-config
```

Then `pip install .` (or `uv sync` + `maturin develop` for development).

## Quick Start

```python
import pgfx

pgfx.init(800, 600, "My Game")

x, y = 400, 300

def update(dt):
    global x, y
    speed = 200 * dt

    if pgfx.key_down(pgfx.KEY_LEFT):  x -= speed
    if pgfx.key_down(pgfx.KEY_RIGHT): x += speed
    if pgfx.key_down(pgfx.KEY_UP):    y -= speed
    if pgfx.key_down(pgfx.KEY_DOWN):  y += speed

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)

def render():
    pgfx.clear(pgfx.Color(20, 20, 40))
    pgfx.circle_fill(x, y, 20, pgfx.WHITE)

pgfx.run(update, render)
```

## Features

- **Sprites** — load, draw, rotate, scale, flip
- **Text** — TrueType fonts
- **Audio** — sounds and music
- **Input** — keyboard, mouse, gamepads
- **Particles** — configurable particle systems
- **Lighting** — ambient and point lights
- **Collision** — rects, circles, raycasts

## Examples

### Sprites

```python
ship = pgfx.sprite_load("ship.png")
pgfx.draw(ship, 100, 200)
pgfx.draw_ex(ship, 300, 200, rot=0.5, scale=2, alpha=0.8)
```

### Text

```python
font = pgfx.font_load("font.ttf", 24)
pgfx.text(font, f"FPS: {pgfx.fps()}", 10, 10, pgfx.WHITE)
```

### Audio

```python
laser = pgfx.sound_load("laser.wav")
pgfx.sound_play(laser)

music = pgfx.music_load("theme.ogg")
pgfx.music_play(music, loop=True)
```

### Input

```python
# Keyboard
if pgfx.key_pressed(pgfx.KEY_SPACE):
    shoot()

# Mouse
mx, my = pgfx.mouse_pos()
if pgfx.mouse_pressed(pgfx.MOUSE_LEFT):
    click(mx, my)

# Gamepad
if pgfx.gamepad_connected():
    move_x = pgfx.gamepad_axis(0, pgfx.GAMEPAD_AXIS_LX)
```

### Particles

```python
fire = pgfx.particles_create(
    primitive="circle_soft",
    emission_rate=100,
    lifetime_min=0.3, lifetime_max=0.8,
    speed_min=50, speed_max=100,
    start_color=(255, 200, 50, 255),
    end_color=(255, 50, 0, 0),
    start_size=10, end_size=2
)

# In update:
pgfx.particles_fire(fire, x, y)
pgfx.particles_update(fire, dt)

# In render:
pgfx.particles_render(fire)
```

### Lighting

```python
pgfx.set_ambient(pgfx.Color(30, 30, 50))
torch = pgfx.light_create(150, pgfx.Color(255, 200, 100))
pgfx.light_set_flicker(torch, 0.2, 3.0)

# In render:
pgfx.light_draw(torch, player_x, player_y)
```

---

## Notes

**Coordinates.** The whole API works in logical pixels — the units you pass
to `init()`. On HiDPI displays (Retina, 2x scale) the content is scaled up
automatically; `screen_size()` and `mouse_pos()` stay in logical pixels.

**Draw order.** Everything is drawn back-to-front by `z` (default 0, higher =
on top). Within the same `z`, all commands — sprites, primitives, text,
lights and particles — keep their call order.

**`clear(color)`** sets the frame's background color (the last call wins) —
it does not erase what was already drawn this frame.

**Camera.** `set_view(x, y, zoom, rot)` applies to every draw call issued
after it — sprites, primitives, text, particles and lights alike;
`reset_view()` switches back to screen space (HUD). The view resets at the
start of every frame, so call it in `render()` before drawing the world.
Which view a draw uses is decided by call order, not by `z`. Input stays in
screen space: converting `mouse_pos()` to world coordinates is up to you.
See `examples/camera.py`.

**Threading.** Call the pgfx API from the main thread only (`init`/`run` is an
OS requirement). Background `threading.Thread`s keep running during the game
loop — use them for pure Python work (networking, file I/O) and hand results
to `update` via `queue.Queue`. `multiprocessing` cannot be used with pgfx
objects: resource IDs are process-local.

**Audio latency.** The default audio buffer follows the output device. If you
get crackling audio (VMs, CI), set `PGFX_AUDIO_BUFFER=14400` (frames; 14400 ≈
300 ms at 48 kHz) before starting.

**Lighting.** `set_ambient` darkens the whole frame to the given color and
`light_draw` adds light on top: lights are accumulated additively into a
lightmap which then multiplies the rendered frame. Lighting applies to
everything drawn that frame (including text/HUD) regardless of `z`; when the
ambient color is white and no lights are drawn, the lighting passes are
skipped entirely.

## API Reference

### System (7)

| Function | Description |
|----------|-------------|
| `init(width, height, title, **opts)` | Initialize window. Options: `fullscreen`, `resizable`, `fps_limit` (frames per second cap, `0` = uncapped; VSync is always on) |
| `run(update_fn, render_fn, on_ready=None)` | Start game loop. `update_fn(dt)` returns `False` to quit; `on_ready()` is called once when the GPU is initialized. Exceptions raised in callbacks propagate out of `run()`. When `run()` returns, the window is closed, all resources are freed and pgfx is deinitialized — call `init()` to start again |
| `quit()` | Exit game loop (from a callback) or deinitialize pgfx (outside of `run()`) |
| `dt()` | Delta time in seconds |
| `fps()` | Current FPS (averaged over ~0.5s) |
| `time()` | Time since init |
| `screen_size()` | Current window size in logical pixels (tracks resize and fullscreen) |

### Input (11)

| Function | Description |
|----------|-------------|
| `key_down(key)` | Key is held |
| `key_pressed(key)` | Key just pressed |
| `key_released(key)` | Key just released |
| `mouse_pos()` | Returns `(x, y)` |
| `mouse_down(btn)` | Button is held |
| `mouse_pressed(btn)` | Button just pressed |
| `mouse_wheel()` | Wheel delta this frame (float; fractional for touchpads) |
| `gamepad_connected(idx=0)` | Gamepad connected |
| `gamepad_button(idx, btn)` | Button pressed |
| `gamepad_axis(idx, axis)` | Axis value (-1 to 1) |
| `gamepad_trigger(idx, trigger)` | Trigger value (0 to 1) |

### Texture (3)

| Function | Description |
|----------|-------------|
| `texture_load(path)` | Load texture, returns ID |
| `texture_free(tex)` | Free texture |
| `texture_size(tex)` | Returns `(width, height)` |

### Sprite (6)

| Function | Description |
|----------|-------------|
| `sprite_load(path)` | Load sprite from image |
| `sprite_create(tex, x, y, w, h)` | Create sprite from texture region |
| `sprite_sheet(path, cols, rows)` | Load sprite sheet, returns list of IDs |
| `sprite_set_origin(spr, ox, oy)` | Set origin point |
| `sprite_set_color(spr, color)` | Set tint color |
| `sprite_free(spr)` | Free sprite |

### Drawing (9)

| Function | Description |
|----------|-------------|
| `clear(color)` | Clear screen |
| `draw(spr, x, y)` | Draw sprite |
| `draw_ex(spr, x, y, rot=0, scale=1, alpha=1, flip_x=False, flip_y=False, scale_y=None)` | Draw with transform; `scale_y` (default: same as `scale`) enables non-uniform scaling |
| `rect_fill(x, y, w, h, color)` | Draw rectangle |
| `rect_fill_ex(x, y, w, h, color, rot=0)` | Draw a rotated rectangle **centered** on `(x, y)` — the anchor is the middle (like `circle_fill`), rotation is around it |
| `line(x1, y1, x2, y2, color, width=2)` | Draw line (centered on the segment) |
| `circle_fill(x, y, r, color)` | Draw circle |
| `set_view(x, y, zoom=1, rot=0)` | Camera: place world point `(x, y)` at the screen center for subsequent draws, zoomed and rotated around it |
| `reset_view()` | Back to screen space — for HUD/UI |

### Text (4)

| Function | Description |
|----------|-------------|
| `font_load(path, size, smooth=True)` | Load TTF font. `smooth=False` gives pixel-perfect rendering |
| `font_free(font)` | Free font |
| `text(font, string, x, y, color, z=0, align="left")` | Draw text. Any Unicode the font provides (glyphs are rasterized on demand), `\n` starts a new line, missing glyphs render as □. `align` ("left"/"center"/"right") sets what `x` anchors — each line aligns separately |
| `text_size(font, string)` | Measure without drawing: `(width, height)` of the block — widest line and line count × line height, matching `text()` exactly |

### Audio (12)

| Function | Description |
|----------|-------------|
| `sound_load(path)` | Load sound (WAV, OGG) |
| `sound_free(snd)` | Free sound |
| `sound_play(snd, volume=1, pan=0, loop=False)` | Play sound. `pan` is -1 (left) to 1 (right); non-zero pan mixes the source to mono |
| `sound_stop(snd)` | Stop sound |
| `music_load(path)` | Load music |
| `music_free(mus)` | Free music |
| `music_play(mus, loop=True)` | Play music |
| `music_stop(mus)` | Stop music |
| `music_pause(mus)` | Pause music |
| `music_resume(mus)` | Resume music |
| `set_master_volume(vol)` | Set master volume (0-1), applies to already-playing audio |
| `set_music_volume(vol)` | Set music volume (0-1), applies to the current track |

### Collision (9)

| Function | Description |
|----------|-------------|
| `collide_rects(x1, y1, w1, h1, x2, y2, w2, h2)` | Rectangle overlap |
| `collide_circles(x1, y1, r1, x2, y2, r2)` | Circle overlap |
| `collide_circle_rect(cx, cy, r, rx, ry, rw, rh)` | Circle-rect overlap |
| `point_in_rect(px, py, rx, ry, rw, rh)` | Point in rectangle |
| `point_in_circle(px, py, cx, cy, r)` | Point in circle |
| `raycast_rect(ox, oy, dx, dy, rx, ry, rw, rh)` | Ray-rect intersection, returns distance or `None` |
| `sprite_rect(spr, x, y)` | Get sprite bounds `(x, y, w, h)` |
| `collide_sprites(spr1, x1, y1, spr2, x2, y2)` | Sprite overlap |
| `point_in_sprite(px, py, spr, sx, sy)` | Point in sprite |

### Particles (12)

| Function | Description |
|----------|-------------|
| `particles_create(**params)` | Create particle system |
| `particles_load(path)` | Create particle system from a JSON file (as saved by `utils/particle_editor`): the same keys as `particles_create`, plus optional `primitive` or `sprite` — an image path resolved relative to the file |
| `particles_free(ps)` | Free particle system |
| `particles_set(ps, **params)` | Update parameters; the ones not passed keep their current values |
| `particles_fire(ps, x, y)` | Start emitting |
| `particles_emit(ps, x, y, count)` | Emit burst |
| `particles_stop(ps)` | Stop emitting |
| `particles_move_to(ps, x, y)` | Move emitter |
| `particles_update(ps, dt)` | Update (call in update) |
| `particles_render(ps)` | Render (call in render) |
| `particles_is_alive(ps)` | Any particles alive |
| `particles_count(ps)` | Active particle count |

**Particle parameters:** `primitive`, `emission_rate`, `lifetime_min`, `lifetime_max`, `speed_min`, `speed_max`, `direction`, `spread`, `gravity`, `start_color`, `end_color`, `start_size`, `end_size`, `max_particles`

### Lighting (7)

| Function | Description |
|----------|-------------|
| `set_ambient(color)` | Set ambient light (darkens the whole frame; white = lighting off) |
| `light_create(radius, color)` | Create light |
| `light_set_color(light, color)` | Change color |
| `light_set_intensity(light, intensity)` | Set intensity (0-1) |
| `light_set_flicker(light, amount, speed)` | Add flicker effect |
| `light_draw(light, x, y)` | Draw light |
| `light_free(light)` | Free light |

### Constants

**Colors:** `WHITE`, `BLACK`, `RED`, `GREEN`, `BLUE`, `YELLOW`, `CYAN`, `MAGENTA`, `TRANSPARENT`

**Keys:** `KEY_A`-`KEY_Z`, `KEY_0`-`KEY_9`, `KEY_F1`-`KEY_F12`, `KEY_LEFT`, `KEY_RIGHT`, `KEY_UP`, `KEY_DOWN`, `KEY_SPACE`, `KEY_ESCAPE`, `KEY_ENTER`, `KEY_TAB`, `KEY_BACKSPACE`, `KEY_LSHIFT`, `KEY_RSHIFT`, `KEY_LCTRL`, `KEY_RCTRL`, `KEY_LALT`, `KEY_RALT`

**Mouse:** `MOUSE_LEFT`, `MOUSE_RIGHT`, `MOUSE_MIDDLE`

**Gamepad buttons:** `GAMEPAD_A`, `GAMEPAD_B`, `GAMEPAD_X`, `GAMEPAD_Y`, `GAMEPAD_LB`, `GAMEPAD_RB`, `GAMEPAD_LT`, `GAMEPAD_RT`, `GAMEPAD_BACK`, `GAMEPAD_START`, `GAMEPAD_GUIDE`, `GAMEPAD_LSTICK`, `GAMEPAD_RSTICK`, `GAMEPAD_DPAD_UP`, `GAMEPAD_DPAD_DOWN`, `GAMEPAD_DPAD_LEFT`, `GAMEPAD_DPAD_RIGHT`

**Gamepad axes:** `GAMEPAD_AXIS_LX`, `GAMEPAD_AXIS_LY`, `GAMEPAD_AXIS_RX`, `GAMEPAD_AXIS_RY`

**Gamepad triggers:** `GAMEPAD_TRIGGER_L`, `GAMEPAD_TRIGGER_R`

## License

MIT
