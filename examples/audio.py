"""Audio example - sounds and music."""

import os

import pgfx

pgfx.init(800, 600, "Audio Example")

sound = None
music = None


def on_ready():
    global sound, music
    print("Loading audio...")

    try:
        sound = pgfx.sound_load(os.path.join(os.path.dirname(__file__), "assets/sound.wav"))
        print(f"Sound loaded: ID {sound}")
    except Exception as e:
        print(f"Failed to load sound: {e}")

    try:
        music = pgfx.music_load(os.path.join(os.path.dirname(__file__), "assets/music.wav"))
        print(f"Music loaded: ID {music}")
        pgfx.music_play(music)
        print("Music started (looping)")
        pgfx.set_music_volume(0.5)
    except Exception as e:
        print(f"Failed to load/play music: {e}")


def update(dt):
    if pgfx.key_pressed(pgfx.KEY_SPACE) and sound:
        print("Playing sound effect...")
        pgfx.sound_play(sound)

    if pgfx.key_pressed(pgfx.KEY_M) and music:
        print("Pausing music...")
        pgfx.music_pause(music)

    if pgfx.key_pressed(pgfx.KEY_R) and music:
        print("Resuming music...")
        pgfx.music_resume(music)

    if pgfx.key_pressed(pgfx.KEY_S) and music:
        print("Stopping music...")
        pgfx.music_stop(music)

    if pgfx.key_pressed(pgfx.KEY_P) and music:
        print("Replaying music...")
        pgfx.music_play(music)

    return not pgfx.key_pressed(pgfx.KEY_ESCAPE)


def render():
    pgfx.clear(pgfx.Color(30, 30, 50))


print("Controls:")
print("  SPACE - Play sound effect")
print("  M - Pause music")
print("  R - Resume music")
print("  S - Stop music")
print("  P - Play music again")
print("  ESC - Exit")
print()

pgfx.run(update, render, on_ready=on_ready)
