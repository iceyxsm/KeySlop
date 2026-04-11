use muda::{Menu, MenuEvent, MenuItem};
use std::sync::mpsc;
use tray_icon::{TrayIcon, TrayIconBuilder};

/// Messages from the tray icon to the app.
#[derive(Debug, Clone, PartialEq)]
pub enum TrayMessage {
    Show,
    Quit,
}

/// Holds the tray icon and menu state.
pub struct AppTray {
    _tray: TrayIcon,
    show_id: muda::MenuId,
    quit_id: muda::MenuId,
    tx: mpsc::Sender<TrayMessage>,
    rx: mpsc::Receiver<TrayMessage>,
}

/// Generate a simple 16x16 RGBA icon (a filled square in a given color).
fn generate_icon() -> tray_icon::Icon {
    let size: u32 = 16;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for _ in 0..(size * size) {
        // A nice purple-ish color
        rgba.push(140); // R
        rgba.push(80);  // G
        rgba.push(220); // B
        rgba.push(255); // A
    }
    tray_icon::Icon::from_rgba(rgba, size, size).expect("Failed to create tray icon")
}

impl AppTray {
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();

        let menu = Menu::new();
        let show_item = MenuItem::new("Show KeySlop", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        let show_id = show_item.id().clone();
        let quit_id = quit_item.id().clone();

        menu.append(&show_item).map_err(|e| format!("Menu error: {}", e))?;
        menu.append(&quit_item).map_err(|e| format!("Menu error: {}", e))?;

        let icon = generate_icon();

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("KeySlop - Keyboard Soundboard")
            .with_icon(icon)
            .build()
            .map_err(|e| format!("Failed to create tray icon: {}", e))?;

        Ok(Self {
            _tray: tray,
            show_id,
            quit_id,
            tx,
            rx,
        })
    }

    /// Poll for tray menu events. Call this from the UI loop.
    pub fn poll(&self) -> Option<TrayMessage> {
        // Check muda menu events
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.show_id {
                return Some(TrayMessage::Show);
            } else if event.id == self.quit_id {
                return Some(TrayMessage::Quit);
            }
        }

        // Check channel for programmatic messages
        if let Ok(msg) = self.rx.try_recv() {
            return Some(msg);
        }

        None
    }
}
