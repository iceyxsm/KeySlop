use crate::audio::{self, AudioPlayer};
use crate::autostart;
use crate::config::AppConfig;
use crate::listener::{self, KeyMessage};
use eframe::egui;
use std::sync::mpsc;

/// The main application state.
pub struct KeySlopApp {
    config: AppConfig,
    audio: Option<AudioPlayer>,
    key_rx: mpsc::Receiver<KeyMessage>,
    last_key: String,
    status_msg: String,
    capturing_key: bool,
    selected_key: Option<String>,
    filter_text: String,
    /// Cached list of audio output devices.
    available_devices: Vec<String>,
}

impl KeySlopApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load();
        let available_devices = audio::list_output_devices();

        // Try to use saved device, fall back to default
        let audio = if let Some(ref device_name) = config.audio_device {
            match AudioPlayer::with_device(device_name) {
                Ok(player) => Some(player),
                Err(_) => AudioPlayer::new().ok(),
            }
        } else {
            AudioPlayer::new().ok()
        };

        if let Some(ref player) = audio {
            player.set_volume(config.volume);
            player.set_max_polyphony(config.max_polyphony);
        }

        let key_rx = listener::start_listener();

        Self {
            config,
            audio,
            key_rx,
            last_key: String::new(),
            status_msg: "Ready".into(),
            capturing_key: false,
            selected_key: None,
            filter_text: String::new(),
            available_devices,
        }
    }

    fn refresh_devices(&mut self) {
        self.available_devices = audio::list_output_devices();
    }

    fn switch_audio_device(&mut self, device_name: &str) {
        match AudioPlayer::with_device(device_name) {
            Ok(player) => {
                player.set_volume(self.config.volume);
                player.set_max_polyphony(self.config.max_polyphony);
                self.audio = Some(player);
                self.config.audio_device = Some(device_name.to_string());
                self.status_msg = format!("Switched to: {}", device_name);
                self.save_config();
            }
            Err(e) => {
                self.status_msg = format!("Failed to switch device: {}", e);
            }
        }
    }

    fn save_config(&mut self) {
        match self.config.save() {
            Ok(_) => self.status_msg = "Config saved".into(),
            Err(e) => self.status_msg = format!("Save failed: {}", e),
        }
    }

    fn pick_sound_file() -> Option<String> {
        let file = rfd::FileDialog::new()
            .add_filter("Audio files", &["wav", "mp3", "ogg", "flac"])
            .pick_file();
        file.map(|p| p.to_string_lossy().into_owned())
    }

    fn process_key_events(&mut self) {
        while let Ok(msg) = self.key_rx.try_recv() {
            match msg {
                KeyMessage::KeyPressed(key_name) => {
                    self.last_key = key_name.clone();

                    if self.capturing_key {
                        self.selected_key = Some(key_name);
                        self.capturing_key = false;
                    } else if self.config.enabled {
                        if let Some(sound_path) = self.config.sound_for_key(&key_name).cloned() {
                            if let Some(ref audio) = self.audio {
                                if let Err(e) = audio.play(&sound_path) {
                                    self.status_msg = format!("Playback error: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for KeySlopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process incoming key events
        self.process_key_events();

        // Request repaint to keep processing events
        ctx.request_repaint();

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("KeySlop");
                ui.separator();
                ui.label(format!("Last key: {}", self.last_key));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let toggle_text = if self.config.enabled {
                        "🔊 Enabled"
                    } else {
                        "🔇 Disabled"
                    };
                    if ui.button(toggle_text).clicked() {
                        self.config.enabled = !self.config.enabled;
                        self.save_config();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_msg);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // --- Global Sound Section ---
            ui.group(|ui| {
                ui.heading("Global Sound");
                ui.label("Plays for any key that doesn't have a specific sound assigned.");
                ui.horizontal(|ui| {
                    let display = self
                        .config
                        .global_sound
                        .as_deref()
                        .unwrap_or("None");
                    ui.label(format!("Current: {}", display));
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = Self::pick_sound_file() {
                            self.config.global_sound = Some(path);
                            self.save_config();
                        }
                    }
                    if self.config.global_sound.is_some() && ui.button("Clear").clicked() {
                        self.config.global_sound = None;
                        self.save_config();
                    }
                });
            });

            ui.add_space(8.0);

            // --- Volume ---
            ui.group(|ui| {
                ui.heading("Volume");
                let mut vol = self.config.volume;
                if ui
                    .add(egui::Slider::new(&mut vol, 0.0..=1.0).text("Master Volume"))
                    .changed()
                {
                    self.config.volume = vol;
                    if let Some(ref audio) = self.audio {
                        audio.set_volume(vol);
                    }
                    self.save_config();
                }
            });

            ui.add_space(8.0);

            // --- Settings ---
            ui.group(|ui| {
                ui.heading("Settings");

                // Audio device selector
                ui.horizontal(|ui| {
                    ui.label("Audio Output:");
                    let current_device = self
                        .audio
                        .as_ref()
                        .map(|a| a.device_name())
                        .unwrap_or_else(|| "None".into());

                    egui::ComboBox::from_id_salt("audio_device")
                        .selected_text(&current_device)
                        .show_ui(ui, |ui| {
                            let mut selected: Option<String> = None;
                            for device in &self.available_devices {
                                if ui
                                    .selectable_label(*device == current_device, device)
                                    .clicked()
                                {
                                    selected = Some(device.clone());
                                }
                            }
                            // Apply selection outside the borrow
                            if let Some(device_name) = selected {
                                // We can't call self methods here, so store for later
                                self.status_msg = format!("__switch_device:{}", device_name);
                            }
                        });
                    if ui.button("🔄").on_hover_text("Refresh devices").clicked() {
                        self.refresh_devices();
                    }
                });

                // Handle deferred device switch
                if self.status_msg.starts_with("__switch_device:") {
                    let device_name = self.status_msg["__switch_device:".len()..].to_string();
                    self.switch_audio_device(&device_name);
                }

                // Polyphony limit
                let mut polyphony = self.config.max_polyphony as i32;
                if ui
                    .add(
                        egui::Slider::new(&mut polyphony, 1..=20)
                            .text("Max simultaneous sounds"),
                    )
                    .changed()
                {
                    self.config.max_polyphony = polyphony as usize;
                    if let Some(ref audio) = self.audio {
                        audio.set_max_polyphony(polyphony as usize);
                    }
                    self.save_config();
                }

                // Autostart toggle
                let mut autostart_enabled = self.config.autostart;
                if ui
                    .checkbox(&mut autostart_enabled, "Start on boot")
                    .changed()
                {
                    self.config.autostart = autostart_enabled;
                    if autostart_enabled {
                        match autostart::enable() {
                            Ok(_) => self.status_msg = "Autostart enabled".into(),
                            Err(e) => {
                                self.status_msg = format!("Autostart failed: {}", e);
                                self.config.autostart = false;
                            }
                        }
                    } else {
                        match autostart::disable() {
                            Ok(_) => self.status_msg = "Autostart disabled".into(),
                            Err(e) => {
                                self.status_msg = format!("Failed to disable autostart: {}", e);
                                self.config.autostart = true;
                            }
                        }
                    }
                    self.save_config();
                }
            });

            ui.add_space(8.0);

            // --- Per-Key Sounds ---
            ui.group(|ui| {
                ui.heading("Per-Key Sounds");

                ui.horizontal(|ui| {
                    if self.capturing_key {
                        ui.label("⌨ Press any key to select it...");
                        if ui.button("Cancel").clicked() {
                            self.capturing_key = false;
                        }
                    } else {
                        if ui.button("Capture Key").clicked() {
                            self.capturing_key = true;
                            self.selected_key = None;
                        }
                        if let Some(ref key) = self.selected_key {
                            ui.label(format!("Selected: {}", key));
                            if ui.button("Assign Sound").clicked() {
                                if let Some(path) = Self::pick_sound_file() {
                                    self.config.key_sounds.insert(key.clone(), path);
                                    self.save_config();
                                }
                            }
                        }
                    }
                });

                ui.add_space(4.0);
                ui.separator();

                // Filter
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.filter_text);
                });

                // Mappings list
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        let mut to_remove: Option<String> = None;
                        let filter_lower = self.filter_text.to_lowercase();

                        let mut keys: Vec<String> =
                            self.config.key_sounds.keys().cloned().collect();
                        keys.sort();

                        for key in &keys {
                            if !filter_lower.is_empty()
                                && !key.to_lowercase().contains(&filter_lower)
                            {
                                continue;
                            }
                            let sound = &self.config.key_sounds[key];
                            ui.horizontal(|ui| {
                                ui.monospace(format!("{:>15}", key));
                                ui.label("→");
                                // Show just the filename for readability
                                let display_name = std::path::Path::new(sound)
                                    .file_name()
                                    .map(|f| f.to_string_lossy().into_owned())
                                    .unwrap_or_else(|| sound.clone());
                                ui.label(&display_name);
                                if ui.small_button("🗑").on_hover_text("Remove").clicked() {
                                    to_remove = Some(key.clone());
                                }
                                if ui.small_button("🔊").on_hover_text("Test").clicked() {
                                    if let Some(ref audio) = self.audio {
                                        if let Err(e) = audio.play(sound) {
                                            self.status_msg =
                                                format!("Test playback error: {}", e);
                                        }
                                    }
                                }
                            });
                        }

                        if let Some(key) = to_remove {
                            self.config.key_sounds.remove(&key);
                            self.save_config();
                        }

                        if keys.is_empty() {
                            ui.label("No per-key sounds configured yet.");
                        }
                    });
            });
        });
    }
}
