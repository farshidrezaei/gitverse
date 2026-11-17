pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;
pub mod services;

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
        .plugin(tauri_plugin_dialog::init())
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
            commands::get_repository_status,
            commands::stage_file,
            commands::unstage_file,
            commands::stage_all,
            commands::unstage_all,
            commands::discard_file_changes,
            commands::commit,
            commands::get_file_diff,
            commands::get_commit_graph,
            commands::get_repository_references,
            commands::checkout_branch,
            commands::create_branch,
            commands::delete_branch,
            commands::fetch_remote,
            commands::pull_changes,
            commands::push_changes,
            commands::get_commit_details,
            commands::get_commit_file_diff,
            commands::get_recent_repositories,
            commands::remove_recent_repository,
            commands::get_stash_list,
            commands::save_stash,
            commands::apply_stash,
            commands::pop_stash,
            commands::drop_stash,
            commands::cherry_pick_commit,
            commands::revert_commit,
            commands::reset_to_commit,
            commands::get_conflict_file_details,
            commands::resolve_conflict_file,
            commands::write_conflict_resolved_content,
            commands::abort_repository_operation,
            commands::continue_repository_operation,
            commands::merge_branch,
            commands::rebase_branch,
        ])
        .run(tauri::generate_context!());

    if let Err(error) = application_result {
        tracing::error!("Failed to launch GitVerse application: {error}");
    }
}
