"""Fixtures for integration tests that need a window and GPU.

Each test runs the engine in a subprocess: the engine binds to the process
(one event loop per process), and a GPU crash must not kill the pytest run.
"""

import os
import subprocess
import sys

import pytest


def _has_display():
    if sys.platform != "linux":
        return True
    return bool(os.environ.get("DISPLAY") or os.environ.get("WAYLAND_DISPLAY"))


def pytest_collection_modifyitems(config, items):
    skip = pytest.mark.skipif(not _has_display(), reason="no display available")
    for item in items:
        item.add_marker(pytest.mark.gpu)
        item.add_marker(skip)


@pytest.fixture
def run_script():
    """Run a Python snippet in a subprocess and return the CompletedProcess."""

    def _run(code, timeout=60, env=None):
        full_env = {**os.environ, **(env or {})}
        return subprocess.run(
            [sys.executable, "-c", code],
            capture_output=True,
            text=True,
            timeout=timeout,
            env=full_env,
        )

    return _run
