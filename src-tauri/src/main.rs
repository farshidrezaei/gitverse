#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() >= 4 && arguments[1] == "--sequence-editor" {
        let source_todo = &arguments[2];
        let target_todo = &arguments[3];
        if let Err(error) = std::fs::copy(source_todo, target_todo) {
            eprintln!(
                "Failed to copy rebase todo file from {} to {}: {}",
                source_todo, target_todo, error
            );
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    gitverse_lib::run();
}
