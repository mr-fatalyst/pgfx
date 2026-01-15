"""Tests for collision functions."""

import pgfx


def test_collide_rects_overlap():
    assert pgfx.collide_rects(0, 0, 10, 10, 5, 5, 10, 10) is True


def test_collide_rects_no_overlap():
    assert pgfx.collide_rects(0, 0, 10, 10, 20, 20, 10, 10) is False


def test_collide_circles_overlap():
    assert pgfx.collide_circles(0, 0, 10, 15, 0, 10) is True


def test_collide_circles_no_overlap():
    assert pgfx.collide_circles(0, 0, 5, 20, 0, 5) is False


def test_point_in_rect_inside():
    assert pgfx.point_in_rect(5, 5, 0, 0, 10, 10) is True


def test_point_in_rect_outside():
    assert pgfx.point_in_rect(15, 15, 0, 0, 10, 10) is False


def test_point_in_circle_inside():
    assert pgfx.point_in_circle(3, 4, 0, 0, 10) is True


def test_point_in_circle_outside():
    assert pgfx.point_in_circle(10, 10, 0, 0, 5) is False


def test_collide_circle_rect():
    assert pgfx.collide_circle_rect(15, 5, 10, 0, 0, 10, 10) is True
    assert pgfx.collide_circle_rect(25, 5, 5, 0, 0, 10, 10) is False


def test_raycast_rect_hit():
    result = pgfx.raycast_rect(0, 5, 1, 0, 10, 0, 10, 10)
    assert result is not None
    assert abs(result - 10.0) < 0.001


def test_raycast_rect_miss():
    result = pgfx.raycast_rect(0, 0, 0, 1, 10, 10, 10, 10)
    assert result is None


# Edge cases
def test_rects_touching():
    # Rects touching at edge should not collide
    assert pgfx.collide_rects(0, 0, 10, 10, 10, 0, 10, 10) is False


def test_circles_touching():
    # Circles exactly touching
    assert pgfx.collide_circles(0, 0, 5, 10, 0, 5) is True


def test_point_on_rect_edge():
    # Point on edge is inside
    assert pgfx.point_in_rect(0, 0, 0, 0, 10, 10) is True
    assert pgfx.point_in_rect(10, 10, 0, 0, 10, 10) is True


def test_point_on_circle_edge():
    # Point exactly on circle edge
    assert pgfx.point_in_circle(5, 0, 0, 0, 5) is True


def test_zero_size_rect():
    # Zero size rect (point) inside another rect - collides
    assert pgfx.collide_rects(5, 5, 0, 0, 0, 0, 10, 10) is True
    # Zero size rect outside - no collision
    assert pgfx.collide_rects(15, 15, 0, 0, 0, 0, 10, 10) is False


def test_zero_radius_circle():
    # Zero radius circle (point)
    assert pgfx.collide_circles(5, 5, 0, 5, 5, 0) is True  # Same point
    assert pgfx.collide_circles(0, 0, 0, 5, 5, 0) is False  # Different points


def test_raycast_from_inside():
    # Ray starting inside rect
    result = pgfx.raycast_rect(5, 5, 1, 0, 0, 0, 10, 10)
    assert result is not None
