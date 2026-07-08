"""The audio API step by step, no interaction: each step runs on a timer.

Plays a sound, starts music, adjusts volumes, pauses, resumes and stops —
the current step is shown on screen. Quits by itself after the last step.
"""

import os

import pgfx

SCREEN_W, SCREEN_H = 800, 600
STEP_SECONDS = 1.5
DIM = pgfx.Color(150, 155, 165)

pgfx.init(SCREEN_W, SCREEN_H, "pgfx audio walkthrough")

font = sound = music = None
step = -1
step_timer = 0.0

STEPS = [
    ("sound_play(sound)", lambda: pgfx.sound_play(sound)),
    ("music_play(music, loop_=True)", lambda: pgfx.music_play(music, loop_=True)),
    ("set_music_volume(0.4)", lambda: pgfx.set_music_volume(0.4)),
    ("set_master_volume(0.7)", lambda: pgfx.set_master_volume(0.7)),
    ("music_pause(music)", lambda: pgfx.music_pause(music)),
    ("music_resume(music)", lambda: pgfx.music_resume(music)),
    ("sound_play(sound, volume=0.6, pan=-1)", lambda: pgfx.sound_play(sound, volume=0.6, pan=-1)),
    ("sound_play(sound, volume=0.6, pan=1)", lambda: pgfx.sound_play(sound, volume=0.6, pan=1)),
    ("music_stop(music)", lambda: pgfx.music_stop(music)),
]


def on_ready():
    global font, sound, music
    here = os.path.dirname(__file__)
    font = pgfx.font_load(os.path.join(here, "assets/font.ttf"), 20)
    sound = pgfx.sound_load(os.path.join(here, "assets/sound.wav"))
    music = pgfx.music_load(os.path.join(here, "assets/music.wav"))


def update(dt):
    global step, step_timer
    if not font:
        return True

    step_timer -= dt
    if step_timer <= 0:
        step += 1
        step_timer = STEP_SECONDS
        if step >= len(STEPS):
            return False  # done
        STEPS[step][1]()

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(24, 26, 34))
    if not font or step < 0:
        return

    for i, (caption, _) in enumerate(STEPS):
        if i < step:
            color = pgfx.Color(90, 200, 110)
        elif i == step:
            color = pgfx.YELLOW
        else:
            color = DIM
        marker = "> " if i == step else "  "
        pgfx.text(font, marker + caption, 120, 120 + i * 34, color)

    pgfx.text(
        font, "runs by itself — ESC to quit early", SCREEN_W / 2, SCREEN_H - 34, DIM, align="center"
    )


pgfx.run(update, render, on_ready=on_ready)
