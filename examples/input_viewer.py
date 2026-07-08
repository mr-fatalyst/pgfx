"""Input viewer: every held key, press history, mouse state.

A small debugging tool — press anything and watch it show up.
"""

import os

import pgfx

SCREEN_W, SCREEN_H = 800, 600
DIM = pgfx.Color(150, 155, 165)
MAX_HISTORY = 15

pgfx.init(SCREEN_W, SCREEN_H, "pgfx input viewer")

# Key-code -> name map, generated from the pgfx constants
KEY_NAMES = {}
for _ch in "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789":
    KEY_NAMES[getattr(pgfx, f"KEY_{_ch}")] = _ch
for _i in range(1, 13):
    KEY_NAMES[getattr(pgfx, f"KEY_F{_i}")] = f"F{_i}"
for _i in range(10):
    KEY_NAMES[getattr(pgfx, f"KEY_NUMPAD{_i}")] = f"Num{_i}"
for _name, _label in {
    "ESCAPE": "Esc",
    "SPACE": "Space",
    "ENTER": "Enter",
    "TAB": "Tab",
    "BACKSPACE": "Backspace",
    "LEFT": "Left",
    "RIGHT": "Right",
    "UP": "Up",
    "DOWN": "Down",
    "LSHIFT": "LShift",
    "RSHIFT": "RShift",
    "LCTRL": "LCtrl",
    "RCTRL": "RCtrl",
    "LALT": "LAlt",
    "RALT": "RAlt",
    "INSERT": "Ins",
    "DELETE": "Del",
    "HOME": "Home",
    "END": "End",
    "PAGEUP": "PgUp",
    "PAGEDOWN": "PgDn",
    "MINUS": "-",
    "EQUAL": "=",
    "LBRACKET": "[",
    "RBRACKET": "]",
    "BACKSLASH": "\\",
    "SEMICOLON": ";",
    "QUOTE": "'",
    "BACKQUOTE": "`",
    "COMMA": ",",
    "PERIOD": ".",
    "SLASH": "/",
    "NUMPAD_ADD": "Num+",
    "NUMPAD_SUBTRACT": "Num-",
    "NUMPAD_MULTIPLY": "Num*",
    "NUMPAD_DIVIDE": "Num/",
    "NUMPAD_ENTER": "NumEnter",
    "NUMPAD_DECIMAL": "Num.",
    "CAPSLOCK": "CapsLock",
    "NUMLOCK": "NumLock",
    "SCROLLLOCK": "ScrollLock",
    "PRINTSCREEN": "PrtSc",
    "PAUSE": "Pause",
}.items():
    KEY_NAMES[getattr(pgfx, f"KEY_{_name}")] = _label

font = None
held = []
history = []


def on_ready():
    global font
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 20)


def update(dt):
    global held
    held = []
    for code, name in KEY_NAMES.items():
        if pgfx.key_down(code):
            held.append(name)
        if pgfx.key_pressed(code):
            history.insert(0, name)
            del history[MAX_HISTORY:]
    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))
    if not font:
        return

    pgfx.text(font, "held keys:", 20, 30, pgfx.YELLOW)
    pgfx.text(font, " + ".join(held) if held else "(none)", 20, 60, pgfx.GREEN if held else DIM)

    pgfx.text(font, "history:", 20, 120, pgfx.YELLOW)
    for i, name in enumerate(history):
        pgfx.text(font, name, 20, 150 + i * 25, pgfx.Color(200, 200, 255, 255 - i * 15))

    mx, my = pgfx.mouse_pos()
    buttons = [
        label
        for label, btn in (
            ("Left", pgfx.MOUSE_LEFT),
            ("Right", pgfx.MOUSE_RIGHT),
            ("Middle", pgfx.MOUSE_MIDDLE),
        )
        if pgfx.mouse_down(btn)
    ]
    pgfx.text(font, f"mouse: {mx}, {my}", SCREEN_W - 40, 30, pgfx.WHITE, align="right")
    pgfx.text(
        font,
        " + ".join(buttons) if buttons else "(no buttons)",
        SCREEN_W - 40,
        60,
        pgfx.GREEN if buttons else DIM,
        align="right",
    )
    wheel = pgfx.mouse_wheel()
    if wheel:
        pgfx.text(font, f"wheel: {wheel:+.1f}", SCREEN_W - 40, 90, pgfx.CYAN, align="right")
    pgfx.circle_fill(mx, my, 5, pgfx.Color(255, 100, 100, 150))

    pgfx.text(
        font, "PRESS ANYTHING — ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center"
    )


pgfx.run(update, render, on_ready=on_ready)
