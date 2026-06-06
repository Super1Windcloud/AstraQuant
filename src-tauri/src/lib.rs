mod logging;
mod market_data;
mod window;

use log::info;
use tauri::{self, Manager};

const APP_LOG_TARGET: &str = "astraquant::app";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let (_, max_level, logger) = logging::build_log_plugin().split(app.handle())?;
            tauri_plugin_log::attach_logger(
                max_level,
                Box::new(logging::DedupLogger::new(logger)),
            )?;

            info!(
                target: APP_LOG_TARGET,
                "tauri setup start package={} version={}",
                app.package_info().name,
                app.package_info().version
            );

            if let (Some(window), Some(icon)) =
                (app.get_webview_window("main"), app.default_window_icon())
            {
                window.set_icon(icon.clone())?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            window::show_window,
            market_data::service::get_market_snapshot,
            market_data::service::get_indices_overview
        ])
        .plugin(logging::build_log_plugin().skip_logger().build())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
