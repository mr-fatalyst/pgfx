use pyo3::prelude::*;
use rodio::cpal::BufferSize;
use rodio::{Decoder, OutputStreamBuilder, Source};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

pub type SoundId = u32;
pub type MusicId = u32;

// Thread-local audio state (OutputStream is not Send on macOS)
thread_local! {
    static AUDIO: RefCell<Option<AudioState>> = const { RefCell::new(None) };
}

/// Sound data - raw file bytes in memory, decoded on playback
pub struct SoundData {
    data: Arc<Vec<u8>>,
}

impl SoundData {
    fn from_file(path: &str) -> Result<Self, String> {
        // Read entire file into memory
        let data = std::fs::read(path).map_err(|e| format!("Failed to read audio file: {}", e))?;

        // Verify it can be decoded
        let cursor = Cursor::new(data.clone());
        Decoder::new(cursor).map_err(|e| format!("Failed to decode audio: {}", e))?;

        Ok(Self {
            data: Arc::new(data),
        })
    }

    fn create_source(&self) -> Result<Decoder<Cursor<Vec<u8>>>, String> {
        let cursor = Cursor::new((*self.data).clone());
        Decoder::new(cursor).map_err(|e| format!("Failed to decode audio: {}", e))
    }
}

/// Music data - raw file bytes in memory, decoded on playback
pub struct MusicData {
    data: Arc<Vec<u8>>,
}

impl MusicData {
    fn from_file(path: &str) -> Result<Self, String> {
        // Read entire file into memory
        let data = std::fs::read(path).map_err(|e| format!("Failed to read music file: {}", e))?;

        // Verify it can be decoded
        let cursor = Cursor::new(data.clone());
        Decoder::new(cursor).map_err(|e| format!("Failed to decode music: {}", e))?;

        Ok(Self {
            data: Arc::new(data),
        })
    }

    fn create_source(&self) -> Result<Decoder<Cursor<Vec<u8>>>, String> {
        let cursor = Cursor::new((*self.data).clone());
        Decoder::new(cursor).map_err(|e| format!("Failed to decode music: {}", e))
    }
}

/// Audio state managed by the engine
pub struct AudioState {
    // Keep the output stream alive
    output_stream: Arc<rodio::OutputStream>,

    // Resource pools
    sounds: crate::resources::ResourcePool<SoundData>,
    music: crate::resources::ResourcePool<MusicData>,

    // Active sound playback sinks
    sound_sinks: HashMap<SoundId, Vec<rodio::Sink>>,

    // Music playback (only one music track at a time)
    music_sink: Option<(MusicId, rodio::Sink)>,

    // Volume controls
    master_volume: f32,
    music_volume: f32,
}

impl AudioState {
    pub fn new() -> Result<Self, String> {
        // Use larger buffer (300ms at 48kHz = 14400 frames) for stability in VM environments
        let output_stream = OutputStreamBuilder::from_default_device()
            .map_err(|e| format!("Failed to get default audio device: {}", e))?
            .with_buffer_size(BufferSize::Fixed(14400))
            .open_stream()
            .map_err(|e| format!("Failed to open audio stream: {}", e))?;

        Ok(Self {
            output_stream: Arc::new(output_stream),
            sounds: crate::resources::ResourcePool::new(),
            music: crate::resources::ResourcePool::new(),
            sound_sinks: HashMap::new(),
            music_sink: None,
            master_volume: 1.0,
            music_volume: 1.0,
        })
    }

    pub fn load_sound(&mut self, path: &str) -> Result<SoundId, String> {
        let sound_data = SoundData::from_file(path)?;
        let id = self.sounds.insert(sound_data);
        Ok(id)
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
        _pan: f32,
        loop_: bool,
    ) -> Result<(), String> {
        let sound = self
            .sounds
            .get(id)
            .ok_or_else(|| format!("Invalid sound ID: {}", id))?;

        let mixer = self.output_stream.mixer();
        let sink = rodio::Sink::connect_new(mixer);

        let source = sound.create_source()?;
        let amplified = source.amplify(volume * self.master_volume);

        if loop_ {
            sink.append(amplified.repeat_infinite());
        } else {
            sink.append(amplified);
        }

        // Store the sink so we can stop it later
        self.sound_sinks.entry(id).or_default().push(sink);

        // Clean up finished sinks
        if let Some(sinks) = self.sound_sinks.get_mut(&id) {
            sinks.retain(|s| !s.empty());
        }

        Ok(())
    }

    pub fn stop_sound(&mut self, id: SoundId) {
        if let Some(sinks) = self.sound_sinks.get_mut(&id) {
            for sink in sinks.iter() {
                sink.stop();
            }
            sinks.clear();
        }
    }

    pub fn load_music(&mut self, path: &str) -> Result<MusicId, String> {
        let music_data = MusicData::from_file(path)?;
        let id = self.music.insert(music_data);
        Ok(id)
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

        let mixer = self.output_stream.mixer();
        let sink = rodio::Sink::connect_new(mixer);

        let source = music.create_source()?;
        let amplified = source.amplify(self.music_volume * self.master_volume);

        if loop_ {
            sink.append(amplified.repeat_infinite());
        } else {
            sink.append(amplified);
        }

        self.music_sink = Some((id, sink));

        Ok(())
    }

    pub fn stop_music(&mut self) {
        if let Some((_, sink)) = &self.music_sink {
            sink.stop();
        }
        self.music_sink = None;
    }

    pub fn pause_music(&mut self) -> Result<(), String> {
        if let Some((_, sink)) = &self.music_sink {
            sink.pause();
            Ok(())
        } else {
            Err("No music is currently playing".to_string())
        }
    }

    pub fn resume_music(&mut self) -> Result<(), String> {
        if let Some((_, sink)) = &self.music_sink {
            sink.play();
            Ok(())
        } else {
            Err("No music is currently playing".to_string())
        }
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);

        // Note: In rodio 0.21, volume is applied when creating the source
        // We can't change volume of already-playing sounds
        // This will affect new sounds/music played after this call
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);

        // Note: Volume is applied when creating the source
        // This will affect new music played after this call
    }
}

// Helper function to access audio state (thread-local)
fn with_audio<F, R>(f: F) -> PyResult<R>
where
    F: FnOnce(&mut AudioState) -> Result<R, String>,
{
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
pub fn music_stop(_mus: MusicId) -> PyResult<()> {
    with_audio(|audio| {
        audio.stop_music();
        Ok(())
    })
}

#[pyfunction]
pub fn music_pause(_mus: MusicId) -> PyResult<()> {
    with_audio(|audio| audio.pause_music())
}

#[pyfunction]
pub fn music_resume(_mus: MusicId) -> PyResult<()> {
    with_audio(|audio| audio.resume_music())
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
