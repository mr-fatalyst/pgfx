use gilrs::{Axis, Button, Gilrs};
use pyo3::prelude::*;
use std::collections::HashSet;
use winit::event::MouseButton;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Input state for keyboard and mouse
pub struct InputState {
    // Keyboard
    pub keys_down: HashSet<u32>,     // currently held
    pub keys_pressed: HashSet<u32>,  // pressed this frame
    pub keys_released: HashSet<u32>, // released this frame

    // Mouse
    pub mouse_pos: (i32, i32),
    pub mouse_down: [bool; 3],     // left, right, middle
    pub mouse_pressed: [bool; 3],  // pressed this frame
    pub mouse_released: [bool; 3], // released this frame
    pub mouse_wheel: i32,          // wheel delta this frame
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            mouse_pos: (0, 0),
            mouse_down: [false; 3],
            mouse_pressed: [false; 3],
            mouse_released: [false; 3],
            mouse_wheel: 0,
        }
    }

    /// Clear per-frame state (called at the start of each frame)
    pub fn clear_frame_state(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_pressed = [false; 3];
        self.mouse_released = [false; 3];
        self.mouse_wheel = 0;
    }

    /// Handle key press
    pub fn on_key_down(&mut self, key: u32) {
        if !self.keys_down.contains(&key) {
            self.keys_pressed.insert(key);
            self.keys_down.insert(key);
        }
    }

    /// Handle key release
    pub fn on_key_up(&mut self, key: u32) {
        self.keys_released.insert(key);
        self.keys_down.remove(&key);
    }

    /// Handle mouse move
    pub fn on_mouse_move(&mut self, x: i32, y: i32) {
        self.mouse_pos = (x, y);
    }

    /// Handle mouse button press
    pub fn on_mouse_down(&mut self, button: usize) {
        if button < 3 && !self.mouse_down[button] {
            self.mouse_pressed[button] = true;
            self.mouse_down[button] = true;
        }
    }

    /// Handle mouse button release
    pub fn on_mouse_up(&mut self, button: usize) {
        if button < 3 {
            self.mouse_released[button] = true;
            self.mouse_down[button] = false;
        }
    }

    /// Handle mouse wheel
    pub fn on_mouse_wheel(&mut self, delta: i32) {
        self.mouse_wheel += delta;
    }
}

/// State for a single gamepad
#[derive(Clone)]
pub struct GamepadState {
    pub connected: bool,
    pub buttons: [bool; 20], // Support up to 20 buttons
    pub axes: [f32; 6],      // Left stick (2), right stick (2), triggers (2)
}

impl GamepadState {
    fn new() -> Self {
        Self {
            connected: false,
            buttons: [false; 20],
            axes: [0.0; 6],
        }
    }

    fn disconnect(&mut self) {
        self.connected = false;
        self.buttons = [false; 20];
        self.axes = [0.0; 6];
    }
}

/// Gamepad manager using gilrs
pub struct GamepadManager {
    gilrs: Gilrs,
    gamepads: [GamepadState; 4], // Support up to 4 gamepads
}

impl GamepadManager {
    pub fn new() -> Result<Self, String> {
        let gilrs = Gilrs::new().map_err(|e| format!("Failed to initialize gilrs: {}", e))?;
        Ok(Self {
            gilrs,
            gamepads: [
                GamepadState::new(),
                GamepadState::new(),
                GamepadState::new(),
                GamepadState::new(),
            ],
        })
    }

    /// Poll gamepad events and update state
    pub fn poll(&mut self) {
        // Process all pending events
        while let Some(event) = self.gilrs.next_event() {
            use gilrs::EventType;

            // Find the gamepad slot (0-3) for this gamepad ID
            if let Some(slot) = self.find_gamepad_slot(event.id) {
                match event.event {
                    EventType::ButtonPressed(button, _) => {
                        if let Some(btn_idx) = map_button(button) {
                            if btn_idx < 20 {
                                self.gamepads[slot].buttons[btn_idx] = true;
                            }
                        }
                    }
                    EventType::ButtonReleased(button, _) => {
                        if let Some(btn_idx) = map_button(button) {
                            if btn_idx < 20 {
                                self.gamepads[slot].buttons[btn_idx] = false;
                            }
                        }
                    }
                    EventType::AxisChanged(axis, value, _) => {
                        if let Some(axis_idx) = map_axis(axis) {
                            if axis_idx < 6 {
                                self.gamepads[slot].axes[axis_idx] = value;
                            }
                        }
                    }
                    EventType::Connected => {
                        self.gamepads[slot].connected = true;
                    }
                    EventType::Disconnected => {
                        self.gamepads[slot].disconnect();
                    }
                    _ => {}
                }
            }
        }
    }

    /// Find a slot for the given gamepad ID (0-3)
    /// Returns the existing slot if already assigned, or finds a free slot
    fn find_gamepad_slot(&mut self, gamepad_id: gilrs::GamepadId) -> Option<usize> {
        // First check if this gamepad is already assigned a slot
        for gamepad in self.gamepads.iter() {
            if gamepad.connected {
                // We need to track gamepad IDs to slots
                // For now, use a simple approach: map gamepad index to slot
                if self.gilrs.gamepad(gamepad_id).is_connected() {
                    // Just use the first 4 connected gamepads
                    let connected_count = self
                        .gilrs
                        .gamepads()
                        .filter(|(_, gp)| gp.is_connected())
                        .take_while(|(id, _)| *id != gamepad_id)
                        .count();
                    return Some(connected_count.min(3));
                }
            }
        }

        // Find first free slot
        for (slot, gamepad) in self.gamepads.iter().enumerate() {
            if !gamepad.connected {
                return Some(slot);
            }
        }

        // All slots full, use first slot
        Some(0)
    }

    pub fn get_gamepad(&self, idx: usize) -> Option<&GamepadState> {
        if idx < 4 {
            Some(&self.gamepads[idx])
        } else {
            None
        }
    }
}

/// Map gilrs button to our button index
fn map_button(button: Button) -> Option<usize> {
    match button {
        Button::South => Some(0),         // A on Xbox, Cross on PS
        Button::East => Some(1),          // B on Xbox, Circle on PS
        Button::West => Some(2),          // X on Xbox, Square on PS
        Button::North => Some(3),         // Y on Xbox, Triangle on PS
        Button::LeftTrigger => Some(4),   // L1/LB
        Button::RightTrigger => Some(5),  // R1/RB
        Button::LeftTrigger2 => Some(6),  // L2/LT (as button)
        Button::RightTrigger2 => Some(7), // R2/RT (as button)
        Button::Select => Some(8),        // Select/Back/Share
        Button::Start => Some(9),         // Start/Options
        Button::LeftThumb => Some(10),    // L3
        Button::RightThumb => Some(11),   // R3
        Button::DPadUp => Some(12),
        Button::DPadDown => Some(13),
        Button::DPadLeft => Some(14),
        Button::DPadRight => Some(15),
        Button::Mode => Some(16), // Guide/Home/PS button
        _ => None,
    }
}

/// Map gilrs axis to our axis index
/// Axes: 0=LeftX, 1=LeftY, 2=RightX, 3=RightY, 4=LeftTrigger, 5=RightTrigger
fn map_axis(axis: Axis) -> Option<usize> {
    match axis {
        Axis::LeftStickX => Some(0),
        Axis::LeftStickY => Some(1),
        Axis::RightStickX => Some(2),
        Axis::RightStickY => Some(3),
        Axis::LeftZ => Some(4),  // Left trigger
        Axis::RightZ => Some(5), // Right trigger
        _ => None,
    }
}

/// Convert winit KeyCode to pgfx key constant
pub fn winit_key_to_pgfx(key: KeyCode) -> Option<u32> {
    use KeyCode::*;

    match key {
        // Letters
        KeyA => Some(0),
        KeyB => Some(1),
        KeyC => Some(2),
        KeyD => Some(3),
        KeyE => Some(4),
        KeyF => Some(5),
        KeyG => Some(6),
        KeyH => Some(7),
        KeyI => Some(8),
        KeyJ => Some(9),
        KeyK => Some(10),
        KeyL => Some(11),
        KeyM => Some(12),
        KeyN => Some(13),
        KeyO => Some(14),
        KeyP => Some(15),
        KeyQ => Some(16),
        KeyR => Some(17),
        KeyS => Some(18),
        KeyT => Some(19),
        KeyU => Some(20),
        KeyV => Some(21),
        KeyW => Some(22),
        KeyX => Some(23),
        KeyY => Some(24),
        KeyZ => Some(25),

        // Numbers
        Digit0 => Some(26),
        Digit1 => Some(27),
        Digit2 => Some(28),
        Digit3 => Some(29),
        Digit4 => Some(30),
        Digit5 => Some(31),
        Digit6 => Some(32),
        Digit7 => Some(33),
        Digit8 => Some(34),
        Digit9 => Some(35),

        // Special keys
        Escape => Some(36),
        Space => Some(37),
        Enter => Some(38),
        Tab => Some(39),
        Backspace => Some(40),

        // Arrow keys
        ArrowLeft => Some(41),
        ArrowRight => Some(42),
        ArrowUp => Some(43),
        ArrowDown => Some(44),

        // Modifiers
        ShiftLeft => Some(45),
        ShiftRight => Some(46),
        ControlLeft => Some(47),
        ControlRight => Some(48),
        AltLeft => Some(49),
        AltRight => Some(50),

        // Function keys
        F1 => Some(51),
        F2 => Some(52),
        F3 => Some(53),
        F4 => Some(54),
        F5 => Some(55),
        F6 => Some(56),
        F7 => Some(57),
        F8 => Some(58),
        F9 => Some(59),
        F10 => Some(60),
        F11 => Some(61),
        F12 => Some(62),

        // Navigation keys
        Insert => Some(63),
        Delete => Some(64),
        Home => Some(65),
        End => Some(66),
        PageUp => Some(67),
        PageDown => Some(68),

        // Punctuation and symbols
        Minus => Some(69),
        Equal => Some(70),
        BracketLeft => Some(71),
        BracketRight => Some(72),
        Backslash => Some(73),
        Semicolon => Some(74),
        Quote => Some(75),
        Backquote => Some(76),
        Comma => Some(77),
        Period => Some(78),
        Slash => Some(79),

        // NumPad
        Numpad0 => Some(80),
        Numpad1 => Some(81),
        Numpad2 => Some(82),
        Numpad3 => Some(83),
        Numpad4 => Some(84),
        Numpad5 => Some(85),
        Numpad6 => Some(86),
        Numpad7 => Some(87),
        Numpad8 => Some(88),
        Numpad9 => Some(89),
        NumpadAdd => Some(90),
        NumpadSubtract => Some(91),
        NumpadMultiply => Some(92),
        NumpadDivide => Some(93),
        NumpadEnter => Some(94),
        NumpadDecimal => Some(95),

        // Other special keys
        CapsLock => Some(96),
        NumLock => Some(97),
        ScrollLock => Some(98),
        PrintScreen => Some(99),
        Pause => Some(100),

        _ => None,
    }
}

/// Convert winit MouseButton to button index
pub fn winit_mouse_to_index(button: MouseButton) -> Option<usize> {
    match button {
        MouseButton::Left => Some(0),
        MouseButton::Right => Some(1),
        MouseButton::Middle => Some(2),
        _ => None,
    }
}

/// Handle keyboard input event from winit
pub fn handle_keyboard_input(physical_key: PhysicalKey, pressed: bool) {
    if let PhysicalKey::Code(keycode) = physical_key {
        if let Some(pgfx_key) = winit_key_to_pgfx(keycode) {
            crate::engine::with_engine(|engine| {
                if pressed {
                    engine.input.on_key_down(pgfx_key);
                } else {
                    engine.input.on_key_up(pgfx_key);
                }
            })
            .ok();
        }
    }
}

/// Handle mouse move event from winit
pub fn handle_mouse_move(x: f64, y: f64) {
    crate::engine::with_engine(|engine| {
        engine.input.on_mouse_move(x as i32, y as i32);
    })
    .ok();
}

/// Handle mouse button event from winit
pub fn handle_mouse_button(button: MouseButton, pressed: bool) {
    if let Some(index) = winit_mouse_to_index(button) {
        crate::engine::with_engine(|engine| {
            if pressed {
                engine.input.on_mouse_down(index);
            } else {
                engine.input.on_mouse_up(index);
            }
        })
        .ok();
    }
}

/// Handle mouse wheel event from winit
pub fn handle_mouse_wheel(delta: f32) {
    crate::engine::with_engine(|engine| {
        engine.input.on_mouse_wheel(delta as i32);
    })
    .ok();
}

// Helper function to access gamepad manager with lazy initialization
fn with_gamepad<F, R>(f: F) -> PyResult<R>
where
    F: FnOnce(&mut GamepadManager) -> R,
{
    crate::engine::with_engine(|engine| {
        // Initialize gamepad on first use if not already initialized
        if engine.gamepad.is_none() {
            engine.gamepad = Some(GamepadManager::new().map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to initialize gamepad: {}",
                    e
                ))
            })?);
        }

        let gamepad = engine.gamepad.as_mut().unwrap();
        Ok(f(gamepad))
    })?
}

// Python API functions

#[pyfunction]
pub fn key_down(key: u32) -> PyResult<bool> {
    crate::engine::with_engine(|engine| engine.input.keys_down.contains(&key))
}

#[pyfunction]
pub fn key_pressed(key: u32) -> PyResult<bool> {
    crate::engine::with_engine(|engine| engine.input.keys_pressed.contains(&key))
}

#[pyfunction]
pub fn key_released(key: u32) -> PyResult<bool> {
    crate::engine::with_engine(|engine| engine.input.keys_released.contains(&key))
}

#[pyfunction]
pub fn mouse_pos() -> PyResult<(i32, i32)> {
    crate::engine::with_engine(|engine| engine.input.mouse_pos)
}

#[pyfunction]
pub fn mouse_down(btn: u32) -> PyResult<bool> {
    crate::engine::with_engine(|engine| {
        if (btn as usize) < 3 {
            engine.input.mouse_down[btn as usize]
        } else {
            false
        }
    })
}

#[pyfunction]
pub fn mouse_pressed(btn: u32) -> PyResult<bool> {
    crate::engine::with_engine(|engine| {
        if (btn as usize) < 3 {
            engine.input.mouse_pressed[btn as usize]
        } else {
            false
        }
    })
}

#[pyfunction]
pub fn mouse_wheel() -> PyResult<i32> {
    crate::engine::with_engine(|engine| engine.input.mouse_wheel)
}

#[pyfunction]
#[pyo3(signature = (idx=0))]
pub fn gamepad_connected(idx: u32) -> PyResult<bool> {
    with_gamepad(|gamepad| {
        if let Some(gp) = gamepad.get_gamepad(idx as usize) {
            gp.connected
        } else {
            false
        }
    })
}

#[pyfunction]
pub fn gamepad_button(idx: u32, btn: u32) -> PyResult<bool> {
    with_gamepad(|gamepad| {
        if let Some(gp) = gamepad.get_gamepad(idx as usize) {
            if (btn as usize) < gp.buttons.len() {
                gp.buttons[btn as usize]
            } else {
                false
            }
        } else {
            false
        }
    })
}

#[pyfunction]
pub fn gamepad_axis(idx: u32, axis: u32) -> PyResult<f32> {
    with_gamepad(|gamepad| {
        if let Some(gp) = gamepad.get_gamepad(idx as usize) {
            if (axis as usize) < 4 {
                // Return axes 0-3 (left stick and right stick)
                gp.axes[axis as usize]
            } else {
                0.0
            }
        } else {
            0.0
        }
    })
}

#[pyfunction]
pub fn gamepad_trigger(idx: u32, trigger: u32) -> PyResult<f32> {
    with_gamepad(|gamepad| {
        if let Some(gp) = gamepad.get_gamepad(idx as usize) {
            if trigger == 0 {
                // Left trigger (axis 4)
                // Convert from -1..1 to 0..1
                (gp.axes[4] + 1.0) / 2.0
            } else if trigger == 1 {
                // Right trigger (axis 5)
                // Convert from -1..1 to 0..1
                (gp.axes[5] + 1.0) / 2.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    })
}
