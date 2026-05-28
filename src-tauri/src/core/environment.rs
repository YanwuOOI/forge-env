use std::{collections::HashSet, fs, path::PathBuf};

use super::{
    hosts,
    models::{EnvPlan, EnvProfileTarget, EnvVariableSuggestion, RuntimeFamilyState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentErrorKind {
    PermissionDenied,
    PathUnavailable,
    HostUnavailable,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct EnvironmentError {
    pub kind: EnvironmentErrorKind,
    pub message: String,
}

impl EnvironmentError {
    fn new(kind: EnvironmentErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    fn classify(message: impl Into<String>) -> Self {
        let message = message.into();
        let lowered = message.to_lowercase();
        let kind = if lowered.contains("permission denied")
            || lowered.contains("operation not permitted")
            || lowered.contains("access is denied")
        {
            EnvironmentErrorKind::PermissionDenied
        } else if lowered.contains("writable")
            || lowered.contains("read-only")
            || lowered.contains("could not determine")
            || lowered.contains("no such file")
        {
            EnvironmentErrorKind::PathUnavailable
        } else if lowered.contains("unknown host")
            || lowered.contains("cannot resolve home directory")
        {
            EnvironmentErrorKind::HostUnavailable
        } else {
            EnvironmentErrorKind::Unknown
        };
        Self { kind, message }
    }

    pub fn outcome_title(&self) -> &'static str {
        match self.kind {
            EnvironmentErrorKind::PermissionDenied => "Permission denied",
            EnvironmentErrorKind::PathUnavailable => "Profile path unavailable",
            EnvironmentErrorKind::HostUnavailable => "Host unavailable",
            EnvironmentErrorKind::Unknown => "Environment apply failed",
        }
    }

    pub fn next_step(&self, target_name: &str) -> String {
        match self.kind {
            EnvironmentErrorKind::PermissionDenied => format!(
                "Grant write access to {} or rerun the environment update through a user that can modify the target shell profile.",
                target_name
            ),
            EnvironmentErrorKind::PathUnavailable => format!(
                "Confirm that {} exists on the selected host and that its parent directory is writable.",
                target_name
            ),
            EnvironmentErrorKind::HostUnavailable => {
                "Re-select a reachable host or restore the missing native/WSL execution path before retrying."
                    .into()
            }
            EnvironmentErrorKind::Unknown => {
                "Inspect the selected shell profile path and host shell configuration, then retry once the root cause is understood."
                    .into()
            }
        }
    }
}

const ENV_BEGIN: &str = "# >>> forge-env env >>>";
const ENV_END: &str = "# <<< forge-env env <<<";

pub fn build_env_plan(
    host_id: &str,
    runtimes: &[RuntimeFamilyState],
    shell_profiles: &[String],
) -> Result<EnvPlan, EnvironmentError> {
    let home = hosts::home_dir_for_host(host_id).ok_or_else(|| {
        EnvironmentError::new(
            EnvironmentErrorKind::HostUnavailable,
            format!("Cannot resolve home directory for host {host_id}."),
        )
    })?;
    let current_path = hosts::path_entries_for_host(host_id);

    let mut variables = Vec::new();
    let mut path_entries = Vec::new();
    let mut seen_paths = HashSet::new();
    let mut notes = vec![
        "Forge Env writes a managed shell block and leaves the rest of your profile untouched."
            .to_string(),
        "PATH entries are prepended so shimmed toolchains win over system fallbacks.".to_string(),
    ];

    for runtime in runtimes {
        match runtime.family.as_str() {
            "Python" if runtime.provider == "pyenv" && runtime.provider_status == "ready" => {
                push_variable(
                    &mut variables,
                    "PYENV_ROOT",
                    "$HOME/.pyenv",
                    "pyenv shims and version metadata live under this root.",
                );
                push_path(
                    &mut path_entries,
                    &mut seen_paths,
                    "$PYENV_ROOT/bin",
                    &current_path,
                );
            }
            "Node.js" if runtime.provider == "Volta" && runtime.provider_status == "ready" => {
                push_variable(
                    &mut variables,
                    "VOLTA_HOME",
                    "$HOME/.volta",
                    "Volta installs its toolchain shims and cache under this directory.",
                );
                push_path(
                    &mut path_entries,
                    &mut seen_paths,
                    "$VOLTA_HOME/bin",
                    &current_path,
                );
            }
            "Rust" if runtime.provider == "rustup" && runtime.provider_status == "ready" => {
                push_variable(
                    &mut variables,
                    "CARGO_HOME",
                    "$HOME/.cargo",
                    "Cargo binaries and install metadata live under this directory.",
                );
                push_variable(
                    &mut variables,
                    "RUSTUP_HOME",
                    "$HOME/.rustup",
                    "rustup stores managed toolchains and metadata here.",
                );
                push_path(
                    &mut path_entries,
                    &mut seen_paths,
                    "$CARGO_HOME/bin",
                    &current_path,
                );
            }
            ".NET" if runtime.capabilities.can_install => {
                push_variable(
                    &mut variables,
                    "DOTNET_ROOT",
                    "$HOME/.dotnet",
                    "The official dotnet-install script places user-managed SDKs under this directory.",
                );
                push_path(
                    &mut path_entries,
                    &mut seen_paths,
                    "$DOTNET_ROOT",
                    &current_path,
                );
                push_path(
                    &mut path_entries,
                    &mut seen_paths,
                    "$DOTNET_ROOT/tools",
                    &current_path,
                );
            }
            _ => {}
        }
    }

    if path_entries.is_empty() {
        notes.push(
            "No managed runtime roots are ready yet, so the PATH plan is informational only."
                .to_string(),
        );
    }

    let mut available_profiles = shell_profiles
        .iter()
        .map(|profile| EnvProfileTarget {
            path: profile.clone(),
            exists: expand_home(profile, &home).is_file(),
            managed_by_forge_env: existing_managed_block(profile, &home),
        })
        .collect::<Vec<_>>();

    if available_profiles.is_empty() {
        available_profiles.push(EnvProfileTarget {
            path: format!("{home}/.profile"),
            exists: expand_home(&format!("{home}/.profile"), &home).is_file(),
            managed_by_forge_env: existing_managed_block(&format!("{home}/.profile"), &home),
        });
    }

    let target_profile = available_profiles
        .first()
        .map(|profile| profile.path.clone())
        .unwrap_or_else(|| format!("{home}/.profile"));

    let managed_block = render_managed_block(&variables, &path_entries);

    Ok(EnvPlan {
        target_profile,
        available_profiles,
        path_entries,
        variables,
        managed_block,
        notes,
    })
}

pub fn apply_env_plan_for_host(
    host_id: &str,
    plan: &EnvPlan,
    profile_path: &str,
) -> Result<String, EnvironmentError> {
    let home = hosts::home_dir_for_host(host_id).ok_or_else(|| {
        EnvironmentError::new(
            EnvironmentErrorKind::HostUnavailable,
            format!("Cannot resolve home directory for host {host_id}."),
        )
    })?;
    let expanded = expand_home(profile_path, &home);

    if let Some(parent) = expanded.parent() {
        hosts::create_dir_all_on_host(host_id, parent).map_err(EnvironmentError::classify)?;
    }

    let existing =
        hosts::read_file_on_host(host_id, &expanded).map_err(EnvironmentError::classify)?;
    let updated = replace_managed_block(&existing, &plan.managed_block);
    hosts::write_file_on_host(host_id, &expanded, &updated).map_err(EnvironmentError::classify)?;

    Ok(format!(
        "Applied Forge Env shell block to {} on {}.",
        expanded.to_string_lossy(),
        host_id
    ))
}

fn push_variable(variables: &mut Vec<EnvVariableSuggestion>, key: &str, value: &str, reason: &str) {
    if variables.iter().any(|item| item.key == key) {
        return;
    }

    variables.push(EnvVariableSuggestion {
        key: key.to_string(),
        value: value.to_string(),
        reason: reason.to_string(),
    });
}

fn push_path(
    path_entries: &mut Vec<String>,
    seen_paths: &mut HashSet<String>,
    candidate: &str,
    current_path: &HashSet<String>,
) {
    let normalized = candidate.to_string();
    if current_path.contains(&normalized) || !seen_paths.insert(normalized.clone()) {
        return;
    }

    path_entries.push(normalized);
}

fn render_managed_block(variables: &[EnvVariableSuggestion], path_entries: &[String]) -> String {
    let mut lines = vec![
        ENV_BEGIN.to_string(),
        "# Generated by Forge Env. Edit via the app so changes stay in sync.".to_string(),
    ];

    for variable in variables {
        lines.push(format!("export {}=\"{}\"", variable.key, variable.value));
    }

    if !path_entries.is_empty() {
        lines.push(format!("export PATH=\"{}:$PATH\"", path_entries.join(":")));
    }

    lines.push(ENV_END.to_string());
    lines.join("\n")
}

fn replace_managed_block(existing: &str, managed_block: &str) -> String {
    if let (Some(begin), Some(end)) = (existing.find(ENV_BEGIN), existing.find(ENV_END)) {
        let after_end = end + ENV_END.len();
        let mut next = String::new();
        next.push_str(existing[..begin].trim_end());
        if !next.is_empty() {
            next.push_str("\n\n");
        }
        next.push_str(managed_block.trim_end());
        let tail = existing[after_end..].trim();
        if !tail.is_empty() {
            next.push_str("\n\n");
            next.push_str(tail);
        }
        next.push('\n');
        return next;
    }

    if existing.trim().is_empty() {
        return format!("{}\n", managed_block.trim_end());
    }

    format!("{}\n\n{}\n", existing.trim_end(), managed_block.trim_end())
}

fn existing_managed_block(profile: &str, home: &str) -> bool {
    let path = expand_home(profile, home);
    fs::read_to_string(path)
        .map(|content| content.contains(ENV_BEGIN) && content.contains(ENV_END))
        .unwrap_or(false)
}

fn expand_home(path: &str, home: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        return PathBuf::from(home).join(rest);
    }
    if path == "~" {
        return PathBuf::from(home);
    }

    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::{build_env_plan, render_managed_block, replace_managed_block, ENV_BEGIN};
    use crate::core::models::{EnvVariableSuggestion, RuntimeCapabilities, RuntimeFamilyState};

    #[test]
    fn renders_managed_block_with_path() {
        let block = render_managed_block(
            &[EnvVariableSuggestion {
                key: "VOLTA_HOME".into(),
                value: "$HOME/.volta".into(),
                reason: "Volta root".into(),
            }],
            &["$VOLTA_HOME/bin".into()],
        );
        assert!(block.contains("export VOLTA_HOME=\"$HOME/.volta\""));
        assert!(block.contains("export PATH=\"$VOLTA_HOME/bin:$PATH\""));
    }

    #[test]
    fn replaces_existing_env_block() {
        let before = format!("export FOO=bar\n\n{ENV_BEGIN}\nold block\n# <<< forge-env env <<<\n");
        let after = replace_managed_block(
            &before,
            "# >>> forge-env env >>>\nnew block\n# <<< forge-env env <<<\n",
        );
        assert!(after.contains("new block"));
        assert!(!after.contains("old block"));
        assert!(after.contains("export FOO=bar"));
    }

    #[test]
    fn dotnet_installable_runtime_adds_dotnet_paths() {
        let plan = build_env_plan(
            "native",
            &[RuntimeFamilyState {
                family: ".NET".into(),
                provider: "dotnet SDK".into(),
                provider_status: "fallback-system".into(),
                detected_binary: Some("/usr/local/share/dotnet/dotnet".into()),
                health: "attention".into(),
                package_tools: vec!["NuGet".into()],
                mirrors: vec!["nuget.org".into()],
                recommended_versions: vec!["8.0 LTS".into()],
                capabilities: RuntimeCapabilities {
                    can_install: true,
                    can_activate: false,
                    can_remove: false,
                },
                notes: vec![],
                installed: vec![],
            }],
            &[],
        )
        .unwrap();

        assert!(plan
            .managed_block
            .contains("export DOTNET_ROOT=\"$HOME/.dotnet\""));
        assert!(plan
            .managed_block
            .contains("export PATH=\"$DOTNET_ROOT:$DOTNET_ROOT/tools:$PATH\""));
    }
}
