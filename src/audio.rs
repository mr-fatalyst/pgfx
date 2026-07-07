use pyo3::prelude::*;
use rodio::cpal::BufferSize;
use rodio::source::ChannelVolume;
use rodio::{Decoder, OutputStreamBuilder, Source};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, OnceLock};

pub type SoundId = u32;
pub type MusicId = u32;

// Audio state lives on the thread that first used the audio API (rodio's
// OutputStream is not Send on macOS). with_audio() guards against use from
// other threads instead of silently creating a second output stream.
thread_local! {
    static AUDIO: RefCell<Option<AudioState>> = const { RefCell::new(None) };
}

static AUDIO_THREAD: OnceLock<std::thread::ThreadId> = OnceLock::new();

/// Decoded-on-demand audio clip: raw file bytes shared via Arc,
/// so creating a playback source is a pointer copy, not a data copy.
pub struct AudioData {
    data: Arc<[u8]>,
}

impl AudioData {
    fn from_file(path: &str) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("Failed to read audio file: {}", e))?;
        let data: Arc<[u8]> = data.into();

        // Verify it can be decoded
        Decoder::new(Cursor::new(data.clone()))
            .map_err(|e| format!("Failed to decode audio: {}", e))?;

        Ok(Self { data })
    }

    fn create_source(&self) -> Result<Decoder<Cursor<Arc<[u8]>>>, String> {
        Decoder::new(Cursor::new(self.data.clone()))
            .map_err(|e| format!("Failed to decode audio: {}", e))
    }
}

/// Audio state managed by the engine
pub struct AudioState {
    // Keep the output stream alive
    output_stream: rodio::OutputStream,

    // Resource pools
    sounds: crate::resources::ResourcePool<AudioData>,
    music: crate::resources::ResourcePool<AudioData>,

    // Active sound playback sinks with their base volume (before master)
    sound_sinks: HashMap<SoundId, Vec<(rodio::Sink, f32)>>,

    // Music playback (only one music track at a time)
    music_sink: Option<(MusicId, rodio::Sink)>,

    // Volume controls
    master_volume: f32,
    music_volume: f32,
}

impl AudioState {
    pub fn new() -> Result<Self, String> {
        let mut builder = OutputStreamBuilder::from_default_device()
            .map_err(|e| format!("Failed to get default audio device: {}", e))?;

        // Default: the device's native buffer size (lowest reliable latency).
        // PGFX_AUDIO_BUFFER=<frames> opts into a fixed size, e.g. large values
        // for crackle-free audio in VMs/CI (14400 = 300ms at 48kHz).
        if let Ok(frames) = std::env::var("PGFX_AUDIO_BUFFER") {
            let frames: u32 = frames
                .parse()
                .map_err(|_| "PGFX_AUDIO_BUFFER must be a positive integer".to_string())?;
            builder = builder.with_buffer_size(BufferSize::Fixed(frames));
        }

        let output_stream = builder
            .open_stream()
            .map_err(|e| format!("Failed to open audio stream: {}", e))?;

        Ok(Self {
            output_stream,
            sounds: crate::resources::ResourcePool::new(),
            music: crate::resources::ResourcePool::new(),
            sound_sinks: HashMap::new(),
            music_sink: None,
            master_volume: 1.0,
            music_volume: 1.0,
        })
    }

    pub fn load_sound(&mut self, path: &str) -> Result<SoundId, String> {
        Ok(self.sounds.insert(AudioData::from_file(path)?))
    }

    pub fn free_sound(&mut self, id: SoundId) {
        self.sounds.remove(id);
        // Stop all instances of this sound
        self.sound_sinks.remove(&id);
    }

    pub fn play_sound(
        &mut self,
        id: SoundId,
        volume: f32,
        pan: f32,
        loop_: bool,
    ) -> Result<(), String> {
        let sound = self
            .sounds
            .get(id)
            .ok_or_else(|| format!("Invalid sound ID: {}", id))?;

        let sink = rodio::Sink::connect_new(self.output_stream.mixer());
        sink.set_volume(volume * self.master_volume);

        let source = sound.create_source()?;

        // Pan by playing the (mono-mixed) source at different L/R volumes.
        // pan=0 keeps the source untouched to preserve stereo.
        let pan = pan.clamp(-1.0, 1.0);
        if pan != 0.0 {
            let left = 1.0 - pan.max(0.0);
            let right = 1.0 + pan.min(0.0);
            let panned = ChannelVolume::new(source, vec![left, right]);
            if loop_ {
                sink.append(panned.repeat_infinite());
            } else {
                sink.append(panned);
            }
        } else if loop_ {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        // Store the sink (with its base volume) so we can stop it or
        // re-apply master volume later; drop finished sinks on the way.
        let sinks = self.sound_sinks.entry(id).or_default();
        sinks.retain(|(s, _)| !s.empty());
        sinks.push((sink, volume));

        Ok(())
    }

    pub fn stop_sound(&mut self, id: SoundId) {
        if let Some(sinks) = self.sound_sinks.get_mut(&id) {
            for (sink, _) in sinks.iter() {
                sink.stop();
            }
            sinks.clear();
        }
    }

    pub fn load_music(&mut self, path: &str) -> Result<MusicId, String> {
        Ok(self.music.insert(AudioData::from_file(path)?))
    }

    pub fn free_music(&mut self, id: MusicId) {
        self.music.remove(id);
        // Stop music if it's currently playing
        if let Some((current_id, _)) = &self.music_sink {
            if *current_id == id {
                self.music_sink = None;
            }
        }
    }

    pub fn play_music(&mut self, id: MusicId, loop_: bool) -> Result<(), String> {
        let music = self
            .music
            .get(id)
            .ok_or_else(|| format!("Invalid music ID: {}", id))?;

        // Stop current music if any
        self.music_sink = None;

        let sink = rodio::Sink::connect_new(self.output_stream.mixer());
        sink.set_volume(self.music_volume * self.master_volume);

        let source = music.create_source()?;
        if loop_ {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        self.music_sink = Some((id, sink));

        Ok(())
    }

    /// Stop music if the given track is the one playing (no-op otherwise)
    pub fn stop_music(&mut self, id: MusicId) {
        if let Some((current_id, sink)) = &self.music_sink {
            if *current_id == id {
                sink.stop();
                self.music_sink = None;
            }
        }
    }

    /// Pause music if the given track is the one playing (no-op otherwise)
    pub fn pause_music(&mut self, id: MusicId) {
        if let Some((current_id, sink)) = &self.music_sink {
            if *current_id == id {
                sink.pause();
            }
        }
    }

    /// Resume music if the given track is the one paused (no-op otherwise)
    pub fn resume_music(&mut self, id: MusicId) {
        if let Some((current_id, sink)) = &self.music_sink {
            if *current_id == id {
                sink.play();
            }
        }
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        self.apply_volumes();
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
        self.apply_volumes();
    }

    /// Re-apply volumes to everything currently playing
    fn apply_volumes(&mut self) {
        for sinks in self.sound_sinks.values_mut() {
            sinks.retain(|(sink, _)| !sink.empty());
            for (sink, base) in sinks.iter() {
                sink.set_volume(base * self.master_volume);
            }
        }
        if let Some((_, sink)) = &self.music_sink {
            sink.set_volume(self.music_volume * self.master_volume);
        }
    }
}

// Helper function to access audio state (owned by the first thread that used it)
fn with_audio<F, R>(f: F) -> PyResult<R>
where
    F: FnOnce(&mut AudioState) -> Result<R, String>,
{
    let current = std::thread::current().id();
    let owner = *AUDIO_THREAD.get_or_init(|| current);
    if owner != current {
        return Err(pyo3::exceptions::PyRuntimeError::new_err(
            "pgfx audio API can only be used from the thread that first used it \
             (usually the main thread)",
        ));
    }

    AUDIO.with(|audio_cell| {
        let mut audio_opt = audio_cell.borrow_mut();

        // Initialize audio on first use if not already initialized
        if audio_opt.is_none() {
            *audio_opt = Some(AudioState::new().map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to initialize audio: {}",
                    e
                ))
            })?);
        }

        let audio = audio_opt.as_mut().unwrap();
        f(audio).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    })
}

#[pyfunction]
pub fn sound_load(path: &str) -> PyResult<SoundId> {
    with_audio(|audio| audio.load_sound(path))
}

#[pyfunction]
pub fn sound_free(snd: SoundId) -> PyResult<()> {
    with_audio(|audio| {
        audio.free_sound(snd);
        Ok(())
    })
}

#[pyfunction]
#[pyo3(signature = (snd, volume=1.0, pan=0.0, loop_=false))]
pub fn sound_play(snd: SoundId, volume: f32, pan: f32, loop_: bool) -> PyResult<()> {
    with_audio(|audio| audio.play_sound(snd, volume, pan, loop_))
}

#[pyfunction]
pub fn sound_stop(snd: SoundId) -> PyResult<()> {
    with_audio(|audio| {
        audio.stop_sound(snd);
        Ok(())
    })
}

#[pyfunction]
pub fn music_load(path: &str) -> PyResult<MusicId> {
    with_audio(|audio| audio.load_music(path))
}

#[pyfunction]
pub fn music_free(mus: MusicId) -> PyResult<()> {
    with_audio(|audio| {
        audio.free_music(mus);
        Ok(())
    })
}

#[pyfunction]
#[pyo3(signature = (mus, loop_=true))]
pub fn music_play(mus: MusicId, loop_: bool) -> PyResult<()> {
    with_audio(|audio| audio.play_music(mus, loop_))
}

#[pyfunction]
pub fn music_stop(mus: MusicId) -> PyResult<()> {
    with_audio(|audio| {
        audio.stop_music(mus);
        Ok(())
    })
}

#[pyfunction]
pub fn music_pause(mus: MusicId) -> PyResult<()> {
    with_audio(|audio| {
        audio.pause_music(mus);
        Ok(())
    })
}

#[pyfunction]
pub fn music_resume(mus: MusicId) -> PyResult<()> {
    with_audio(|audio| {
        audio.resume_music(mus);
        Ok(())
    })
}

#[pyfunction]
pub fn set_master_volume(vol: f32) -> PyResult<()> {
    with_audio(|audio| {
        audio.set_master_volume(vol);
        Ok(())
    })
}

#[pyfunction]
pub fn set_music_volume(vol: f32) -> PyResult<()> {
    with_audio(|audio| {
        audio.set_music_volume(vol);
        Ok(())
    })
}
