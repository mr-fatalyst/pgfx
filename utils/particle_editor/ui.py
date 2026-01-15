"""UI components for particle editor"""

import pgfx

# Colors
COL_BG = pgfx.Color(25, 25, 35)
COL_PANEL = pgfx.Color(40, 40, 50)
COL_SLIDER_BG = pgfx.Color(60, 60, 70)
COL_SLIDER_FG = pgfx.Color(100, 150, 255)
COL_SLIDER_HOVER = pgfx.Color(130, 180, 255)
COL_BTN = pgfx.Color(70, 70, 85)
COL_BTN_HOVER = pgfx.Color(90, 90, 110)
COL_BTN_ACTIVE = pgfx.Color(100, 150, 255)
COL_TEXT = pgfx.Color(220, 220, 220)
COL_TEXT_DIM = pgfx.Color(140, 140, 140)
COL_LABEL = pgfx.Color(180, 180, 180)


class Slider:
    """Horizontal slider with label and value display"""

    def __init__(self, x, y, w, h, label, min_val, max_val, value, fmt=".1f"):
        self.x = x
        self.y = y
        self.w = w
        self.h = h
        self.label = label
        self.min_val = min_val
        self.max_val = max_val
        self.value = value
        self.fmt = fmt
        self.dragging = False
        self.hovered = False
        self._drag_start_x = 0
        self._drag_start_value = 0

    def update(self, mx, my, mouse_down, mouse_pressed, shift_held=False):
        self.hovered = pgfx.point_in_rect(mx, my, self.x, self.y - 18, self.w, self.h + 18)

        if mouse_pressed and self.hovered:
            self.dragging = True
            self._drag_start_x = mx
            self._drag_start_value = self.value

        if not mouse_down:
            self.dragging = False

        if self.dragging:
            # Shift = fine control (10x slower)
            sensitivity = 0.1 if shift_held else 1.0
            delta_x = (mx - self._drag_start_x) * sensitivity
            ratio = delta_x / self.w
            new_value = self._drag_start_value + ratio * (self.max_val - self.min_val)
            self.value = max(self.min_val, min(self.max_val, new_value))

            # Update drag start for continuous fine control
            if shift_held:
                self._drag_start_x = mx
                self._drag_start_value = self.value

    def render(self, font):
        # Background
        pgfx.rect_fill(self.x, self.y, self.w, self.h, COL_SLIDER_BG)

        # Filled portion
        ratio = (self.value - self.min_val) / (self.max_val - self.min_val)
        fill_w = max(2, self.w * ratio)
        color = COL_SLIDER_HOVER if self.hovered or self.dragging else COL_SLIDER_FG
        pgfx.rect_fill(self.x, self.y, fill_w, self.h, color)

        # Label and value (skip if no label)
        if self.label:
            val_str = f"{self.value:{self.fmt}}"
            pgfx.text(font, f"{self.label}: {val_str}", self.x, self.y - 16, COL_LABEL)


class Button:
    """Clickable button with optional toggle mode"""

    def __init__(self, x, y, w, h, label, toggle=False):
        self.x = x
        self.y = y
        self.w = w
        self.h = h
        self.label = label
        self.toggle = toggle
        self.active = False
        self.hovered = False
        self.clicked = False

    def update(self, mx, my, mouse_pressed):
        self.hovered = pgfx.point_in_rect(mx, my, self.x, self.y, self.w, self.h)
        self.clicked = False

        if mouse_pressed and self.hovered:
            self.clicked = True
            if self.toggle:
                self.active = not self.active

    def render(self, font):
        if self.active:
            color = COL_BTN_ACTIVE
        elif self.hovered:
            color = COL_BTN_HOVER
        else:
            color = COL_BTN

        pgfx.rect_fill(self.x, self.y, self.w, self.h, color)

        # Center text vertically
        text_x = self.x + 8
        text_y = self.y + (self.h - 14) // 2
        pgfx.text(font, self.label, text_x, text_y, COL_TEXT)


class ColorPicker:
    """RGBA color picker with 4 sliders"""

    def __init__(self, x, y, label, color):
        self.x = x
        self.y = y
        self.label = label
        self.r = color[0]
        self.g = color[1]
        self.b = color[2]
        self.a = color[3]

        # Layout: "R [====] G [====] B [====] A [====] [preview]"
        # Each group: label (10px) + slider (50px) + gap (8px) = 68px per channel
        slider_w = 50
        slider_h = 12
        slider_y = y + 18
        group_w = 68  # width per RGBA group

        self._slider_r = Slider(x + 12, slider_y, slider_w, slider_h, "", 0, 255, self.r, ".0f")
        self._slider_g = Slider(
            x + 12 + group_w, slider_y, slider_w, slider_h, "", 0, 255, self.g, ".0f"
        )
        self._slider_b = Slider(
            x + 12 + group_w * 2, slider_y, slider_w, slider_h, "", 0, 255, self.b, ".0f"
        )
        self._slider_a = Slider(
            x + 12 + group_w * 3, slider_y, slider_w, slider_h, "", 0, 255, self.a, ".0f"
        )
        self.sliders = [self._slider_r, self._slider_g, self._slider_b, self._slider_a]

        # Label positions (left of each slider)
        self._label_positions = [
            (x, slider_y),
            (x + group_w, slider_y),
            (x + group_w * 2, slider_y),
            (x + group_w * 3, slider_y),
        ]
        self._slider_y = slider_y
        self._group_w = group_w

    def update(self, mx, my, mouse_down, mouse_pressed, shift_held=False):
        for slider in self.sliders:
            slider.update(mx, my, mouse_down, mouse_pressed, shift_held)
        self.r = int(self.sliders[0].value)
        self.g = int(self.sliders[1].value)
        self.b = int(self.sliders[2].value)
        self.a = int(self.sliders[3].value)

    def render(self, font):
        # Main label
        pgfx.text(font, self.label, self.x, self.y, COL_LABEL)

        # Draw R/G/B/A labels left of each slider
        labels = ["R", "G", "B", "A"]
        for i, lbl in enumerate(labels):
            lx, ly = self._label_positions[i]
            pgfx.text(font, lbl, lx, ly, COL_TEXT_DIM)

        # Render sliders
        for slider in self.sliders:
            slider.render(font)

        # Color preview square (after all sliders)
        preview_x = self.x + 12 + self._group_w * 4
        pgfx.rect_fill(
            preview_x, self._slider_y, 14, 12, pgfx.Color(self.r, self.g, self.b, self.a)
        )

    def get_tuple(self):
        return (self.r, self.g, self.b, self.a)

    def set_color(self, color):
        self.r, self.g, self.b, self.a = color
        self.sliders[0].value = self.r
        self.sliders[1].value = self.g
        self.sliders[2].value = self.b
        self.sliders[3].value = self.a
