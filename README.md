# KeySlop

A lightweight, cross-platform keyboard soundboard. Assign custom sounds to individual keys or set a single global sound for every keypress. Built with Rust for minimal resource usage.

## Features

- Global sound: one sound for all keys
- Per-key sounds: assign a different sound to each key
- Audio device selector: pick which output device to use
- Polyphony control: limit max simultaneous sounds to prevent distortion
- Master volume slider
- Enable/disable toggle
- Start on boot (Windows and Linux)
- Config auto-saves to disk
- Supports WAV, MP3, OGG, and FLAC

## Requirements

- Rust 1.70+
- On Linux: `libasound2-dev` (ALSA) and `libx11-dev` / `libxtst-dev` for global key capture

## Build

```
cargo build --release
```

The binary will be at `target/release/keyslop` (Linux) or `target\release\keyslop.exe` (Windows).

## Run

```
cargo run
```

## How It Works

1. Launch the app
2. Set a global sound using Browse, or capture individual keys and assign sounds to them
3. Type anywhere on your system -- sounds play globally, even when the app is not focused
4. Adjust volume, polyphony limit, and output device from the Settings panel

## Config

Config is stored at:

- Windows: `%APPDATA%\keyslop\config.json`
- Linux: `~/.config/keyslop/config.json`

## Tech Stack

- eframe/egui -- GPU-accelerated GUI
- rdev -- global keyboard listener
- rodio/cpal -- audio playback
- rfd -- native file dialogs
- serde -- config serialization

## License

MIT
