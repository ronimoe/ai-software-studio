pub mod artifacts;
pub mod commands;
pub mod config;
pub mod core;
pub mod db;
pub mod engines;
pub mod error;
pub mod fixtures;
pub mod git;
pub mod models;
pub mod policy;
pub mod process;
pub mod projects;
pub mod state;
pub mod tasks;
pub mod verification;

use state::AppState;
use tauri_specta::{collect_commands, Builder};

pub fn run() {
    let specta_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::projects::list_projects,
            commands::tasks::list_tasks,
            commands::tasks::get_task,
            commands::engines::list_engines,
            commands::engines::detect_engines,
            commands::verification::list_verification,
        ]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(
            specta_typescript::Typescript::default().formatter(specta_typescript::formatter::prettier),
            "../lib/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod export_bindings_test {
    use super::*;

    #[test]
    fn export_bindings() {
        let builder = Builder::<tauri::Wry>::new()
            .commands(collect_commands![
                commands::projects::list_projects,
                commands::tasks::list_tasks,
                commands::tasks::get_task,
                commands::engines::list_engines,
                commands::engines::detect_engines,
                commands::verification::list_verification,
            ]);
        builder
            .export(specta_typescript::Typescript::default(), "../lib/bindings.ts")
            .expect("export bindings");
    }
}
