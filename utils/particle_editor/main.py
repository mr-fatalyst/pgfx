#!/usr/bin/env python3
"""Particle Editor for pgfx"""

import os
import sys

# Add project root to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

import pgfx

from utils.particle_editor.editor import SCREEN_H, SCREEN_W, ParticleEditor


def main():
    pgfx.init(SCREEN_W, SCREEN_H, "pgfx Particle Editor")
    editor = ParticleEditor()
    pgfx.run(editor.update, editor.render, on_ready=editor.on_ready)


if __name__ == "__main__":
    main()
