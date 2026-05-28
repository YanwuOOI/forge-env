use std::path::PathBuf;

use super::{hosts, shell};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorErrorKind {
    PermissionDenied,
    PathUnavailable,
    ToolUnavailable,
    HostUnavailable,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct MirrorError {
    pub kind: MirrorErrorKind,
    pub message: String,
}

impl MirrorError {
    fn new(kind: MirrorErrorKind, message: impl Into<String>) -> Self {
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
            MirrorErrorKind::PermissionDenied
        } else if lowered.contains("writable")
            || lowered.contains("read-only")
            || lowered.contains("could not determine")
            || lowered.contains("no such file")
        {
            MirrorErrorKind::PathUnavailable
        } else if lowered.contains("command not found")
            || lowered.contains("not found")
            || lowered.contains("unsupported")
        {
            MirrorErrorKind::ToolUnavailable
        } else if lowered.contains("unknown host")
            || lowered.contains("cannot resolve home directory")
        {
            MirrorErrorKind::HostUnavailable
        } else {
            MirrorErrorKind::Unknown
        };
        Self { kind, message }
    }

    pub fn outcome_title(&self) -> &'static str {
        match self.kind {
            MirrorErrorKind::PermissionDenied => "Permission denied",
            MirrorErrorKind::PathUnavailable => "Registry config unavailable",
            MirrorErrorKind::ToolUnavailable => "Mirror tooling unavailable",
            MirrorErrorKind::HostUnavailable => "Host unavailable",
            MirrorErrorKind::Unknown => "Mirror apply failed",
        }
    }

    pub fn next_step(&self, target_name: &str) -> String {
        match self.kind {
            MirrorErrorKind::PermissionDenied => format!(
                "Grant the required write access for {} registry config files or rerun the update through a user that can modify them.",
                target_name
            ),
            MirrorErrorKind::PathUnavailable => format!(
                "Confirm that {} registry config locations exist and are writable on the selected host.",
                target_name
            ),
            MirrorErrorKind::ToolUnavailable => format!(
                "Install or expose the required registry tooling for {} before retrying.",
                target_name
            ),
            MirrorErrorKind::HostUnavailable => {
                "Re-select a reachable host or restore the missing native/WSL execution path before retrying."
                    .into()
            }
            MirrorErrorKind::Unknown => {
                "Inspect host package manager and registry configuration state, then retry once the failure cause is understood."
                    .into()
            }
        }
    }
}

const CARGO_BEGIN: &str = "# >>> forge-env mirror >>>";
const CARGO_END: &str = "# <<< forge-env mirror <<<";

pub struct MirrorPreset {
    pub npm_registry: &'static str,
    pub pip_index_url: &'static str,
    pub cargo_sparse_registry: &'static str,
    pub notes: &'static [&'static str],
}

pub fn apply_preset_for_host(host_id: &str, preset_name: &str) -> Result<Vec<String>, MirrorError> {
    let preset = preset_by_name(preset_name).ok_or_else(|| {
        MirrorError::new(
            MirrorErrorKind::Unknown,
            format!("Unknown mirror preset: {preset_name}"),
        )
    })?;
    let mut changes = Vec::new();

    if host_command_exists(host_id, "npm") {
        hosts::run_checked_on_host(
            host_id,
            "npm",
            &["config", "set", "registry", preset.npm_registry],
        )
        .map_err(MirrorError::classify)?;
        changes.push(format!("Updated npm registry to {}", preset.npm_registry));
    }

    if host_command_exists(host_id, "python3") {
        hosts::run_checked_on_host(
            host_id,
            "python3",
            &[
                "-m",
                "pip",
                "config",
                "set",
                "global.index-url",
                preset.pip_index_url,
            ],
        )
        .map_err(MirrorError::classify)?;
        changes.push(format!("Updated pip index-url to {}", preset.pip_index_url));
    }

    if host_command_exists(host_id, "cargo") || host_command_exists(host_id, "rustup") {
        write_cargo_config_for_host(host_id, &preset)?;
        changes.push(format!(
            "Updated Cargo mirror block on {} to {}",
            host_id, preset.cargo_sparse_registry
        ));
    }

    changes.extend(preset.notes.iter().map(|note| note.to_string()));
    Ok(changes)
}

fn preset_by_name(name: &str) -> Option<MirrorPreset> {
    match name {
        "Tsinghua" => Some(MirrorPreset {
            npm_registry: "https://registry.npmmirror.com/",
            pip_index_url: "https://pypi.tuna.tsinghua.edu.cn/simple",
            cargo_sparse_registry: "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/",
            notes: &[
                "npm uses npmmirror while Python and Cargo point at Tsinghua-maintained mirrors.",
            ],
        }),
        "Aliyun" => Some(MirrorPreset {
            npm_registry: "https://registry.npmmirror.com/",
            pip_index_url: "https://mirrors.aliyun.com/pypi/simple/",
            cargo_sparse_registry: "sparse+https://rsproxy.cn/index/",
            notes: &["Cargo falls back to rsproxy because Aliyun does not provide a first-party crates sparse index."],
        }),
        "Huawei Cloud" => Some(MirrorPreset {
            npm_registry: "https://repo.huaweicloud.com/repository/npm/",
            pip_index_url: "https://repo.huaweicloud.com/repository/pypi/simple",
            cargo_sparse_registry: "sparse+https://rsproxy.cn/index/",
            notes: &["Cargo falls back to rsproxy because Huawei Cloud does not provide a first-party crates sparse index."],
        }),
        "Company Proxy" => Some(MirrorPreset {
            npm_registry: "https://registry.npmmirror.com/",
            pip_index_url: "https://pypi.org/simple",
            cargo_sparse_registry: "sparse+https://index.crates.io/",
            notes: &["Company Proxy is currently a placeholder preset; replace these URLs with your internal registry endpoints."],
        }),
        _ => None,
    }
}

fn write_cargo_config_for_host(host_id: &str, preset: &MirrorPreset) -> Result<(), MirrorError> {
    let home = hosts::home_dir_for_host(host_id).ok_or_else(|| {
        MirrorError::new(
            MirrorErrorKind::HostUnavailable,
            format!("Cannot resolve home directory for host {host_id}."),
        )
    })?;
    let cargo_dir = PathBuf::from(home).join(".cargo");
    hosts::create_dir_all_on_host(host_id, &cargo_dir).map_err(MirrorError::classify)?;

    let config_path = cargo_dir.join("config.toml");
    let existing =
        hosts::read_file_on_host(host_id, &config_path).map_err(MirrorError::classify)?;
    let managed_block = format!(
        "{CARGO_BEGIN}\n[source.crates-io]\nreplace-with = \"forge-env-mirror\"\n\n[source.forge-env-mirror]\nregistry = \"{}\"\n{CARGO_END}\n",
        preset.cargo_sparse_registry
    );

    let updated = replace_managed_block(&existing, &managed_block);
    hosts::write_file_on_host(host_id, &config_path, &updated).map_err(MirrorError::classify)
}

fn host_command_exists(host_id: &str, command: &str) -> bool {
    if host_id.starts_with("wsl:") {
        hosts::run_shell_script_on_host(
            host_id,
            &format!("command -v {command} >/dev/null 2>&1 && echo found"),
        )
        .ok()
        .is_some_and(|result| !result.stdout.is_empty() || !result.stderr.is_empty())
    } else {
        shell::command_exists(command)
    }
}

fn replace_managed_block(existing: &str, managed_block: &str) -> String {
    if let (Some(begin), Some(end)) = (existing.find(CARGO_BEGIN), existing.find(CARGO_END)) {
        let after_end = end + CARGO_END.len();
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

#[cfg(test)]
mod tests {
    use super::{replace_managed_block, CARGO_BEGIN};

    #[test]
    fn replaces_existing_managed_block() {
        let before = format!(
            "[build]\ntarget-dir = \"target\"\n\n{CARGO_BEGIN}\nold block\n# <<< forge-env mirror <<<\n"
        );
        let after = replace_managed_block(
            &before,
            "# >>> forge-env mirror >>>\nnew block\n# <<< forge-env mirror <<<\n",
        );
        assert!(after.contains("new block"));
        assert!(!after.contains("old block"));
        assert!(after.contains("target-dir"));
    }
}
