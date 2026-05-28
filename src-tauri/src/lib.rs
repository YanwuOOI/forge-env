mod commands;
mod core;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
        .run(tauri::generate_context!())
        .expect("error while running Forge Env");
}
