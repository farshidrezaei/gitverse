pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

#[cfg(test)]
mod lib_test;

use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tracing_subscriber::EnvFilter;
use application::ApplicationState;

/// Configures and launches the Tauri desktop application runtime.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("gitverse_lib=info,tauri=info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    let application_result = tauri::Builder::default()
        .setup(|application| {
            tracing::info!("GitVerse desktop application runtime initializing...");

            // Resolve local app data directory for SQLite persistence
            let app_data_directory = application
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));

            if !app_data_directory.exists() {
                let _ = fs::create_dir_all(&app_data_directory);
            }

            let database_path = app_data_directory.join("gitverse_workspace.sqlite");
            let state = ApplicationState::new(&database_path).map_err(|error| {
                tracing::error!("Failed to initialize ApplicationState: {:?}", error);
                Box::new(error) as Box<dyn std::error::Error>
            })?;

            application.manage(state);
            tracing::info!("GitVerse application state managed successfully.");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::discover_git_binary,
            commands::open_repository,
            commands::get_current_repository,
            commands::close_repository,
            commands::get_recent_repositories,
            commands::remove_recent_repository,
        ])
        .run(tauri::generate_context!());

    if let Err(error) = application_result {
        tracing::error!("Failed to launch GitVerse application: {error}");
    }
}
