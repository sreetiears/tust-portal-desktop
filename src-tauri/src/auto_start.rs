use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::Mutex;

static REGISTRY_MUTEX: Mutex<()> = Mutex::new(());

pub fn is_enabled() -> bool {
    match Command::new("reg")
        .creation_flags(0x08000000)
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            "TustPortal",
        ])
        .output()
    {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    if enabled {
        let exe_path = std::env::current_exe()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();
        Command::new("reg")
            .creation_flags(0x08000000)
            .args([
                "add", key, "/v", "TustPortal", "/t", "REG_SZ", "/d", &exe_path, "/f",
            ])
            .status()
            .map_err(|e| e.to_string())?;
    } else {
        Command::new("reg")
            .creation_flags(0x08000000)
            .args(["delete", key, "/v", "TustPortal", "/f"])
            .status()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cleanup() {
        let _ = Command::new("reg")
            .creation_flags(0x08000000)
            .args([
                "delete",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "TustPortal",
                "/f",
            ])
            .status();
    }

    #[test]
    fn is_enabled_returns_false_when_no_key() {
        let _guard = REGISTRY_MUTEX.lock().unwrap();
        cleanup();
        assert!(!is_enabled());
    }

    #[test]
    fn set_enabled_true_creates_key() {
        let _guard = REGISTRY_MUTEX.lock().unwrap();
        cleanup();
        set_enabled(true).unwrap();
        assert!(is_enabled());
        cleanup();
    }

    #[test]
    fn set_enabled_false_removes_key() {
        let _guard = REGISTRY_MUTEX.lock().unwrap();
        cleanup();
        set_enabled(true).unwrap();
        set_enabled(false).unwrap();
        assert!(!is_enabled());
    }
}
