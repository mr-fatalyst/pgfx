import pytest
from pgfx import _batch


@pytest.fixture
def clean_commands():
    """Clear the draw command buffer before and after the test."""
    _batch._commands.clear()
    yield
    _batch._commands.clear()
