mod commands;
mod core;
mod state;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Build tray menu
            let show = MenuItemBuilder::with_id("show", "Show Forge Env").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Forge Env — Development Environment Manager")
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            Ok(())
        })
        .manage(state::SharedState::new())
        .invoke_handler(tauri::generate_handler![
            commands::hosts_list,
            commands::hosts_inspect,
            commands::projects_inspect,
            commands::project_runtime_apply,
            commands::runtimes_list,
            commands::runtimes_install,
            commands::runtimes_switch,
            commands::runtimes_remove,
            commands::provider_bootstrap,
            commands::deps_install,
            commands::services_list,
            commands::service_artifacts_list,
            commands::service_action,
            commands::services_config_list,
            commands::service_config_apply,
            commands::service_config_apply_and_restart,
            commands::service_backup_create,
            commands::service_data_export,
            commands::service_backup_restore,
            commands::service_logical_backup_create,
            commands::service_artifact_validate,
            commands::service_artifact_delete,
            commands::mirrors_apply,
            commands::env_export,
            commands::env_import,
            commands::env_import_apply,
            commands::app_preferences,
            commands::env_preview,
            commands::env_apply,
            commands::jobs_subscribe,
            commands::deps_list,
            commands::proxy_settings_load,
            commands::proxy_settings_save,
            commands::proxy_settings_clear
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Minimize to tray instead of closing
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Forge Env");
}
