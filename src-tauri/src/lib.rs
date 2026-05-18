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

#[cfg(not(test))]
use state::AppState;
#[cfg(not(test))]
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

#[cfg(not(test))]
pub fn run() {
    let specta_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::projects::list_projects,
            commands::projects::open_project,
            commands::tasks::list_tasks,
            commands::tasks::get_task,
            commands::tasks::create_task,
            commands::engines::list_engines,
            commands::engines::detect_engines,
            commands::verification::list_verification,
            commands::worktrees::create_worktree,
            commands::worktrees::remove_worktree,
        ]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(
            specta_typescript::Typescript::default().formatter(specta_typescript::formatter::prettier),
            "../lib/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
            let state = tauri::async_runtime::block_on(AppState::init())
                .map_err(|e| format!("AppState::init failed: {e}"))?;
            app.manage(state);
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
                commands::projects::open_project,
                commands::tasks::list_tasks,
                commands::tasks::get_task,
                commands::tasks::create_task,
                commands::engines::list_engines,
                commands::engines::detect_engines,
                commands::verification::list_verification,
                commands::worktrees::create_worktree,
                commands::worktrees::remove_worktree,
            ]);
        builder
            .export(specta_typescript::Typescript::default(), "../lib/bindings.ts")
            .expect("export bindings");
    }
}
