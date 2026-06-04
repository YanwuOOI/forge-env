use std::sync::MutexGuard;
use std::{env, path::PathBuf};

use log::error;
use tauri::{AppHandle, Emitter, State};

use crate::{
    core::{
        deps, detect, environment, execution, hosts, importer, mirrors,
        models::{
            AppPreferences, EnvPlan, ExportBundle, HostDetail, HostTemplateSnapshot, ImportAction,
            ImportResult, ProjectRuntimePolicyOptions, ProxySettings, ServiceArtifact,
            ServiceConfigState, ServiceState, SystemDependencyState, TemplateSnapshot,
        },
        project_policy, provider, secure_store, services,
    },
    state::SharedState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppCommandErrorKind {
    StateUnavailable,
    InvalidInput,
    SerializationFailed,
    EventEmitFailed,
}

#[derive(Debug, Clone)]
struct AppCommandError {
    kind: AppCommandErrorKind,
    message: String,
}

impl AppCommandError {
    fn new(kind: AppCommandErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    fn outcome_title(&self) -> &'static str {
        match self.kind {
            AppCommandErrorKind::StateUnavailable => "State unavailable",
            AppCommandErrorKind::InvalidInput => "Invalid request",
            AppCommandErrorKind::SerializationFailed => "Serialization failed",
            AppCommandErrorKind::EventEmitFailed => "Event delivery failed",
        }
    }

    fn next_step(&self) -> &'static str {
        match self.kind {
            AppCommandErrorKind::StateUnavailable => {
                "Retry once Forge Env can lock its in-memory state without contention."
            }
            AppCommandErrorKind::InvalidInput => {
                "Adjust the request payload or parameters, then retry."
            }
            AppCommandErrorKind::SerializationFailed => {
                "Retry export after removing malformed state or inspect the generated payload inputs."
            }
            AppCommandErrorKind::EventEmitFailed => {
                "Retry after the frontend bridge is ready to receive job updates."
            }
        }
    }

    fn to_message(&self) -> String {
        format!(
            "{}: {} {}",
            self.outcome_title(),
            self.message,
            self.next_step()
        )
    }
}

fn lock_memory_state<'a>(
    state: &'a SharedState,
) -> Result<MutexGuard<'a, crate::state::MemoryState>, String> {
    state.inner.lock().map_err(|_| {
        error!("Failed to acquire shared state lock — possible deadlock");
        AppCommandError::new(
            AppCommandErrorKind::StateUnavailable,
            "Failed to lock shared state.",
        )
        .to_message()
    })
}

fn service_job_record(
    state: &mut crate::state::MemoryState,
    label: String,
    service_name: &str,
    outcome_title: &str,
    outcome_detail: &str,
    next_step: &str,
) -> crate::core::models::JobRecord {
    state.structured_job(
        label,
        None,
        None,
        Some("service"),
        Some(service_name),
        Some(outcome_title),
        Some(outcome_detail),
        Some(next_step),
    )
}

fn runtime_job_record(
    state: &mut crate::state::MemoryState,
    label: String,
    family: &str,
    version: &str,
    outcome_title: &str,
    outcome_detail: &str,
    next_step: &str,
) -> crate::core::models::JobRecord {
    state.structured_job(
        label,
        Some(family),
        Some(version),
        Some("runtime"),
        Some(family),
        Some(outcome_title),
        Some(outcome_detail),
        Some(next_step),
    )
}

fn infra_job_record(
    state: &mut crate::state::MemoryState,
    label: String,
    category: &str,
    target_name: &str,
    outcome_title: &str,
    outcome_detail: &str,
    next_step: &str,
) -> crate::core::models::JobRecord {
    state.structured_job(
        label,
        None,
        None,
        Some(category),
        Some(target_name),
        Some(outcome_title),
        Some(outcome_detail),
        Some(next_step),
    )
}

fn failed_job_record(
    state: &mut crate::state::MemoryState,
    label: String,
    family: Option<&str>,
    version: Option<&str>,
    category: Option<&str>,
    target_name: Option<&str>,
    outcome_title: &str,
    outcome_detail: &str,
    next_step: &str,
) -> crate::core::models::JobRecord {
    state.failed_job(
        label,
        family,
        version,
        category,
        target_name,
        Some(outcome_title),
        Some(outcome_detail),
        Some(next_step),
    )
}

fn emit_jobs(app: &AppHandle, state: &SharedState) -> Result<(), String> {
    let jobs = lock_memory_state(state)?.jobs();
    app.emit("jobs://updated", jobs).map_err(|error| {
        error!("Failed to emit jobs://updated event: {error}");
        AppCommandError::new(AppCommandErrorKind::EventEmitFailed, error.to_string()).to_message()
    })
}

fn import_parse_failure_result(error: &str) -> ImportResult {
    ImportResult {
        accepted: false,
        ready_to_apply: false,
        runtime_count: 0,
        host_count: 0,
        planned_count: 0,
        applied_count: 0,
        issues: vec![format!("Invalid JSON bundle: {error}")],
        actions: vec![ImportAction {
            id: "import-parse-error".into(),
            host_id: "bundle".into(),
            host_label: "Bundle payload".into(),
            kind: "bundle".into(),
            family: None,
            status: "blocked".into(),
            selected: false,
            label: "Bundle parsing failed before Forge Env could generate an import plan."
                .into(),
            reason: Some(format!("Invalid JSON bundle: {error}")),
            outcome_title: Some("Invalid import bundle".into()),
            next_step: Some(
                "Fix the bundle JSON shape or re-export the template from Forge Env before retrying import."
                    .into(),
            ),
            remediation: Some(
                "Validate the JSON payload, then retry with a well-formed Forge Env export bundle."
                    .into(),
            ),
            repair_available: false,
        }],
    }
}

#[tauri::command]
pub fn hosts_list() -> Vec<crate::core::models::HostSummary> {
    hosts::discover_hosts()
}

#[tauri::command]
pub fn hosts_inspect(state: State<SharedState>, host_id: String) -> Result<HostDetail, String> {
    let detail = hosts::inspect_host(&host_id).map_err(|error| {
        format!(
            "{}: {} {}",
            error.outcome_title(),
            error.message,
            error.next_step()
        )
    })?;
    let mut state = lock_memory_state(&state)?;
    state.set_selected_host_id(&host_id);
    Ok(detail)
}

#[tauri::command]
pub fn projects_inspect(
    path: Option<String>,
) -> Result<Vec<crate::core::models::ProjectProfile>, String> {
    let root = path
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    Ok(detect::scan_projects(&root))
}

#[tauri::command]
pub fn project_runtime_apply(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    project_path: String,
    family: String,
    version: String,
    options: Option<ProjectRuntimePolicyOptions>,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match project_policy::apply_project_runtime_policy(
        &host_id,
        &project_path,
        &family,
        &version,
        options.clone(),
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Apply {} project policy", family),
                    Some(&family),
                    Some(&version),
                    Some("project-policy"),
                    Some(&family),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&family),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };

    let (title, detail, next_step) = match family.as_str() {
        ".NET" => (
            "Project SDK pinned",
            format!(
                "{} now contains a Forge Env managed global.json SDK pin for {}{}.",
                project_path,
                version,
                describe_project_policy_options(".NET", options.as_ref())
            ),
            "Reopen the project shell or rerun dotnet restore so the pinned SDK is picked up."
                .to_string(),
        ),
        "C/C++" => (
            "Project preset written",
            format!(
                "{} now contains a Forge Env CMakePresets.json compiler/build preset for {}{}.",
                project_path,
                version,
                describe_project_policy_options("C/C++", options.as_ref())
            ),
            "Configure the project through CMakePresets.json or rerun your configure step so the compiler hint takes effect."
                .to_string(),
        ),
        _ => (
            "Project policy written",
            format!("Forge Env wrote project policy for {}.", family),
            "Refresh project tooling so the new policy file is picked up.".to_string(),
        ),
    };

    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        infra_job_record(
            &mut state,
            outcome,
            "project-policy",
            &family,
            title,
            &detail,
            &next_step,
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

fn describe_project_policy_options(
    family: &str,
    options: Option<&ProjectRuntimePolicyOptions>,
) -> String {
    match (family, options) {
        (
            ".NET",
            Some(ProjectRuntimePolicyOptions {
                dotnet: Some(dotnet),
                ..
            }),
        ) => {
            let mut parts = Vec::new();
            if let Some(roll_forward) = dotnet.roll_forward.as_deref() {
                if !roll_forward.trim().is_empty() {
                    parts.push(format!("rollForward={}", roll_forward.trim()));
                }
            }
            if let Some(allow_prerelease) = dotnet.allow_prerelease {
                parts.push(format!("allowPrerelease={allow_prerelease}"));
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!(" ({})", parts.join(", "))
            }
        }
        ("C/C++", Some(ProjectRuntimePolicyOptions { cpp: Some(cpp), .. })) => {
            let mut parts = Vec::new();
            if let Some(generator) = cpp.generator.as_deref() {
                if !generator.trim().is_empty() {
                    parts.push(format!("generator={}", generator.trim()));
                }
            }
            if let Some(binary_dir) = cpp.binary_dir.as_deref() {
                if !binary_dir.trim().is_empty() {
                    parts.push(format!("binaryDir={}", binary_dir.trim()));
                }
            }
            if let Some(toolchain_file) = cpp.toolchain_file.as_deref() {
                if !toolchain_file.trim().is_empty() {
                    parts.push(format!("toolchainFile={}", toolchain_file.trim()));
                }
            }
            if let Some(build_type) = cpp.build_type.as_deref() {
                if !build_type.trim().is_empty() {
                    parts.push(format!("buildType={}", build_type.trim()));
                }
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!(" ({})", parts.join(", "))
            }
        }
        _ => String::new(),
    }
}

#[tauri::command]
pub fn runtimes_list(
    host_id: String,
) -> Result<Vec<crate::core::models::RuntimeFamilyState>, String> {
    Ok(provider::detect_runtimes_for_host(&host_id))
}

#[tauri::command]
pub fn runtimes_install(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    family: String,
    version: String,
    _scope: Option<String>,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match execution::execute_runtime_mutation(
        &host_id,
        &family,
        &version,
        execution::RuntimeMutation::Install,
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            error!("Runtime install failed: {family} {version} on {host_id}: {}", error.message);
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Install {} {}", family, version),
                    Some(&family),
                    Some(&version),
                    Some("runtime"),
                    Some(&family),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&family),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };

    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        runtime_job_record(
            &mut state,
            outcome,
            &family,
            &version,
            "Installed",
            &format!(
                "{} {} was installed on {} through its canonical provider.",
                family, version, host_id
            ),
            &format!(
                "Activate {} {} if it should become the default toolchain on this host.",
                family, version
            ),
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn runtimes_switch(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    family: String,
    version: String,
    _scope: Option<String>,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match execution::execute_runtime_mutation(
        &host_id,
        &family,
        &version,
        execution::RuntimeMutation::Activate,
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Activate {} {}", family, version),
                    Some(&family),
                    Some(&version),
                    Some("runtime"),
                    Some(&family),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&family),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };

    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        runtime_job_record(
            &mut state,
            outcome,
            &family,
            &version,
            "Activated",
            &format!(
                "{} {} was set as the active toolchain on {}.",
                family, version, host_id
            ),
            &format!(
                "Reopen project shells or refresh environment blocks if {} should win on PATH immediately.",
                family
            ),
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn runtimes_remove(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    family: String,
    version: String,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match execution::execute_runtime_mutation(
        &host_id,
        &family,
        &version,
        execution::RuntimeMutation::Remove,
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Remove {} {}", family, version),
                    Some(&family),
                    Some(&version),
                    Some("runtime"),
                    Some(&family),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&family),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };

    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        runtime_job_record(
            &mut state,
            outcome,
            &family,
            &version,
            "Removed",
            &format!(
                "{} {} was removed from the canonical provider on {}.",
                family, version, host_id
            ),
            &format!(
                "Install or reactivate another {} version if dependent projects still require it.",
                family
            ),
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn provider_bootstrap(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    family: String,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match execution::execute_provider_bootstrap(&host_id, &family) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Bootstrap canonical provider for {}", family),
                    Some(&family),
                    None,
                    Some("provider"),
                    Some(&family),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&family),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        infra_job_record(
            &mut state,
            outcome,
            "provider",
            &family,
            "Provider bootstrapped",
            &format!(
                "The canonical {} provider was bootstrapped on {}.",
                family, host_id
            ),
            &format!(
                "Retry the blocked {} action now that its provider path should exist.",
                family
            ),
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn deps_install(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    dependencies: Vec<String>,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match deps::install_dependency_template_for_host(&host_id, &dependencies) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    "Install base dependency template".into(),
                    None,
                    None,
                    Some("dependency"),
                    Some("base-template"),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step("base-template"),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        infra_job_record(
            &mut state,
            outcome,
            "dependency",
            "base-template",
            "Dependencies installed",
            &format!(
                "The base system dependency template was sent to the detected package manager on {}.",
                host_id
            ),
            "Verify any privileged package operations completed successfully in the host package manager output.",
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn services_list(host_id: String) -> Vec<ServiceState> {
    services::detect_services_for_host(&host_id)
}

#[tauri::command]
pub fn service_artifacts_list(host_id: String) -> Vec<ServiceArtifact> {
    services::detect_service_artifacts_for_host(&host_id)
}

#[tauri::command]
pub fn service_action(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    name: String,
    action: String,
) -> Result<crate::core::models::JobRecord, String> {
    let action_kind = match action.as_str() {
        "start" => services::ServiceAction::Start,
        "stop" => services::ServiceAction::Stop,
        "restart" => services::ServiceAction::Restart,
        other => {
            let command_error = AppCommandError::new(
                AppCommandErrorKind::InvalidInput,
                format!("Unsupported service action: {other}"),
            );
            let message = command_error.to_message();
            {
                let mut state = lock_memory_state(&state)?;
                failed_job_record(
                    &mut state,
                    format!("Invalid service action for {}", name),
                    None,
                    None,
                    Some("service"),
                    Some(&name),
                    command_error.outcome_title(),
                    &message,
                    command_error.next_step(),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(message);
        }
    };

    let outcome = match services::control_service_for_host(&host_id, &name, action_kind) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("{} {}", action, name),
                    None,
                    None,
                    Some("service"),
                    Some(&name),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&name),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        let (title, detail, next_step) = match action_kind {
            services::ServiceAction::Start => (
                "Started",
                format!("{name} was started on {host_id} through the detected host control path."),
                format!("Verify that {name} is reachable on its expected local port."),
            ),
            services::ServiceAction::Stop => (
                "Stopped",
                format!("{name} was stopped on {host_id}."),
                format!("Start {name} again when dependent projects need the service online."),
            ),
            services::ServiceAction::Restart => (
                "Restarted",
                format!("{name} was restarted on {host_id} to reload the current service state."),
                format!(
                    "Verify that {name} came back on its expected local port without bind errors."
                ),
            ),
        };
        service_job_record(&mut state, outcome, &name, title, &detail, &next_step)
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn services_config_list(host_id: String) -> Vec<ServiceConfigState> {
    services::detect_service_configs_for_host(&host_id)
}

#[tauri::command]
pub fn service_config_apply(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    name: String,
    port: Option<u16>,
    data_dir: Option<String>,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match services::apply_service_config_for_host(&host_id, &name, port, data_dir) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Apply {} config", name),
                    None,
                    None,
                    Some("service"),
                    Some(&name),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&name),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        service_job_record(
            &mut state,
            outcome,
            &name,
            "Config staged",
            &format!(
                "{} config overrides were written on {} without restarting the service process.",
                name, host_id
            ),
            &format!(
                "Use Apply + restart or restart {} manually when the new settings should take effect.",
                name
            ),
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn service_config_apply_and_restart(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    name: String,
    port: Option<u16>,
    data_dir: Option<String>,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match services::apply_service_config_and_reconcile_for_host(
        &host_id, &name, port, data_dir,
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Apply {} config and reconcile service", name),
                    None,
                    None,
                    Some("service"),
                    Some(&name),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&name),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        let running_after = services::detect_services_for_host(&host_id)
            .into_iter()
            .find(|service| service.name == name)
            .map(|service| service.running)
            .unwrap_or(false);
        let title = if running_after {
            "Applied + reconciled"
        } else {
            "Config applied"
        };
        let detail = if running_after {
            format!(
                "{} overrides were written on {}, and Forge Env then brought the service online with the new settings.",
                name, host_id
            )
        } else {
            format!(
                "{} overrides were written on {}, but the service did not report a running state afterward.",
                name, host_id
            )
        };
        let next_step = if running_after {
            format!(
                "Verify that {} is reachable on its expected local port and that clients reconnect cleanly.",
                name
            )
        } else {
            format!(
                "Check host permissions or service logs if {} should have restarted automatically.",
                name
            )
        };
        service_job_record(&mut state, outcome, &name, title, &detail, &next_step)
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn service_backup_create(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    name: String,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match services::create_service_backup_for_host(&host_id, &name) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Create {} snapshot backup", name),
                    None,
                    None,
                    Some("service"),
                    Some(&name),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&name),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        service_job_record(
            &mut state,
            outcome,
            &name,
            "Snapshot backup created",
            &format!(
                "{} now has a managed data-dir snapshot archive on {}.",
                name, host_id
            ),
            "Keep the archive path for later restore or export workflows, and restart the service only after you no longer need it offline.",
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn service_data_export(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    name: String,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match services::export_service_data_for_host(&host_id, &name) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Export {} data directory", name),
                    None,
                    None,
                    Some("service"),
                    Some(&name),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&name),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        service_job_record(
            &mut state,
            outcome,
            &name,
            "Data dir exported",
            &format!(
                "{} data-dir contents were archived into a managed export path on {}.",
                name, host_id
            ),
            "Use the export artifact outside Forge Env or keep it as a portable archive for later restore.",
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn service_backup_restore(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    name: String,
    archive_path: Option<String>,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match services::restore_service_backup_for_host(&host_id, &name, archive_path) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Restore {} snapshot backup", name),
                    None,
                    None,
                    Some("service"),
                    Some(&name),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&name),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        service_job_record(
            &mut state,
            outcome,
            &name,
            "Snapshot restored",
            &format!(
                "{} data-dir contents were restored on {} from the selected archive.",
                name, host_id
            ),
            "Inspect the pre-restore directory if you need to compare state, then start the service when the restored snapshot should go live.",
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn service_logical_backup_create(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    name: String,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match services::create_service_logical_backup_for_host(&host_id, &name) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Create {} logical backup", name),
                    None,
                    None,
                    Some("service"),
                    Some(&name),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&name),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        service_job_record(
            &mut state,
            outcome,
            &name,
            "Logical backup created",
            &format!(
                "{} now has a managed logical backup artifact on {}.",
                name, host_id
            ),
            "Validate the artifact before depending on it for restore or off-host archive workflows.",
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn service_artifact_validate(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    path: String,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match services::validate_service_artifact_for_host(&host_id, &path) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                let next_step = error.next_step("artifact");
                failed_job_record(
                    &mut state,
                    format!("Validate service artifact {}", path),
                    None,
                    None,
                    Some("service"),
                    Some(&path),
                    error.outcome_title(),
                    &error.message,
                    &next_step,
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        infra_job_record(
            &mut state,
            outcome,
            "service",
            &path,
            "Artifact validated",
            &format!("Forge Env validated the selected service artifact on {}.", host_id),
            "Keep the artifact if validation passed, or delete and recreate it if you no longer trust the archive.",
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn service_artifact_delete(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    path: String,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match services::delete_service_artifact_for_host(&host_id, &path) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                let next_step = error.next_step("artifact");
                failed_job_record(
                    &mut state,
                    format!("Delete service artifact {}", path),
                    None,
                    None,
                    Some("service"),
                    Some(&path),
                    error.outcome_title(),
                    &error.message,
                    &next_step,
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        infra_job_record(
            &mut state,
            outcome,
            "service",
            &path,
            "Artifact deleted",
            "Forge Env removed the selected managed service artifact.",
            "Refresh service artifacts if you need to confirm the remaining archive inventory.",
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn mirrors_apply(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    mirror_set: String,
) -> Result<crate::core::models::JobRecord, String> {
    let changes = match mirrors::apply_preset_for_host(&host_id, &mirror_set) {
        Ok(changes) => changes,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Apply mirror preset {}", mirror_set),
                    None,
                    None,
                    Some("mirror"),
                    Some(&mirror_set),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&mirror_set),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let label = format!(
        "Apply mirror preset {} on {host_id}: {}",
        mirror_set,
        changes.join(" | ")
    );
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        state.set_mirror_preset(&mirror_set);
        infra_job_record(
            &mut state,
            label,
            "mirror",
            &mirror_set,
            "Mirror preset applied",
            &format!(
                "Mirror preset {} was written for supported package ecosystems on {}.",
                mirror_set, host_id
            ),
            "Run a package install or metadata fetch to confirm the new mirror endpoints resolve correctly.",
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn env_export(
    state: State<SharedState>,
    host_id: Option<String>,
) -> Result<ExportBundle, String> {
    let state = lock_memory_state(&state)?;
    let hosts = hosts::discover_hosts();
    let selected_host_id = host_id
        .or_else(|| state.selected_host_id())
        .unwrap_or_else(|| "native".into());
    let export_host_id = hosts
        .iter()
        .find(|host| host.id == selected_host_id)
        .map(|host| host.id.clone())
        .unwrap_or_else(|| "native".into());
    let host_snapshots = hosts
        .iter()
        .cloned()
        .map(|host| HostTemplateSnapshot {
            runtimes: provider::detect_runtimes_for_host(&host.id),
            system_dependencies: deps::detect_system_dependencies_for_host(&host.id),
            host,
        })
        .collect::<Vec<_>>();

    let snapshot = TemplateSnapshot {
        schema_version: 2,
        generated_at: format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ),
        hosts,
        host_snapshots,
        runtimes: if export_host_id == "native" {
            provider::detect_runtimes()
        } else {
            provider::detect_runtimes_for_host(&export_host_id)
        },
        system_dependencies: if export_host_id == "native" {
            deps::detect_system_dependencies()
        } else {
            deps::detect_system_dependencies_for_host(&export_host_id)
        },
        applied_mirror_preset: state.applied_mirror_preset(),
    };

    let payload = serde_json::to_string_pretty(&snapshot).map_err(|error| {
        AppCommandError::new(AppCommandErrorKind::SerializationFailed, error.to_string())
            .to_message()
    })?;

    Ok(ExportBundle {
        file_name: "forge-env-template.json".into(),
        payload,
    })
}

#[tauri::command]
pub fn env_import(payload: String) -> Result<ImportResult, String> {
    let parsed: Result<TemplateSnapshot, _> = serde_json::from_str(&payload);

    match parsed {
        Ok(snapshot) => Ok(importer::plan_import(&snapshot).result),
        Err(error) => Ok(import_parse_failure_result(&error.to_string())),
    }
}

#[tauri::command]
pub fn env_import_apply(
    app: AppHandle,
    state: State<SharedState>,
    payload: String,
    selected_action_ids: Option<Vec<String>>,
) -> Result<ImportResult, String> {
    let parsed: TemplateSnapshot = match serde_json::from_str(&payload) {
        Ok(parsed) => parsed,
        Err(error) => {
            let command_error = AppCommandError::new(
                AppCommandErrorKind::InvalidInput,
                format!("Invalid JSON bundle: {error}"),
            );
            let result = import_parse_failure_result(&error.to_string());
            {
                let mut state = lock_memory_state(&state)?;
                failed_job_record(
                    &mut state,
                    "Parse import bundle".into(),
                    None,
                    None,
                    Some("import"),
                    Some("bundle"),
                    command_error.outcome_title(),
                    &command_error.to_message(),
                    command_error.next_step(),
                );
            }
            emit_jobs(&app, &state)?;
            return Ok(result);
        }
    };
    let preset = parsed.applied_mirror_preset.clone();
    let plan = importer::plan_import(&parsed);
    let selected = selected_action_ids.and_then(|ids| {
        let filtered = ids.into_iter().collect::<std::collections::HashSet<_>>();
        (!filtered.is_empty()).then_some(filtered)
    });
    let result = importer::apply_planned_import(plan, selected);

    {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        if let Some(preset) = preset.as_deref() {
            state.set_mirror_preset(preset);
        }
        for action in &result.actions {
            if action.status != "applied" && action.status != "failed" {
                continue;
            }

            let label = action
                .reason
                .clone()
                .unwrap_or_else(|| action.label.clone());
            let target_name = action.family.as_deref().unwrap_or(action.kind.as_str());
            let detail = action
                .reason
                .clone()
                .unwrap_or_else(|| action.label.clone());
            let outcome_title =
                action
                    .outcome_title
                    .as_deref()
                    .unwrap_or(if action.status == "applied" {
                        "Import action applied"
                    } else {
                        "Import action failed"
                    });
            let next_step = action.next_step.as_deref().unwrap_or(
                "Review any remaining planned or blocked actions before considering the host fully reconstructed.",
            );

            if action.status == "applied" {
                state.structured_job(
                    label,
                    action.family.as_deref(),
                    None,
                    Some("import"),
                    Some(target_name),
                    Some(outcome_title),
                    Some(&detail),
                    Some(next_step),
                );
            } else {
                state.failed_job(
                    label,
                    action.family.as_deref(),
                    None,
                    Some("import"),
                    Some(target_name),
                    Some(outcome_title),
                    Some(&detail),
                    Some(next_step),
                );
            }
        }
    }

    emit_jobs(&app, &state)?;
    Ok(result)
}

#[tauri::command]
pub fn app_preferences(state: State<SharedState>) -> Result<AppPreferences, String> {
    let state = lock_memory_state(&state)?;
    Ok(AppPreferences {
        applied_mirror_preset: state.applied_mirror_preset(),
        selected_host_id: state.selected_host_id(),
        last_env_target_profile: state.last_env_target_profile(),
    })
}

#[tauri::command]
pub fn env_preview(state: State<SharedState>, host_id: String) -> Result<EnvPlan, String> {
    let state = lock_memory_state(&state)?;
    let runtimes = provider::detect_runtimes_for_host(&host_id);
    let preferred_profile = state.last_env_target_profile();
    let profiles = hosts::detect_shell_profiles_for_host(&host_id);
    drop(state);
    let mut plan = environment::build_env_plan(&host_id, &runtimes, &profiles)
        .map_err(|error| error.message)?;
    if let Some(profile) = preferred_profile {
        plan.target_profile = profile;
    }
    Ok(plan)
}

#[tauri::command]
pub fn env_apply(
    app: AppHandle,
    state: State<SharedState>,
    host_id: String,
    profile_path: String,
) -> Result<crate::core::models::JobRecord, String> {
    let plan = {
        let state_guard = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        let runtimes = provider::detect_runtimes_for_host(&host_id);
        drop(state_guard);
        match environment::build_env_plan(
            &host_id,
            &runtimes,
            &hosts::detect_shell_profiles_for_host(&host_id),
        ) {
            Ok(plan) => plan,
            Err(error) => {
                {
                    let mut state = state
                        .inner
                        .lock()
                        .map_err(|_| "Failed to lock shared state.".to_string())?;
                    failed_job_record(
                        &mut state,
                        format!("Prepare Forge Env shell block for {}", profile_path),
                        None,
                        None,
                        Some("environment"),
                        Some(&profile_path),
                        error.outcome_title(),
                        &error.message,
                        &error.next_step(&profile_path),
                    );
                }
                emit_jobs(&app, &state)?;
                return Err(error.message);
            }
        }
    };

    let outcome = match environment::apply_env_plan_for_host(&host_id, &plan, &profile_path) {
        Ok(outcome) => outcome,
        Err(error) => {
            {
                let mut state = state
                    .inner
                    .lock()
                    .map_err(|_| "Failed to lock shared state.".to_string())?;
                failed_job_record(
                    &mut state,
                    format!("Apply Forge Env shell block to {}", profile_path),
                    None,
                    None,
                    Some("environment"),
                    Some(&profile_path),
                    error.outcome_title(),
                    &error.message,
                    &error.next_step(&profile_path),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock shared state.".to_string())?;
        state.set_last_env_target_profile(&profile_path);
        infra_job_record(
            &mut state,
            outcome,
            "environment",
            &profile_path,
            "Environment block applied",
            &format!(
                "Forge Env wrote the managed shell block to {} for host {}.",
                profile_path, host_id
            ),
            "Open a new shell or source the updated profile so PATH and runtime variables take effect.",
        )
    };

    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn jobs_subscribe(
    state: State<SharedState>,
) -> Result<Vec<crate::core::models::JobRecord>, String> {
    let state = state
        .inner
        .lock()
        .map_err(|_| "Failed to lock shared state.".to_string())?;
    Ok(state.jobs())
}

#[tauri::command]
pub fn deps_list(host_id: String) -> Vec<SystemDependencyState> {
    deps::detect_system_dependencies_for_host(&host_id)
}

#[tauri::command]
pub fn proxy_settings_load() -> Result<ProxySettings, String> {
    secure_store::load_proxy_settings().map_err(|error| {
        format!(
            "{}: {} {}",
            error.outcome_title(),
            error.message,
            error.next_step()
        )
    })
}

#[tauri::command]
pub fn proxy_settings_save(
    app: AppHandle,
    state: State<SharedState>,
    settings: ProxySettings,
    password: Option<String>,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match secure_store::save_proxy_settings(settings.clone(), password) {
        Ok(_) => format!(
            "Saved proxy profile for {}:{} and updated the {} credential entry when a password was supplied.",
            settings.host, settings.port, settings.secure_store
        ),
        Err(error) => {
            {
                let mut state = lock_memory_state(&state)?;
                failed_job_record(
                    &mut state,
                    "Save proxy settings".into(),
                    None,
                    None,
                    Some("security"),
                    Some("proxy"),
                    error.outcome_title(),
                    &error.message,
                    error.next_step(),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = lock_memory_state(&state)?;
        infra_job_record(
            &mut state,
            outcome,
            "security",
            "proxy",
            "Proxy profile saved",
            "Forge Env stored non-secret proxy settings locally and routed the password to the system credential store when supplied.",
            "Apply the company proxy preset or inspect host network behavior when this proxy should become active.",
        )
    };
    emit_jobs(&app, &state)?;
    Ok(job)
}

#[tauri::command]
pub fn proxy_settings_clear(
    app: AppHandle,
    state: State<SharedState>,
) -> Result<crate::core::models::JobRecord, String> {
    let outcome = match secure_store::clear_proxy_settings() {
        Ok(_) => {
            "Cleared the stored proxy profile and removed any managed system credential entry."
                .to_string()
        }
        Err(error) => {
            {
                let mut state = lock_memory_state(&state)?;
                failed_job_record(
                    &mut state,
                    "Clear proxy settings".into(),
                    None,
                    None,
                    Some("security"),
                    Some("proxy"),
                    error.outcome_title(),
                    &error.message,
                    error.next_step(),
                );
            }
            emit_jobs(&app, &state)?;
            return Err(error.message);
        }
    };
    let job = {
        let mut state = lock_memory_state(&state)?;
        infra_job_record(
            &mut state,
            outcome,
            "security",
            "proxy",
            "Proxy profile cleared",
            "Forge Env removed the local proxy profile and deleted any managed password entry from the system store.",
            "Re-enter proxy settings only when the host should route traffic through a managed proxy again.",
        )
    };
    emit_jobs(&app, &state)?;
    Ok(job)
}
