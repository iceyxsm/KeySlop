use rdev::{listen, Event, EventType, Key};
use std::sync::mpsc;
use std::thread;

/// Converts an rdev Key to a stable string name for config mapping.
pub fn key_to_string(key: Key) -> String {
    match key {
        Key::Alt => "Alt".into(),
        Key::AltGr => "AltGr".into(),
        Key::Backspace => "Backspace".into(),
        Key::CapsLock => "CapsLock".into(),
        Key::ControlLeft => "ControlLeft".into(),
        Key::ControlRight => "ControlRight".into(),
        Key::Delete => "Delete".into(),
        Key::DownArrow => "DownArrow".into(),
        Key::End => "End".into(),
        Key::Escape => "Escape".into(),
        Key::F1 => "F1".into(),
        Key::F2 => "F2".into(),
        Key::F3 => "F3".into(),
        Key::F4 => "F4".into(),
        Key::F5 => "F5".into(),
        Key::F6 => "F6".into(),
        Key::F7 => "F7".into(),
        Key::F8 => "F8".into(),
        Key::F9 => "F9".into(),
        Key::F10 => "F10".into(),
        Key::F11 => "F11".into(),
        Key::F12 => "F12".into(),
        Key::Home => "Home".into(),
        Key::LeftArrow => "LeftArrow".into(),
        Key::MetaLeft => "MetaLeft".into(),
        Key::MetaRight => "MetaRight".into(),
        Key::PageDown => "PageDown".into(),
        Key::PageUp => "PageUp".into(),
        Key::Return => "Return".into(),
        Key::RightArrow => "RightArrow".into(),
        Key::ShiftLeft => "ShiftLeft".into(),
        Key::ShiftRight => "ShiftRight".into(),
        Key::Space => "Space".into(),
        Key::Tab => "Tab".into(),
        Key::UpArrow => "UpArrow".into(),
        Key::PrintScreen => "PrintScreen".into(),
        Key::ScrollLock => "ScrollLock".into(),
        Key::Pause => "Pause".into(),
        Key::NumLock => "NumLock".into(),
        Key::BackQuote => "BackQuote".into(),
        Key::Num1 => "1".into(),
        Key::Num2 => "2".into(),
        Key::Num3 => "3".into(),
        Key::Num4 => "4".into(),
        Key::Num5 => "5".into(),
        Key::Num6 => "6".into(),
        Key::Num7 => "7".into(),
        Key::Num8 => "8".into(),
        Key::Num9 => "9".into(),
        Key::Num0 => "0".into(),
        Key::Minus => "Minus".into(),
        Key::Equal => "Equal".into(),
        Key::LeftBracket => "LeftBracket".into(),
        Key::RightBracket => "RightBracket".into(),
        Key::SemiColon => "SemiColon".into(),
        Key::Quote => "Quote".into(),
        Key::BackSlash => "BackSlash".into(),
        Key::Comma => "Comma".into(),
        Key::Dot => "Dot".into(),
        Key::Slash => "Slash".into(),
        Key::Insert => "Insert".into(),
        Key::KpReturn => "KpReturn".into(),
        Key::KpMinus => "KpMinus".into(),
        Key::KpPlus => "KpPlus".into(),
        Key::KpMultiply => "KpMultiply".into(),
        Key::KpDivide => "KpDivide".into(),
        Key::Kp0 => "Kp0".into(),
        Key::Kp1 => "Kp1".into(),
        Key::Kp2 => "Kp2".into(),
        Key::Kp3 => "Kp3".into(),
        Key::Kp4 => "Kp4".into(),
        Key::Kp5 => "Kp5".into(),
        Key::Kp6 => "Kp6".into(),
        Key::Kp7 => "Kp7".into(),
        Key::Kp8 => "Kp8".into(),
        Key::Kp9 => "Kp9".into(),
        Key::KpDelete => "KpDelete".into(),
        Key::KeyA => "A".into(),
        Key::KeyB => "B".into(),
        Key::KeyC => "C".into(),
        Key::KeyD => "D".into(),
        Key::KeyE => "E".into(),
        Key::KeyF => "F".into(),
        Key::KeyG => "G".into(),
        Key::KeyH => "H".into(),
        Key::KeyI => "I".into(),
        Key::KeyJ => "J".into(),
        Key::KeyK => "K".into(),
        Key::KeyL => "L".into(),
        Key::KeyM => "M".into(),
        Key::KeyN => "N".into(),
        Key::KeyO => "O".into(),
        Key::KeyP => "P".into(),
        Key::KeyQ => "Q".into(),
        Key::KeyR => "R".into(),
        Key::KeyS => "S".into(),
        Key::KeyT => "T".into(),
        Key::KeyU => "U".into(),
        Key::KeyV => "V".into(),
        Key::KeyW => "W".into(),
        Key::KeyX => "X".into(),
        Key::KeyY => "Y".into(),
        Key::KeyZ => "Z".into(),
        Key::Unknown(code) => format!("Unknown({})", code),
        _ => format!("{:?}", key),
    }
}

/// Message sent from the listener thread to the main app.
#[derive(Debug, Clone)]
pub enum KeyMessage {
    KeyPressed(String),
}

/// Starts the global keyboard listener in a background thread.
/// Returns a receiver that yields key press events.
pub fn start_listener() -> mpsc::Receiver<KeyMessage> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let callback = move |event: Event| {
            if let EventType::KeyPress(key) = event.event_type {
                let key_name = key_to_string(key);
                let _ = tx.send(KeyMessage::KeyPressed(key_name));
            }
        };

        if let Err(e) = listen(callback) {
            eprintln!("Error in keyboard listener: {:?}", e);
        }
    });

    rx
}
