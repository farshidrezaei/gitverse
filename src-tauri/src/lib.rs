use tracing_subscriber::EnvFilter;

/// Configures and launches the Tauri desktop application runtime.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("gitverse_lib=info,tauri=info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    let application_result = tauri::Builder::default()
        .setup(|_application| {
            tracing::info!("GitVerse desktop application runtime initialized.");
            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(error) = application_result {
        tracing::error!("Failed to launch GitVerse application: {error}");
    }
}
