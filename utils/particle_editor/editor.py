"""Particle Editor logic"""

import json
import os

import pgfx

from .ui import COL_BG, COL_LABEL, COL_PANEL, COL_TEXT, COL_TEXT_DIM, Button, ColorPicker, Slider

# Layout constants
SCREEN_W, SCREEN_H = 1280, 720
PANEL_W = 320
PREVIEW_CENTER_X = PANEL_W + (SCREEN_W - PANEL_W) // 2
PREVIEW_CENTER_Y = SCREEN_H // 2

# Font path (relative to this file)
ASSETS_DIR = os.path.join(os.path.dirname(__file__), "assets")
FONT_PATH = os.path.join(ASSETS_DIR, "font.ttf")
FONT_SIZE = 10


class ParticleEditor:
    """Main particle editor class"""

    def __init__(self):
        self.font = None
        self.ps = None
        self.current_primitive = "circle_soft"  # circle, circle_soft, square

        # Emit mode: 0=continuous, 1=click, 2=interval
        self.emit_mode = 0
        self.emit_interval = 1.0
        self.emit_timer = 0

        # File
        self.current_file = None
        self.status_message = ""
        self.status_timer = 0

        # UI elements
        self.sliders = {}
        self.buttons = {}
        self.color_pickers = {}
        self.mode_buttons = []
        self.interval_slider = None

    def on_ready(self):
        """Called when GPU is ready"""
        self.font = pgfx.font_load(FONT_PATH, FONT_SIZE)
        self.setup_ui()
        self.create_particle_system()
        pgfx.particles_fire(self.ps, PREVIEW_CENTER_X, PREVIEW_CENTER_Y)

    def setup_ui(self):
        x = 10
        w = 295
        h = 12
        gap = 34
        btn_w = 95
        btn_h = 26

        # Shape buttons (after header with gap)
        prim_y = 55
        self.primitive_buttons = [
            Button(x, prim_y, btn_w, btn_h, "Circle", toggle=True),
            Button(x + 100, prim_y, btn_w, btn_h, "Soft", toggle=True),
            Button(x + 200, prim_y, btn_w, btn_h, "Square", toggle=True),
        ]
        self.primitive_buttons[1].active = True  # Default to circle_soft
        self.primitive_types = ["circle", "circle_soft", "square"]

        # Parameter sliders (11 штук)
        y = 100
        self.sliders = {
            "emission_rate": Slider(x, y, w, h, "Emission Rate", 0, 200, 50, ".0f"),
            "lifetime_min": Slider(x, y + gap, w, h, "Lifetime Min", 0.1, 5, 0.5, ".2f"),
            "lifetime_max": Slider(x, y + gap * 2, w, h, "Lifetime Max", 0.1, 5, 1.5, ".2f"),
            "speed_min": Slider(x, y + gap * 3, w, h, "Speed Min", 0, 500, 50, ".0f"),
            "speed_max": Slider(x, y + gap * 4, w, h, "Speed Max", 0, 500, 150, ".0f"),
            "direction": Slider(x, y + gap * 5, w, h, "Direction", -3.14, 3.14, -1.57, ".2f"),
            "spread": Slider(x, y + gap * 6, w, h, "Spread", 0, 6.28, 0.5, ".2f"),
            "gravity_x": Slider(x, y + gap * 7, w, h, "Gravity X", -500, 500, 0, ".0f"),
            "gravity_y": Slider(x, y + gap * 8, w, h, "Gravity Y", -500, 500, 100, ".0f"),
            "start_size": Slider(x, y + gap * 9, w, h, "Start Size", 1, 50, 15, ".0f"),
            "end_size": Slider(x, y + gap * 10, w, h, "End Size", 0, 50, 3, ".0f"),
        }

        # Color pickers
        cy = y + gap * 11 + 10
        self.color_pickers = {
            "start_color": ColorPicker(x, cy, "Start Color", (255, 200, 50, 255)),
            "end_color": ColorPicker(x, cy + 45, "End Color", (255, 50, 0, 0)),
        }

        # Emit mode buttons
        mode_y = cy + 100
        self.mode_buttons = [
            Button(x, mode_y, btn_w, btn_h, "Continuous", toggle=True),
            Button(x + 100, mode_y, btn_w, btn_h, "On Click", toggle=True),
            Button(x + 200, mode_y, btn_w, btn_h, "Interval", toggle=True),
        ]
        self.mode_buttons[0].active = True

        # Interval slider
        self.interval_slider = Slider(x, mode_y + 40, w, h, "Interval", 0.1, 3.0, 1.0, ".1f")

        # File buttons at bottom
        btn_y = SCREEN_H - 20 - btn_h
        self.buttons = {
            "new": Button(x, btn_y, btn_w, btn_h, "New"),
            "load": Button(x + 100, btn_y, btn_w, btn_h, "Load"),
            "save": Button(x + 200, btn_y, btn_w, btn_h, "Save"),
        }

    def create_particle_system(self):
        if self.ps is not None:
            pgfx.particles_free(self.ps)

        self.ps = pgfx.particles_create(
            primitive=self.current_primitive,
            emission_rate=self.sliders["emission_rate"].value,
            lifetime_min=self.sliders["lifetime_min"].value,
            lifetime_max=self.sliders["lifetime_max"].value,
            speed_min=self.sliders["speed_min"].value,
            speed_max=self.sliders["speed_max"].value,
            direction=self.sliders["direction"].value,
            spread=self.sliders["spread"].value,
            gravity=(self.sliders["gravity_x"].value, self.sliders["gravity_y"].value),
            start_color=self.color_pickers["start_color"].get_tuple(),
            end_color=self.color_pickers["end_color"].get_tuple(),
            start_size=self.sliders["start_size"].value,
            end_size=self.sliders["end_size"].value,
            max_particles=2000,
        )

    def update_particle_params(self):
        if self.ps is None:
            return

        pgfx.particles_set(
            self.ps,
            emission_rate=self.sliders["emission_rate"].value,
            lifetime_min=self.sliders["lifetime_min"].value,
            lifetime_max=self.sliders["lifetime_max"].value,
            speed_min=self.sliders["speed_min"].value,
            speed_max=self.sliders["speed_max"].value,
            direction=self.sliders["direction"].value,
            spread=self.sliders["spread"].value,
            gravity=(self.sliders["gravity_x"].value, self.sliders["gravity_y"].value),
            start_color=self.color_pickers["start_color"].get_tuple(),
            end_color=self.color_pickers["end_color"].get_tuple(),
            start_size=self.sliders["start_size"].value,
            end_size=self.sliders["end_size"].value,
        )

    def get_params_dict(self):
        return {
            "emission_rate": self.sliders["emission_rate"].value,
            "lifetime_min": self.sliders["lifetime_min"].value,
            "lifetime_max": self.sliders["lifetime_max"].value,
            "speed_min": self.sliders["speed_min"].value,
            "speed_max": self.sliders["speed_max"].value,
            "direction": self.sliders["direction"].value,
            "spread": self.sliders["spread"].value,
            "gravity_x": self.sliders["gravity_x"].value,
            "gravity_y": self.sliders["gravity_y"].value,
            "start_color": list(self.color_pickers["start_color"].get_tuple()),
            "end_color": list(self.color_pickers["end_color"].get_tuple()),
            "start_size": self.sliders["start_size"].value,
            "end_size": self.sliders["end_size"].value,
        }

    def load_params(self, params):
        for key in [
            "emission_rate",
            "lifetime_min",
            "lifetime_max",
            "speed_min",
            "speed_max",
            "direction",
            "spread",
            "gravity_x",
            "gravity_y",
            "start_size",
            "end_size",
        ]:
            if key in params and key in self.sliders:
                self.sliders[key].value = params[key]

        if "start_color" in params:
            self.color_pickers["start_color"].set_color(tuple(params["start_color"]))
        if "end_color" in params:
            self.color_pickers["end_color"].set_color(tuple(params["end_color"]))

    def reset_to_defaults(self):
        defaults = {
            "emission_rate": 50,
            "lifetime_min": 0.5,
            "lifetime_max": 1.5,
            "speed_min": 50,
            "speed_max": 150,
            "direction": -1.57,
            "spread": 0.5,
            "gravity_x": 0,
            "gravity_y": 100,
            "start_size": 15,
            "end_size": 3,
            "start_color": [255, 200, 50, 255],
            "end_color": [255, 50, 0, 0],
        }
        self.load_params(defaults)
        self.show_status("Reset to defaults")

    def show_status(self, message, duration=2.0):
        self.status_message = message
        self.status_timer = duration

    def save_to_file(self, filepath):
        try:
            params = self.get_params_dict()
            with open(filepath, "w") as f:
                json.dump(params, f, indent=2)
            self.current_file = filepath
            self.show_status(f"Saved: {os.path.basename(filepath)}")
        except Exception as e:
            self.show_status(f"Error: {e}")

    def load_from_file(self, filepath):
        try:
            with open(filepath, "r") as f:
                params = json.load(f)
            self.load_params(params)
            self.current_file = filepath
            self.show_status(f"Loaded: {os.path.basename(filepath)}")
        except FileNotFoundError:
            self.show_status("File not found")
        except Exception as e:
            self.show_status(f"Error: {e}")

    def update(self, dt):
        if not self.font:
            return True

        # Input
        mx, my = pgfx.mouse_pos()
        mouse_down = pgfx.mouse_down(pgfx.MOUSE_LEFT)
        mouse_pressed = pgfx.mouse_pressed(pgfx.MOUSE_LEFT)
        shift_held = pgfx.key_down(pgfx.KEY_LSHIFT) or pgfx.key_down(pgfx.KEY_RSHIFT)

        # Update status timer
        if self.status_timer > 0:
            self.status_timer -= dt

        # Update sliders
        for slider in self.sliders.values():
            slider.update(mx, my, mouse_down, mouse_pressed, shift_held)

        # Update color pickers
        for picker in self.color_pickers.values():
            picker.update(mx, my, mouse_down, mouse_pressed, shift_held)

        # Update buttons
        for btn in self.buttons.values():
            btn.update(mx, my, mouse_pressed)

        for btn in self.mode_buttons:
            btn.update(mx, my, mouse_pressed)

        for btn in self.primitive_buttons:
            btn.update(mx, my, mouse_pressed)

        self.interval_slider.update(mx, my, mouse_down, mouse_pressed, shift_held)

        # Mode button radio behavior
        for i, btn in enumerate(self.mode_buttons):
            if btn.clicked:
                self.emit_mode = i
                for j, other in enumerate(self.mode_buttons):
                    other.active = j == i

        # Primitive button radio behavior
        for i, btn in enumerate(self.primitive_buttons):
            if btn.clicked:
                self.current_primitive = self.primitive_types[i]
                for j, other in enumerate(self.primitive_buttons):
                    other.active = j == i
                # Recreate particle system with new primitive
                self.create_particle_system()
                pgfx.particles_fire(self.ps, PREVIEW_CENTER_X, PREVIEW_CENTER_Y)

        # File operations
        if self.buttons["new"].clicked:
            self.reset_to_defaults()
            self.current_file = None

        if self.buttons["save"].clicked:
            filepath = "particle_effect.json"
            self.save_to_file(filepath)

        if self.buttons["load"].clicked:
            filepath = "particle_effect.json"
            self.load_from_file(filepath)

        # Update particle params
        self.update_particle_params()

        # Emission logic
        if self.emit_mode == 0:  # Continuous
            pgfx.particles_fire(self.ps, PREVIEW_CENTER_X, PREVIEW_CENTER_Y)
        elif self.emit_mode == 1:  # On click
            pgfx.particles_stop(self.ps)
            if mouse_pressed and mx > PANEL_W:
                pgfx.particles_emit(self.ps, mx, my, 50)
        elif self.emit_mode == 2:  # Interval
            pgfx.particles_stop(self.ps)
            self.emit_timer += dt
            if self.emit_timer >= self.interval_slider.value:
                pgfx.particles_emit(self.ps, PREVIEW_CENTER_X, PREVIEW_CENTER_Y, 50)
                self.emit_timer = 0

        # Update particles
        pgfx.particles_update(self.ps, dt)

        return not pgfx.key_pressed(pgfx.KEY_ESCAPE)

    def render(self):
        pgfx.clear(COL_BG)

        if not self.font:
            return

        # Panel background
        pgfx.rect_fill(0, 0, PANEL_W, SCREEN_H, COL_PANEL)

        # Header
        pgfx.text(self.font, "PARTICLE EDITOR", 10, 12, COL_TEXT)
        pgfx.text(self.font, f"Count: {pgfx.particles_count(self.ps)}", 180, 12, COL_TEXT_DIM)
        pgfx.text(self.font, f"FPS: {pgfx.fps()}", 260, 12, COL_TEXT_DIM)

        # Separator
        pgfx.line(10, 30, PANEL_W - 10, 30, COL_TEXT_DIM)

        # Shape buttons (top)
        pgfx.text(self.font, "Shape:", 10, self.primitive_buttons[0].y - 16, COL_LABEL)
        for btn in self.primitive_buttons:
            btn.render(self.font)

        # Sliders
        for slider in self.sliders.values():
            slider.render(self.font)

        # Color pickers
        for picker in self.color_pickers.values():
            picker.render(self.font)

        # File buttons
        for btn in self.buttons.values():
            btn.render(self.font)

        # Emit mode label and buttons
        pgfx.text(self.font, "Emit Mode:", 10, self.mode_buttons[0].y - 16, COL_LABEL)
        for btn in self.mode_buttons:
            btn.render(self.font)

        # Interval slider (only in interval mode)
        if self.emit_mode == 2:
            self.interval_slider.render(self.font)

        # Panel separator
        pgfx.line(PANEL_W, 0, PANEL_W, SCREEN_H, pgfx.Color(60, 60, 70))

        # Preview header
        pgfx.text(self.font, "PREVIEW", PREVIEW_CENTER_X - 25, 12, COL_TEXT_DIM)

        # Crosshair
        cross_color = pgfx.Color(80, 80, 90)
        pgfx.line(
            PREVIEW_CENTER_X - 15,
            PREVIEW_CENTER_Y,
            PREVIEW_CENTER_X + 15,
            PREVIEW_CENTER_Y,
            cross_color,
        )
        pgfx.line(
            PREVIEW_CENTER_X,
            PREVIEW_CENTER_Y - 15,
            PREVIEW_CENTER_X,
            PREVIEW_CENTER_Y + 15,
            cross_color,
        )

        # Particles
        pgfx.particles_render(self.ps)

        # Status message
        if self.status_timer > 0:
            alpha = min(255, int(self.status_timer * 255))
            pgfx.text(
                self.font,
                self.status_message,
                PANEL_W + 20,
                SCREEN_H - 45,
                pgfx.Color(100, 255, 100, alpha),
            )

        # Help
        pgfx.text(
            self.font,
            "Shift+drag for fine control | ESC to exit",
            PANEL_W + 20,
            SCREEN_H - 25,
            COL_TEXT_DIM,
        )
