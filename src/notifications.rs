use std::process::Command;

pub enum NotificationLevel {
    Error,
    Warning,
    #[allow(dead_code)]
    Info,
}
impl NotificationLevel {
    fn urgency(&self) -> &str {
        match self {
            NotificationLevel::Error => "critical",
            NotificationLevel::Warning => "normal",
            NotificationLevel::Info => "low",
        }
    }
}

/// Fires a desktop notification via notify-send.
/// Returns Ok(()) if notify-send ran, Err if the command failed.
pub fn notify(level: NotificationLevel, title: &str, message: &str) -> Result<(), String> {
    let output = Command::new("notify-send")
        .args([
            "--urgency",
            level.urgency(),
            "--app-name",
            "Todo Manager",
            title,
            message,
        ])
        .output()
        .map_err(|e| format!("notify-send failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("notify-send error: {}", stderr));
    }
    Ok(())
}

/// Convenience wrapper for Error-level notifications.
pub fn error(title: &str, message: &str) -> () {
    let _ = notify(NotificationLevel::Error, title, message);
}

/// Convenience wrapper for Warning-level notifications.
pub fn warning(title: &str, message: &str) -> () {
    let _ = notify(NotificationLevel::Warning, title, message);
}

/// Convenience wrapper for Info-level notifications.
#[allow(dead_code)]
pub fn info(title: &str, message: &str) -> () {
    let _ = notify(NotificationLevel::Info, title, message);
}
