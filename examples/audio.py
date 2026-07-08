"""Sounds and music with on-screen state.

Controls:
    Space   play the sound effect
    P       play music from the start (loops)
    M / R   pause / resume music
    S       stop music
    Esc     quit
"""

import os

import pgfx

SCREEN_W, SCREEN_H = 800, 600
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx audio")

font = sound = music = None
music_state = "stopped"


def on_ready():
    global font, sound, music, music_state
    here = os.path.dirname(__file__)
    font = pgfx.font_load(os.path.join(here, "assets/font.ttf"), 20)
    sound = pgfx.sound_load(os.path.join(here, "assets/sound.wav"))
    music = pgfx.music_load(os.path.join(here, "assets/music.wav"))

    pgfx.set_music_volume(0.5)
    pgfx.music_play(music)
    music_state = "playing"


def update(dt):
    global music_state
    if not font:
        return True

    if pgfx.key_pressed(pgfx.KEY_SPACE):
        pgfx.sound_play(sound)
    if pgfx.key_pressed(pgfx.KEY_P):
        pgfx.music_play(music)
        music_state = "playing"
    if pgfx.key_pressed(pgfx.KEY_M) and music_state == "playing":
        pgfx.music_pause(music)
        music_state = "paused"
    if pgfx.key_pressed(pgfx.KEY_R) and music_state == "paused":
        pgfx.music_resume(music)
        music_state = "playing"
    if pgfx.key_pressed(pgfx.KEY_S):
        pgfx.music_stop(music)
        music_state = "stopped"

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))
    if not font:
        return

    pgfx.text(font, f"music: {music_state}", SCREEN_W / 2, 200, pgfx.WHITE, align="center")

    lines = [
        "SPACE — sound effect",
        "P — play music    S — stop",
        "M — pause    R — resume",
    ]
    for i, line in enumerate(lines):
        pgfx.text(font, line, SCREEN_W / 2, 280 + i * 32, DIM, align="center")

    pgfx.text(font, "ESC TO QUIT", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center")


pgfx.run(update, render, on_ready=on_ready)
