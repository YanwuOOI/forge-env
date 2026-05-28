use std::collections::{HashMap, HashSet};

use super::{
    deps, execution, hosts, mirrors,
    models::{
        HostSummary, HostTemplateSnapshot, ImportAction, ImportResult, SystemDependencyState,
        TemplateSnapshot,
    },
    provider,
};

#[derive(Debug, Clone)]
pub enum ImportOperation {
    ApplyMirrorPreset {
        host_id: String,
        preset: String,
    },
    InstallDependencies {
        host_id: String,
        names: Vec<String>,
    },
    InstallRuntime {
        host_id: String,
        family: String,
        version: String,
    },
    ActivateRuntime {
        host_id: String,
        family: String,
        version: String,
    },
}

#[derive(Debug, Clone)]
pub struct PlannedImportAction {
    pub action_id: String,
    pub operation: ImportOperation,
}

#[derive(Debug, Clone)]
pub struct PlannedImport {
    pub result: ImportResult,
    pub operations: Vec<PlannedImportAction>,
}

#[derive(Debug, Clone)]
struct ImportExecutionOutcome {
    detail: String,
    outcome_title: String,
    next_step: String,
}

#[derive(Debug, Clone)]
struct ImportExecutionError {
    detail: String,
    outcome_title: String,
    next_step: String,
}

pub fn plan_import(snapshot: &TemplateSnapshot) -> PlannedImport {
    let mut issues = validate_snapshot(snapshot);
    let accepted = issues.is_empty();
    let host_snapshots = expand_host_snapshots(snapshot);
    let runtime_count = host_snapshots.iter().map(|host| host.runtimes.len()).sum();
    let host_count = host_snapshots.len();

    let mut actions = Vec::new();
    let mut operations = Vec::new();
    let current_hosts = hosts::discover_hosts();
    let current_host_ids = current_hosts
        .iter()
        .map(|host| (host.id.clone(), host.clone()))
        .collect::<HashMap<_, _>>();

    for host_snapshot in &host_snapshots {
        let Some(local_host) = current_host_ids.get(&host_snapshot.host.id) else {
            push_action(
                &mut actions,
                &host_snapshot.host,
                "host-unavailable",
                None,
                "blocked",
                format!(
                    "Block {} because the matching local host was not found",
                    host_snapshot.host.label
                ),
                Some("Host mismatch".into()),
                Some(
                    "Open this bundle on a machine with the same host topology, or export again from the target host."
                        .into(),
                ),
                false,
            );
            continue;
        };

        plan_host_actions(
            snapshot,
            host_snapshot,
            local_host,
            &mut actions,
            &mut operations,
            &mut issues,
        );
    }

    let planned_count = actions
        .iter()
        .filter(|action| action.status == "planned")
        .count();
    let applied_count = actions
        .iter()
        .filter(|action| action.status == "applied")
        .count();
    let ready_to_apply = accepted && planned_count > 0;

    PlannedImport {
        result: ImportResult {
            accepted,
            ready_to_apply,
            runtime_count,
            host_count,
            planned_count,
            applied_count,
            issues,
            actions,
        },
        operations,
    }
}

pub fn apply_planned_import(
    plan: PlannedImport,
    selected_action_ids: Option<HashSet<String>>,
) -> ImportResult {
    let mut result = plan.result;

    if !result.accepted {
        return result;
    }

    for planned in plan.operations {
        let should_apply = selected_action_ids
            .as_ref()
            .is_none_or(|selected| selected.contains(&planned.action_id));
        if !should_apply {
            if let Some(action) = result
                .actions
                .iter_mut()
                .find(|action| action.id == planned.action_id)
            {
                action.selected = false;
            }
            continue;
        }

        let execution_result: Result<ImportExecutionOutcome, ImportExecutionError> =
            match &planned.operation {
            ImportOperation::ApplyMirrorPreset { host_id, preset } => {
                mirrors::apply_preset_for_host(host_id, preset)
                    .map(|changes| ImportExecutionOutcome {
                        detail: format!(
                            "Applied mirror preset {} on {}. {}",
                            preset,
                            host_id,
                            changes.join(" | ")
                        ),
                        outcome_title: "Import action applied".into(),
                        next_step:
                            "Review any remaining planned or blocked actions before considering the host fully reconstructed."
                                .into(),
                    })
                    .map_err(|error| {
                        let outcome_title = error.outcome_title().to_string();
                        let next_step = error.next_step("mirror preset");
                        let detail = error.message;
                        ImportExecutionError {
                            detail,
                            outcome_title,
                            next_step,
                        }
                    })
            }
            ImportOperation::InstallDependencies { host_id, names } => {
                deps::install_dependency_template_for_host(host_id, names)
                    .map(|detail| ImportExecutionOutcome {
                        detail,
                        outcome_title: "Import action applied".into(),
                        next_step:
                            "Review any remaining planned or blocked actions before considering the host fully reconstructed."
                                .into(),
                    })
                    .map_err(|error| {
                        let outcome_title = error.outcome_title().to_string();
                        let next_step = error.next_step("base-template");
                        let detail = error.message;
                        ImportExecutionError {
                            detail,
                            outcome_title,
                            next_step,
                        }
                    })
            }
            ImportOperation::InstallRuntime {
                host_id,
                family,
                version,
            } => execution::execute_runtime_mutation(
                host_id,
                family,
                version,
                execution::RuntimeMutation::Install,
            )
            .map(|detail| ImportExecutionOutcome {
                detail,
                outcome_title: "Import action applied".into(),
                next_step:
                    "Review any remaining planned or blocked actions before considering the host fully reconstructed."
                        .into(),
            })
            .map_err(|error| {
                let outcome_title = error.outcome_title().to_string();
                let next_step = error.next_step(family);
                let detail = error.message;
                ImportExecutionError {
                    detail,
                    outcome_title,
                    next_step,
                }
            }),
            ImportOperation::ActivateRuntime {
                host_id,
                family,
                version,
            } => execution::execute_runtime_mutation(
                host_id,
                family,
                version,
                execution::RuntimeMutation::Activate,
            )
            .map(|detail| ImportExecutionOutcome {
                detail,
                outcome_title: "Import action applied".into(),
                next_step:
                    "Review any remaining planned or blocked actions before considering the host fully reconstructed."
                        .into(),
            })
            .map_err(|error| {
                let outcome_title = error.outcome_title().to_string();
                let next_step = error.next_step(family);
                let detail = error.message;
                ImportExecutionError {
                    detail,
                    outcome_title,
                    next_step,
                }
            }),
        };

        let action = result
            .actions
            .iter_mut()
            .find(|action| action.id == planned.action_id);

        match execution_result {
            Ok(outcome) => {
                if let Some(action) = action {
                    action.status = "applied".into();
                    action.selected = false;
                    action.reason = Some(outcome.detail);
                    action.outcome_title = Some(outcome.outcome_title);
                    action.next_step = Some(outcome.next_step);
                }
            }
            Err(error) => {
                if let Some(action) = action {
                    action.status = "failed".into();
                    action.selected = true;
                    action.reason = Some(error.detail.clone());
                    action.outcome_title = Some(error.outcome_title.clone());
                    action.next_step = Some(error.next_step.clone());
                }
                result.issues.push(error.detail);
            }
        }
    }

    result.applied_count = result
        .actions
        .iter()
        .filter(|action| action.status == "applied")
        .count();
    result.planned_count = result
        .actions
        .iter()
        .filter(|action| action.status == "planned")
        .count();
    result.ready_to_apply = result.planned_count > 0 && result.issues.is_empty();
    result.accepted = result
        .actions
        .iter()
        .all(|action| action.status != "failed")
        && result.issues.is_empty();
    result
}

fn plan_host_actions(
    snapshot: &TemplateSnapshot,
    host_snapshot: &HostTemplateSnapshot,
    local_host: &HostSummary,
    actions: &mut Vec<ImportAction>,
    operations: &mut Vec<PlannedImportAction>,
    _issues: &mut Vec<String>,
) {
    let current_runtimes = provider::detect_runtimes_for_host(&local_host.id);
    let current_dependencies = deps::detect_system_dependencies_for_host(&local_host.id);

    if let Some(preset) = &snapshot.applied_mirror_preset {
        let action_id = push_action(
            actions,
            local_host,
            "apply-mirror-preset",
            None,
            "planned",
            format!("Apply mirror preset {} on {}", preset, local_host.label),
            None,
            None,
            false,
        );
        operations.push(PlannedImportAction {
            action_id,
            operation: ImportOperation::ApplyMirrorPreset {
                host_id: local_host.id.clone(),
                preset: preset.clone(),
            },
        });
    }

    let missing_dependencies =
        imported_missing_dependencies(&host_snapshot.system_dependencies, &current_dependencies);
    if !missing_dependencies.is_empty() {
        let action_id = push_action(
            actions,
            local_host,
            "install-dependencies",
            None,
            "planned",
            format!(
                "Install missing base dependencies on {}: {}",
                local_host.label,
                missing_dependencies.join(", ")
            ),
            None,
            None,
            false,
        );
        operations.push(PlannedImportAction {
            action_id,
            operation: ImportOperation::InstallDependencies {
                host_id: local_host.id.clone(),
                names: missing_dependencies,
            },
        });
    }

    for imported_runtime in &host_snapshot.runtimes {
        let Some(current_runtime) = current_runtimes
            .iter()
            .find(|runtime| runtime.family == imported_runtime.family)
        else {
            push_action(
                actions,
                local_host,
                "unsupported-runtime-family",
                Some(imported_runtime.family.clone()),
                "blocked",
                format!(
                    "Block {} on {} because this runtime family is not supported here",
                    imported_runtime.family, local_host.label
                ),
                Some("Unsupported runtime family".into()),
                Some(
                    "This Forge Env build does not recognize that runtime family yet. Remove it from the bundle or extend provider coverage first."
                        .into(),
                ),
                false,
            );
            continue;
        };

        let can_install = current_runtime.capabilities.can_install;
        let can_activate = current_runtime.capabilities.can_activate;
        let current_versions = current_runtime
            .installed
            .iter()
            .map(|installation| installation.version.clone())
            .collect::<HashSet<_>>();
        let current_active = current_runtime
            .installed
            .iter()
            .find(|installation| installation.active)
            .map(|installation| installation.version.clone());

        let mut scheduled_installs = HashSet::new();

        for imported_installation in &imported_runtime.installed {
            if imported_installation.source == "system" || imported_installation.channel == "system"
            {
                continue;
            }

            let version = imported_installation.version.clone();
            if !current_versions.contains(&version) {
                if can_install {
                    let action_id = push_action(
                        actions,
                        local_host,
                        "install-runtime",
                        Some(imported_runtime.family.clone()),
                        "planned",
                        format!(
                            "Install {} {} on {}",
                            imported_runtime.family, version, local_host.label
                        ),
                        None,
                        None,
                        false,
                    );
                    scheduled_installs.insert(version.clone());
                    operations.push(PlannedImportAction {
                        action_id,
                        operation: ImportOperation::InstallRuntime {
                            host_id: local_host.id.clone(),
                            family: imported_runtime.family.clone(),
                            version: version.clone(),
                        },
                    });
                } else {
                    push_action(
                        actions,
                        local_host,
                        "install-runtime",
                        Some(imported_runtime.family.clone()),
                        "blocked",
                        format!(
                            "Block {} {} on {} because {} is not ready",
                            imported_runtime.family,
                            version,
                            local_host.label,
                            current_runtime.provider
                        ),
                        Some(format!(
                            "{} cannot install {} on this host yet.",
                            current_runtime.provider, imported_runtime.family
                        )),
                        Some(runtime_provider_remediation(current_runtime)),
                        provider_repair_available(current_runtime),
                    );
                }
            }

            if imported_installation.active && current_active.as_deref() != Some(version.as_str()) {
                if can_activate {
                    let action_id = push_action(
                        actions,
                        local_host,
                        "activate-runtime",
                        Some(imported_runtime.family.clone()),
                        "planned",
                        format!(
                            "Activate {} {} on {}",
                            imported_runtime.family, version, local_host.label
                        ),
                        if scheduled_installs.contains(&version) {
                            Some("Activation will run after install if needed.".into())
                        } else {
                            None
                        },
                        None,
                        false,
                    );
                    operations.push(PlannedImportAction {
                        action_id,
                        operation: ImportOperation::ActivateRuntime {
                            host_id: local_host.id.clone(),
                            family: imported_runtime.family.clone(),
                            version,
                        },
                    });
                } else {
                    push_action(
                        actions,
                        local_host,
                        "activate-runtime",
                        Some(imported_runtime.family.clone()),
                        "blocked",
                        format!(
                            "Block activation for {} on {} because {} is not ready",
                            imported_runtime.family, local_host.label, current_runtime.provider
                        ),
                        Some(format!(
                            "{} cannot activate {} on this host yet.",
                            current_runtime.provider, imported_runtime.family
                        )),
                        Some(runtime_provider_remediation(current_runtime)),
                        provider_repair_available(current_runtime),
                    );
                }
            }
        }
    }
}

fn validate_snapshot(snapshot: &TemplateSnapshot) -> Vec<String> {
    let mut issues = Vec::new();
    if snapshot.schema_version != 1 && snapshot.schema_version != 2 {
        issues.push(format!(
            "Unsupported schemaVersion {}. Expected 1 or 2.",
            snapshot.schema_version
        ));
    }
    if snapshot.hosts.is_empty() {
        issues.push("Bundle does not contain any hosts.".into());
    }
    if snapshot.schema_version == 2 && snapshot.host_snapshots.is_empty() {
        issues.push("Schema version 2 bundles must include host snapshots.".into());
    }

    let runtime_count = if snapshot.host_snapshots.is_empty() {
        snapshot.runtimes.len()
    } else {
        snapshot
            .host_snapshots
            .iter()
            .map(|snapshot| snapshot.runtimes.len())
            .sum()
    };

    if runtime_count == 0 {
        issues.push("Bundle does not contain any runtimes.".into());
    }

    for host_snapshot in &snapshot.host_snapshots {
        if !snapshot
            .hosts
            .iter()
            .any(|host| host.id == host_snapshot.host.id)
        {
            issues.push(format!(
                "Host snapshot {} is missing a matching host summary entry.",
                host_snapshot.host.id
            ));
        }
    }

    issues
}

fn expand_host_snapshots(snapshot: &TemplateSnapshot) -> Vec<HostTemplateSnapshot> {
    if !snapshot.host_snapshots.is_empty() {
        return snapshot.host_snapshots.clone();
    }

    snapshot
        .hosts
        .first()
        .cloned()
        .map(|host| HostTemplateSnapshot {
            host,
            runtimes: snapshot.runtimes.clone(),
            system_dependencies: snapshot.system_dependencies.clone(),
        })
        .into_iter()
        .collect()
}

fn imported_missing_dependencies(
    imported: &[SystemDependencyState],
    current: &[SystemDependencyState],
) -> Vec<String> {
    imported
        .iter()
        .filter(|dependency| dependency.installed)
        .filter(|dependency| {
            !current
                .iter()
                .any(|current_dep| current_dep.name == dependency.name && current_dep.installed)
        })
        .map(|dependency| dependency.name.clone())
        .collect()
}

fn push_action(
    actions: &mut Vec<ImportAction>,
    host: &HostSummary,
    kind: &str,
    family: Option<String>,
    status: &str,
    label: String,
    reason: Option<String>,
    remediation: Option<String>,
    repair_available: bool,
) -> String {
    let action_id = format!("import-action-{}", actions.len() + 1);
    actions.push(ImportAction {
        id: action_id.clone(),
        host_id: host.id.clone(),
        host_label: host.label.clone(),
        kind: kind.into(),
        family,
        status: status.into(),
        selected: status == "planned",
        label,
        reason,
        outcome_title: None,
        next_step: None,
        remediation,
        repair_available,
    });
    action_id
}

fn runtime_provider_remediation(runtime: &super::models::RuntimeFamilyState) -> String {
    match runtime.family.as_str() {
        "Python" => {
            "Install and initialize pyenv on this host, then refresh runtime detection before replaying the bundle.".into()
        }
        "Node.js" => {
            "Install Volta on this host so Forge Env can manage Node.js versions instead of the system fallback.".into()
        }
        "Rust" => {
            "Install rustup on this host and ensure Cargo shims are available in PATH before retrying.".into()
        }
        "Java" => {
            "Install SDKMAN! on this host and make sure its init script is available before importing Java toolchains.".into()
        }
        "Go" => {
            "Install gvm on this host or use a host where the Go provider is ready before replaying the bundle.".into()
        }
        "Ruby" => {
            "Install and initialize rbenv on this host. Add ruby-build as well if you want Forge Env to install Ruby versions, then refresh detection before replaying the bundle.".into()
        }
        ".NET" => {
            "Install a matching .NET SDK on this host through Forge Env, then use global.json project pinning to keep SDK resolution stable before replaying the bundle."
                .into()
        }
        "PHP" => {
            "Install phpbrew on this host if you want managed PHP versions, then refresh detection before replaying the bundle.".into()
        }
        "C/C++" => {
            "Install the required compiler template through Forge Env, then write a CMakePresets.json policy if the project needs stable compiler hints before replaying the bundle."
                .into()
        }
        _ => format!(
            "Prepare the canonical provider {} on this host and refresh detection before retrying.",
            runtime.provider
        ),
    }
}

fn provider_repair_available(runtime: &super::models::RuntimeFamilyState) -> bool {
    matches!(
        runtime.family.as_str(),
        "Python" | "Node.js" | "Rust" | "Java" | "Go"
    )
}

#[cfg(test)]
mod tests {
    use super::expand_host_snapshots;
    use crate::core::models::{
        HostSummary, RuntimeCapabilities, RuntimeFamilyState, SystemDependencyState,
        TemplateSnapshot,
    };

    #[test]
    fn expands_legacy_snapshot_to_single_host_snapshot() {
        let host = HostSummary {
            id: "macos-native".into(),
            label: "macOS Native Host".into(),
            kind: "macos".into(),
            architecture: "arm64".into(),
            shell: "zsh".into(),
            status: "ready".into(),
            recommended_package_manager: "Homebrew".into(),
            path_preview: vec![],
        };
        let snapshot = TemplateSnapshot {
            schema_version: 1,
            generated_at: "0".into(),
            hosts: vec![host.clone()],
            host_snapshots: vec![],
            runtimes: vec![RuntimeFamilyState {
                family: "Python".into(),
                provider: "pyenv".into(),
                provider_status: "ready".into(),
                detected_binary: None,
                health: "good".into(),
                package_tools: vec![],
                mirrors: vec![],
                recommended_versions: vec![],
                capabilities: RuntimeCapabilities {
                    can_install: true,
                    can_activate: true,
                    can_remove: true,
                },
                notes: vec![],
                installed: vec![],
            }],
            system_dependencies: vec![SystemDependencyState {
                name: "Git".into(),
                command: "git".into(),
                installed: true,
                version: Some("git version 2".into()),
                source_hint: "hint".into(),
            }],
            applied_mirror_preset: None,
        };

        let expanded = expand_host_snapshots(&snapshot);
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0].host.id, host.id);
        assert_eq!(expanded[0].runtimes.len(), 1);
        assert_eq!(expanded[0].system_dependencies.len(), 1);
    }
}
