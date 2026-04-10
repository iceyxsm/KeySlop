use std::path::PathBuf;

const APP_NAME: &str = "KeySlop";

/// Get the path to the current running executable.
fn exe_path() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))
}

/// Enable autostart on the current platform.
pub fn enable() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        enable_windows()
    }
    #[cfg(target_os = "linux")]
    {
        enable_linux()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Autostart is not supported on this platform".into())
    }
}

/// Disable autostart on the current platform.
pub fn disable() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        disable_windows()
    }
    #[cfg(target_os = "linux")]
    {
        disable_linux()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Autostart is not supported on this platform".into())
    }
}

/// Check if autostart is currently enabled.
pub fn is_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        is_enabled_windows()
    }
    #[cfg(target_os = "linux")]
    {
        is_enabled_linux()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

// --- Windows implementation using registry ---

#[cfg(target_os = "windows")]
fn windows_registry_key() -> &'static str {
    r"Software\Microsoft\Windows\CurrentVersion\Run"
}

#[cfg(target_os = "windows")]
fn enable_windows() -> Result<(), String> {
    use std::process::Command;
    let exe = exe_path()?;
    let exe_str = exe.to_string_lossy();
    let reg_key = windows_registry_key();

    let output = Command::new("reg")
        .args([
            "add",
            &format!("HKCU\\{}", reg_key),
            "/v",
            APP_NAME,
            "/t",
            "REG_SZ",
            "/d",
            &format!("\"{}\"", exe_str),
            "/f",
        ])
        .output()
        .map_err(|e| format!("Failed to run reg command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to set registry key: {}", stderr))
    }
}

#[cfg(target_os = "windows")]
fn disable_windows() -> Result<(), String> {
    use std::process::Command;
    let reg_key = windows_registry_key();

    let output = Command::new("reg")
        .args([
            "delete",
            &format!("HKCU\\{}", reg_key),
            "/v",
            APP_NAME,
            "/f",
        ])
        .output()
        .map_err(|e| format!("Failed to run reg command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        // If the key doesn't exist, that's fine
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn is_enabled_windows() -> bool {
    use std::process::Command;
    let reg_key = windows_registry_key();

    let output = Command::new("reg")
        .args([
            "query",
            &format!("HKCU\\{}", reg_key),
            "/v",
            APP_NAME,
        ])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

// --- Linux implementation using XDG autostart ---

#[cfg(target_os = "linux")]
fn autostart_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".config"));
    let dir = base.join("autostart");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

#[cfg(target_os = "linux")]
fn desktop_file_path() -> PathBuf {
    autostart_dir().join("keyslop.desktop")
}

#[cfg(target_os = "linux")]
fn enable_linux() -> Result<(), String> {
    let exe = exe_path()?;
    let exe_str = exe.to_string_lossy();
    let desktop_entry = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={}\n\
         Exec={}\n\
         Hidden=false\n\
         NoDisplay=false\n\
         X-GNOME-Autostart-enabled=true\n\
         Comment=Keyboard soundboard application\n",
        APP_NAME, exe_str
    );

    let path = desktop_file_path();
    std::fs::write(&path, desktop_entry)
        .map_err(|e| format!("Failed to write desktop file: {}", e))
}

#[cfg(target_os = "linux")]
fn disable_linux() -> Result<(), String> {
    let path = desktop_file_path();
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| format!("Failed to remove desktop file: {}", e))
    } else {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn is_enabled_linux() -> bool {
    desktop_file_path().exists()
}
