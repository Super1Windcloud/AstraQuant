use log::{debug, error};

const APP_LOG_TARGET: &str = "astraquant::app";

#[tauri::command]
pub(crate) fn show_window(window: tauri::Window) -> Result<(), String> {
    if window.is_visible().unwrap() {
        debug!(
            target: APP_LOG_TARGET,
            "show_window skipped label={} already_visible=true",
            window.label()
        );
        return Ok(());
    }

    window.show().map_err(|error| {
        let message = format!("Failed to show window: {error}");
        error!(target: APP_LOG_TARGET, "{message}");
        message
    })?;

    debug!(
        target: APP_LOG_TARGET,
        "show_window success label={}",
        window.label()
    );

    Ok(())
}
