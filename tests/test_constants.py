"""Tests for constants and Color class."""

import pgfx


def test_color_creation():
    c = pgfx.Color(255, 128, 64)
    assert c.r == 255
    assert c.g == 128
    assert c.b == 64
    assert c.a == 255  # default alpha


def test_color_with_alpha():
    c = pgfx.Color(100, 100, 100, 128)
    assert c.a == 128


def test_predefined_colors():
    assert pgfx.WHITE.r == 255
    assert pgfx.WHITE.g == 255
    assert pgfx.WHITE.b == 255

    assert pgfx.BLACK.r == 0
    assert pgfx.BLACK.g == 0
    assert pgfx.BLACK.b == 0

    assert pgfx.RED.r == 255
    assert pgfx.RED.g == 0
    assert pgfx.RED.b == 0

    assert pgfx.GREEN.r == 0
    assert pgfx.GREEN.g == 255
    assert pgfx.GREEN.b == 0

    assert pgfx.BLUE.r == 0
    assert pgfx.BLUE.g == 0
    assert pgfx.BLUE.b == 255

    assert pgfx.TRANSPARENT.a == 0


def test_key_constants_exist():
    assert pgfx.KEY_A == 0
    assert pgfx.KEY_Z == 25
    assert pgfx.KEY_SPACE is not None
    assert pgfx.KEY_ESCAPE is not None
    assert pgfx.KEY_ENTER is not None
    assert pgfx.KEY_LEFT is not None
    assert pgfx.KEY_RIGHT is not None
    assert pgfx.KEY_UP is not None
    assert pgfx.KEY_DOWN is not None


def test_mouse_constants_exist():
    assert pgfx.MOUSE_LEFT == 0
    assert pgfx.MOUSE_RIGHT == 1
    assert pgfx.MOUSE_MIDDLE == 2


def test_gamepad_constants_exist():
    assert pgfx.GAMEPAD_A is not None
    assert pgfx.GAMEPAD_B is not None
    assert pgfx.GAMEPAD_X is not None
    assert pgfx.GAMEPAD_Y is not None
    assert pgfx.GAMEPAD_AXIS_LX is not None
    assert pgfx.GAMEPAD_AXIS_LY is not None
    assert pgfx.GAMEPAD_TRIGGER_L is not None
    assert pgfx.GAMEPAD_TRIGGER_R is not None
