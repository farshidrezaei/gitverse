use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use tauri::{AppHandle, Emitter};

pub struct RepositoryWatcherManager {
    is_watching: Arc<AtomicBool>,
    current_path: Arc<Mutex<Option<PathBuf>>>,
}

impl RepositoryWatcherManager {
    pub fn new() -> Self {
        Self {
            is_watching: Arc::new(AtomicBool::new(false)),
            current_path: Arc::new(Mutex::new(None)),
        }
    }

    /// Starts watching the specified repository path and emits 'repository://status-changed' on file modifications.
    pub fn watch_repository(&self, app_handle: AppHandle, repository_path: &Path) {
        let repo_path = repository_path.to_path_buf();
        {
            if let Ok(mut guard) = self.current_path.lock() {
                *guard = Some(repo_path.clone());
            }
        }

        self.is_watching.store(true, Ordering::SeqCst);
        let is_watching_clone = self.is_watching.clone();
        let target_path_clone = self.current_path.clone();

        std::thread::spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel();

            let mut debouncer = match new_debouncer(Duration::from_millis(300), tx) {
                Ok(debouncer) => debouncer,
                Err(error) => {
                    tracing::error!("Failed to create file debouncer: {:?}", error);
                    return;
                }
            };

            if let Err(error) = debouncer.watcher().watch(&repo_path, RecursiveMode::Recursive) {
                tracing::error!("Failed to watch repository directory {:?}: {:?}", repo_path, error);
                return;
            }

            tracing::info!("Started real-time file watcher on {:?}", repo_path);

            while is_watching_clone.load(Ordering::SeqCst) {
                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(DebounceEventResult::Ok(events)) => {
                        let has_relevant_change = events.iter().any(|event| {
                            let path_str = event.path.to_string_lossy();
                            !path_str.contains(".git/objects") && !path_str.contains(".git/logs")
                        });

                        if has_relevant_change {
                            tracing::debug!("Detected file change, emitting status event to frontend");
                            let _ = app_handle.emit("repository://status-changed", ());
                        }
                    }
                    Ok(DebounceEventResult::Err(errors)) => {
                        tracing::warn!("Watcher error events: {:?}", errors);
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        if let Ok(guard) = target_path_clone.lock() {
                            if let Some(ref current) = *guard {
                                if current != &repo_path {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }

            tracing::info!("Stopped file watcher on {:?}", repo_path);
        });
    }

    pub fn stop_watching(&self) {
        self.is_watching.store(false, Ordering::SeqCst);
        if let Ok(mut guard) = self.current_path.lock() {
            *guard = None;
        }
    }
}

