"""Tests for the Python->Rust draw command encoding in pgfx._batch.

These pin the tuple protocol that render_batch parses: command type,
field order and defaults. A mismatch here means garbage on screen.
"""

import pgfx
from pgfx import _batch


def commands():
    return _batch._commands


def test_clear(clean_commands):
    pgfx.clear(pgfx.Color(10, 20, 30, 40))
    assert commands() == [(_batch.CMD_CLEAR, 10, 20, 30, 40)]


def test_draw_defaults(clean_commands):
    pgfx.draw(7, 1.5, 2.5)
    assert commands() == [(_batch.CMD_DRAW, 7, 1.5, 2.5, 0)]


def test_draw_with_z(clean_commands):
    pgfx.draw(7, 1.0, 2.0, z=5)
    assert commands() == [(_batch.CMD_DRAW, 7, 1.0, 2.0, 5)]


def test_draw_ex_defaults(clean_commands):
    pgfx.draw_ex(7, 1.0, 2.0)
    assert commands() == [(_batch.CMD_DRAW_EX, 7, 1.0, 2.0, 0, 1, 1, False, False, 0, 1, 0)]


def test_draw_ex_full(clean_commands):
    pgfx.draw_ex(7, 1.0, 2.0, rot=0.5, scale=2, alpha=0.8, flip_x=True, flip_y=True, z=3)
    assert commands() == [(_batch.CMD_DRAW_EX, 7, 1.0, 2.0, 0.5, 2, 0.8, True, True, 3, 2, 0)]


def test_draw_ex_scale_y(clean_commands):
    # scale_y defaults to scale; an explicit value overrides only the height
    pgfx.draw_ex(7, 1.0, 2.0, scale=2, scale_y=0.5)
    assert commands() == [(_batch.CMD_DRAW_EX, 7, 1.0, 2.0, 0, 2, 1, False, False, 0, 0.5, 0)]


def test_draw_ex_blend(clean_commands):
    pgfx.draw_ex(7, 1.0, 2.0, blend="add")
    assert commands()[0][-1] == 1


def test_draw_ex_rejects_unknown_blend(clean_commands):
    import pytest

    with pytest.raises(ValueError, match="blend"):
        pgfx.draw_ex(7, 1.0, 2.0, blend="multiply")
    assert commands() == []


def test_rect_fill_ex(clean_commands):
    pgfx.rect_fill_ex(10, 20, 30, 40, pgfx.RED, rot=0.5, z=2)
    assert commands() == [(_batch.CMD_RECT_FILL_EX, 10, 20, 30, 40, 0.5, 255, 0, 0, 255, 2)]


def test_rect_fill(clean_commands):
    pgfx.rect_fill(1, 2, 3, 4, pgfx.RED, z=2)
    assert commands() == [(_batch.CMD_RECT_FILL, 1, 2, 3, 4, 255, 0, 0, 255, 2)]


def test_line(clean_commands):
    pgfx.line(1, 2, 3, 4, pgfx.GREEN)
    assert commands() == [(_batch.CMD_LINE, 1, 2, 3, 4, 0, 255, 0, 255, 0, 2)]


def test_line_with_width(clean_commands):
    pgfx.line(1, 2, 3, 4, pgfx.GREEN, z=1, width=5)
    assert commands() == [(_batch.CMD_LINE, 1, 2, 3, 4, 0, 255, 0, 255, 1, 5)]


def test_circle_fill(clean_commands):
    pgfx.circle_fill(5, 6, 7, pgfx.BLUE)
    assert commands() == [(_batch.CMD_CIRCLE_FILL, 5, 6, 7, 0, 0, 255, 255, 0)]


def test_text(clean_commands):
    pgfx.text(3, "hi", 10, 20, pgfx.WHITE, z=1)
    assert commands() == [(_batch.CMD_TEXT, 3, "hi", 10, 20, 255, 255, 255, 255, 1, 0)]


def test_text_align(clean_commands):
    pgfx.text(3, "hi", 10, 20, pgfx.WHITE, align="center")
    pgfx.text(3, "hi", 10, 20, pgfx.WHITE, align="right")
    assert [cmd[-1] for cmd in commands()] == [1, 2]


def test_text_rejects_unknown_align(clean_commands):
    import pytest

    with pytest.raises(ValueError, match="align"):
        pgfx.text(3, "hi", 10, 20, pgfx.WHITE, align="middle")
    assert commands() == []


def test_particles_render(clean_commands):
    pgfx.particles_render(9, z=4)
    assert commands() == [(_batch.CMD_PARTICLES_RENDER, 9, 4)]


def test_light_draw(clean_commands):
    pgfx.light_draw(2, 100, 200)
    assert commands() == [(_batch.CMD_LIGHT_DRAW, 2, 100, 200, 0)]


def test_call_order_is_preserved(clean_commands):
    pgfx.clear(pgfx.BLACK)
    pgfx.draw(1, 0, 0)
    pgfx.text(2, "x", 0, 0, pgfx.WHITE)
    types = [cmd[0] for cmd in commands()]
    assert types == [_batch.CMD_CLEAR, _batch.CMD_DRAW, _batch.CMD_TEXT]


def test_set_view_defaults(clean_commands):
    pgfx.set_view(10, 20)
    assert commands() == [(_batch.CMD_SET_VIEW, 10.0, 20.0, 1.0, 0.0)]


def test_set_view_full(clean_commands):
    pgfx.set_view(100, 200, zoom=2.0, rot=0.5)
    assert commands() == [(_batch.CMD_SET_VIEW, 100.0, 200.0, 2.0, 0.5)]


def test_set_view_rejects_non_positive_zoom(clean_commands):
    import pytest

    with pytest.raises(ValueError, match="zoom"):
        pgfx.set_view(0, 0, zoom=0)
    assert commands() == []


def test_reset_view(clean_commands):
    pgfx.reset_view()
    assert commands() == [(_batch.CMD_RESET_VIEW,)]


def test_render_to(clean_commands):
    pgfx.render_to(5)
    pgfx.render_to(None)  # back to the screen
    assert commands() == [(_batch.CMD_RENDER_TO, 5), (_batch.CMD_RENDER_TO, 0)]
