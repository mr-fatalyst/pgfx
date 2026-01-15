#!/usr/bin/env python3
"""Simple audio API example without window"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pgfx

print("Demonstrating audio API...")

# Initialize engine
pgfx.init(800, 600, "Audio API Example")

# loading sound
try:
    print("\n1. Loading sound...")
    sound = pgfx.sound_load(os.path.join(os.path.dirname(__file__), "assets/sound.wav"))
    print(f"   SUCCESS: Sound loaded with ID {sound}")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

# loading music
try:
    print("\n2. Loading music...")
    music = pgfx.music_load(os.path.join(os.path.dirname(__file__), "assets/music.wav"))
    print(f"   SUCCESS: Music loaded with ID {music}")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

# playing sound
try:
    print("\n3. Playing sound (volume=0.8, no loop)...")
    pgfx.sound_play(sound, volume=0.8, loop_=False)
    print("   SUCCESS: Sound playing")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

# playing music
try:
    print("\n4. Playing music (looping)...")
    pgfx.music_play(music, loop_=True)
    print("   SUCCESS: Music playing")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

# volume controls
try:
    print("\n5. Setting master volume to 0.7...")
    pgfx.set_master_volume(0.7)
    print("   SUCCESS: Master volume set")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

try:
    print("\n6. Setting music volume to 0.5...")
    pgfx.set_music_volume(0.5)
    print("   SUCCESS: Music volume set")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

# pause/resume
try:
    print("\n7. Pausing music...")
    pgfx.music_pause(music)
    print("   SUCCESS: Music paused")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

try:
    print("\n8. Resuming music...")
    pgfx.music_resume(music)
    print("   SUCCESS: Music resumed")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

# stop
try:
    print("\n9. Stopping music...")
    pgfx.music_stop(music)
    print("   SUCCESS: Music stopped")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

try:
    print("\n10. Stopping sound...")
    pgfx.sound_stop(sound)
    print("   SUCCESS: Sound stopped")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

# freeing resources
try:
    print("\n11. Freeing sound...")
    pgfx.sound_free(sound)
    print("   SUCCESS: Sound freed")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

try:
    print("\n12. Freeing music...")
    pgfx.music_free(music)
    print("   SUCCESS: Music freed")
except Exception as e:
    print(f"   FAILED: {e}")
    sys.exit(1)

print("\n" + "=" * 50)
print("ALL AUDIO TESTS PASSED!")
print("=" * 50)
