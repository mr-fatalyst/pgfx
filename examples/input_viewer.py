"""Input viewer - displays all pressed keys and mouse state."""

import os

import pgfx

SCREEN_W, SCREEN_H = 800, 600

pgfx.init(SCREEN_W, SCREEN_H, "Input Viewer")

# Map key codes to names
KEY_NAMES = {
    pgfx.KEY_A: "A",
    pgfx.KEY_B: "B",
    pgfx.KEY_C: "C",
    pgfx.KEY_D: "D",
    pgfx.KEY_E: "E",
    pgfx.KEY_F: "F",
    pgfx.KEY_G: "G",
    pgfx.KEY_H: "H",
    pgfx.KEY_I: "I",
    pgfx.KEY_J: "J",
    pgfx.KEY_K: "K",
    pgfx.KEY_L: "L",
    pgfx.KEY_M: "M",
    pgfx.KEY_N: "N",
    pgfx.KEY_O: "O",
    pgfx.KEY_P: "P",
    pgfx.KEY_Q: "Q",
    pgfx.KEY_R: "R",
    pgfx.KEY_S: "S",
    pgfx.KEY_T: "T",
    pgfx.KEY_U: "U",
    pgfx.KEY_V: "V",
    pgfx.KEY_W: "W",
    pgfx.KEY_X: "X",
    pgfx.KEY_Y: "Y",
    pgfx.KEY_Z: "Z",
    pgfx.KEY_0: "0",
    pgfx.KEY_1: "1",
    pgfx.KEY_2: "2",
    pgfx.KEY_3: "3",
    pgfx.KEY_4: "4",
    pgfx.KEY_5: "5",
    pgfx.KEY_6: "6",
    pgfx.KEY_7: "7",
    pgfx.KEY_8: "8",
    pgfx.KEY_9: "9",
    pgfx.KEY_ESCAPE: "Escape",
    pgfx.KEY_SPACE: "Space",
    pgfx.KEY_ENTER: "Enter",
    pgfx.KEY_TAB: "Tab",
    pgfx.KEY_BACKSPACE: "Backspace",
    pgfx.KEY_LEFT: "Left",
    pgfx.KEY_RIGHT: "Right",
    pgfx.KEY_UP: "Up",
    pgfx.KEY_DOWN: "Down",
    pgfx.KEY_LSHIFT: "LShift",
    pgfx.KEY_RSHIFT: "RShift",
    pgfx.KEY_LCTRL: "LCtrl",
    pgfx.KEY_RCTRL: "RCtrl",
    pgfx.KEY_LALT: "LAlt",
    pgfx.KEY_RALT: "RAlt",
    pgfx.KEY_F1: "F1",
    pgfx.KEY_F2: "F2",
    pgfx.KEY_F3: "F3",
    pgfx.KEY_F4: "F4",
    pgfx.KEY_F5: "F5",
    pgfx.KEY_F6: "F6",
    pgfx.KEY_F7: "F7",
    pgfx.KEY_F8: "F8",
    pgfx.KEY_F9: "F9",
    pgfx.KEY_F10: "F10",
    pgfx.KEY_F11: "F11",
    pgfx.KEY_F12: "F12",
    pgfx.KEY_INSERT: "Insert",
    pgfx.KEY_DELETE: "Delete",
    pgfx.KEY_HOME: "Home",
    pgfx.KEY_END: "End",
    pgfx.KEY_PAGEUP: "PageUp",
    pgfx.KEY_PAGEDOWN: "PageDown",
    pgfx.KEY_MINUS: "-",
    pgfx.KEY_EQUAL: "=",
    pgfx.KEY_LBRACKET: "[",
    pgfx.KEY_RBRACKET: "]",
    pgfx.KEY_BACKSLASH: "\\",
    pgfx.KEY_SEMICOLON: ";",
    pgfx.KEY_QUOTE: "'",
    pgfx.KEY_BACKQUOTE: "`",
    pgfx.KEY_COMMA: ",",
    pgfx.KEY_PERIOD: ".",
    pgfx.KEY_SLASH: "/",
    pgfx.KEY_NUMPAD0: "Num0",
    pgfx.KEY_NUMPAD1: "Num1",
    pgfx.KEY_NUMPAD2: "Num2",
    pgfx.KEY_NUMPAD3: "Num3",
    pgfx.KEY_NUMPAD4: "Num4",
    pgfx.KEY_NUMPAD5: "Num5",
    pgfx.KEY_NUMPAD6: "Num6",
    pgfx.KEY_NUMPAD7: "Num7",
    pgfx.KEY_NUMPAD8: "Num8",
    pgfx.KEY_NUMPAD9: "Num9",
    pgfx.KEY_NUMPAD_ADD: "Num+",
    pgfx.KEY_NUMPAD_SUBTRACT: "Num-",
    pgfx.KEY_NUMPAD_MULTIPLY: "Num*",
    pgfx.KEY_NUMPAD_DIVIDE: "Num/",
    pgfx.KEY_NUMPAD_ENTER: "NumEnter",
    pgfx.KEY_NUMPAD_DECIMAL: "Num.",
    pgfx.KEY_CAPSLOCK: "CapsLock",
    pgfx.KEY_NUMLOCK: "NumLock",
    pgfx.KEY_SCROLLLOCK: "ScrollLock",
    pgfx.KEY_PRINTSCREEN: "PrtSc",
    pgfx.KEY_PAUSE: "Pause",
}

font = None
pressed_keys = []
key_history = []
MAX_HISTORY = 15


def on_ready():
    global font
    font = pgfx.font_load(os.path.join(os.path.dirname(__file__), "assets/font.ttf"), 20)


def update(dt):
    global pressed_keys, key_history

    # Check all keys
    pressed_keys = []
    for key_code, key_name in KEY_NAMES.items():
        if pgfx.key_down(key_code):
            pressed_keys.append(key_name)
        if pgfx.key_pressed(key_code):
            key_history.insert(0, key_name)
            if len(key_history) > MAX_HISTORY:
                key_history.pop()

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(25, 25, 35))

    if not font:
        return

    # Title
    pgfx.text(font, "Input Viewer - Press any key", 20, 20, pgfx.WHITE)

    # Currently held keys
    pgfx.text(font, "Held keys:", 20, 70, pgfx.YELLOW)
    if pressed_keys:
        keys_str = " + ".join(pressed_keys)
        pgfx.text(font, keys_str, 20, 100, pgfx.GREEN)
    else:
        pgfx.text(font, "(none)", 20, 100, pgfx.Color(100, 100, 100))

    # Key history
    pgfx.text(font, "History:", 20, 160, pgfx.YELLOW)
    for i, key_name in enumerate(key_history):
        alpha = 255 - i * 15
        color = pgfx.Color(200, 200, 255, alpha)
        pgfx.text(font, key_name, 20, 190 + i * 25, color)

    # Mouse info
    mx, my = pgfx.mouse_pos()
    pgfx.text(font, f"Mouse: ({mx}, {my})", SCREEN_W - 200, 20, pgfx.WHITE)

    # Mouse buttons
    mouse_btns = []
    if pgfx.mouse_down(pgfx.MOUSE_LEFT):
        mouse_btns.append("Left")
    if pgfx.mouse_down(pgfx.MOUSE_RIGHT):
        mouse_btns.append("Right")
    if pgfx.mouse_down(pgfx.MOUSE_MIDDLE):
        mouse_btns.append("Middle")

    if mouse_btns:
        pgfx.text(font, " + ".join(mouse_btns), SCREEN_W - 200, 50, pgfx.GREEN)

    # Mouse wheel
    wheel = pgfx.mouse_wheel()
    if wheel != 0:
        pgfx.text(font, f"Wheel: {wheel}", SCREEN_W - 200, 80, pgfx.CYAN)

    # Draw mouse cursor marker
    pgfx.circle_fill(mx, my, 5, pgfx.Color(255, 100, 100, 150))

    # Instructions
    pgfx.text(font, "ESC to exit", SCREEN_W - 150, SCREEN_H - 30, pgfx.Color(100, 100, 100))


pgfx.run(update, render, on_ready=on_ready)
