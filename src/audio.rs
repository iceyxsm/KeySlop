use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::cpal::Device;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Default max simultaneous sounds to prevent audio distortion.
const DEFAULT_MAX_POLYPHONY: usize = 5;

/// Returns a list of available output device names.
pub fn list_output_devices() -> Vec<String> {
    let host = rodio::cpal::default_host();
    let mut names = Vec::new();
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                names.push(name);
            }
        }
    }
    names
}

/// Find a cpal device by name.
fn find_device_by_name(name: &str) -> Option<Device> {
    let host = rodio::cpal::default_host();
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(n) = device.name() {
                if n == name {
                    return Some(device);
                }
            }
        }
    }
    None
}

/// Manages audio playback with device selection and polyphony limiting.
pub struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    volume: Arc<Mutex<f32>>,
    /// Active sinks for polyphony management. Uses Vec for simpler cleanup.
    active_sinks: Mutex<VecDeque<Sink>>,
    /// Max simultaneous sounds.
    max_polyphony: Arc<Mutex<usize>>,
    /// Current device name.
    device_name: Mutex<String>,
}

impl AudioPlayer {
    /// Create with the default audio device.
    pub fn new() -> Result<Self, String> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to open audio output: {}", e))?;

        let device_name = rodio::cpal::default_host()
            .default_output_device()
            .and_then(|d| d.name().ok())
            .unwrap_or_else(|| "Default".into());

        Ok(Self {
            _stream: stream,
            stream_handle,
            volume: Arc::new(Mutex::new(1.0)),
            active_sinks: Mutex::new(VecDeque::new()),
            max_polyphony: Arc::new(Mutex::new(DEFAULT_MAX_POLYPHONY)),
            device_name: Mutex::new(device_name),
        })
    }

    /// Create with a specific named audio device.
    pub fn with_device(name: &str) -> Result<Self, String> {
        let device = find_device_by_name(name)
            .ok_or_else(|| format!("Audio device not found: {}", name))?;

        let (stream, stream_handle) = OutputStream::try_from_device(&device)
            .map_err(|e| format!("Failed to open audio device '{}': {}", name, e))?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            volume: Arc::new(Mutex::new(1.0)),
            active_sinks: Mutex::new(VecDeque::new()),
            max_polyphony: Arc::new(Mutex::new(DEFAULT_MAX_POLYPHONY)),
            device_name: Mutex::new(name.to_string()),
        })
    }

    pub fn device_name(&self) -> String {
        self.device_name
            .lock()
            .map(|n| n.clone())
            .unwrap_or_else(|_| "Unknown".into())
    }

    pub fn set_volume(&self, vol: f32) {
        if let Ok(mut v) = self.volume.lock() {
            *v = vol.clamp(0.0, 1.0);
        }
    }

    pub fn set_max_polyphony(&self, max: usize) {
        if let Ok(mut m) = self.max_polyphony.lock() {
            *m = max.max(1);
        }
    }

    /// Play a sound file. Non-blocking with polyphony limiting.
    pub fn play(&self, path: &str) -> Result<(), String> {
        let file_path = Path::new(path);
        if !file_path.exists() {
            return Err(format!("Sound file not found: {}", path));
        }

        // Read max polyphony first (separate lock)
        let max = self
            .max_polyphony
            .lock()
            .map(|m| *m)
            .unwrap_or(DEFAULT_MAX_POLYPHONY);

        // Clean up and enforce polyphony
        if let Ok(mut sinks) = self.active_sinks.lock() {
            // Remove sinks that have finished playing
            sinks.retain(|sink| !sink.empty() || sink.len() > 0);

            // Stop oldest sounds if we're at the limit
            while sinks.len() >= max {
                if let Some(old_sink) = sinks.pop_front() {
                    old_sink.stop();
                    drop(old_sink);
                }
            }
        }

        // Decode the audio
        let file =
            File::open(file_path).map_err(|e| format!("Failed to open sound file: {}", e))?;
        let reader = BufReader::new(file);
        let source =
            Decoder::new(reader).map_err(|e| format!("Failed to decode sound file: {}", e))?;

        // Create sink and play
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| format!("Failed to create audio sink: {}", e))?;

        let vol = self.volume.lock().map(|v| *v).unwrap_or(1.0);
        sink.set_volume(vol);
        sink.append(source);

        // Track the sink so it stays alive
        if let Ok(mut sinks) = self.active_sinks.lock() {
            sinks.push_back(sink);
        } else {
            // Fallback: detach so it at least plays
            sink.detach();
        }

        Ok(())
    }
}
