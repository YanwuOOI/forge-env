use std::path::Path;

use super::{deps, hosts};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    PermissionDenied,
    ToolUnavailable,
    HostUnavailable,
    VersionUnsupported,
    ActiveConflict,
    UnsupportedOperation,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub message: String,
}

impl RuntimeError {
    fn new(kind: RuntimeErrorKind, message: impl Into<String>) -> Self {
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
            RuntimeErrorKind::PermissionDenied
        } else if lowered.contains("unknown host")
            || lowered.contains("wsl command is not available")
            || lowered.contains("cannot resolve home directory")
        {
            RuntimeErrorKind::HostUnavailable
        } else if lowered.contains("command not found")
            || lowered.contains("is required")
            || lowered.contains("install curl first")
        {
            RuntimeErrorKind::ToolUnavailable
        } else if lowered.contains("could not resolve a concrete sdkman! java version")
            || lowered.contains("try an exact candidate")
        {
            RuntimeErrorKind::VersionUnsupported
        } else if lowered.contains("switch to another toolchain first")
            || lowered.contains("activate ")
            || lowered.contains("active default")
        {
            RuntimeErrorKind::ActiveConflict
        } else if lowered.contains("unsupported") || lowered.contains("not implemented") {
            RuntimeErrorKind::UnsupportedOperation
        } else {
            RuntimeErrorKind::Unknown
        };
        Self { kind, message }
    }

    pub fn outcome_title(&self) -> &'static str {
        match self.kind {
            RuntimeErrorKind::PermissionDenied => "Permission denied",
            RuntimeErrorKind::ToolUnavailable => "Runtime tool unavailable",
            RuntimeErrorKind::HostUnavailable => "Host unavailable",
            RuntimeErrorKind::VersionUnsupported => "Version unsupported",
            RuntimeErrorKind::ActiveConflict => "Active version conflict",
            RuntimeErrorKind::UnsupportedOperation => "Unsupported operation",
            RuntimeErrorKind::Unknown => "Runtime action failed",
        }
    }

    pub fn next_step(&self, target_name: &str) -> String {
        match self.kind {
            RuntimeErrorKind::PermissionDenied => format!(
                "Grant the required host permission or rerun the {} action through a user that can modify the managed toolchain.",
                target_name
            ),
            RuntimeErrorKind::ToolUnavailable => format!(
                "Install or bootstrap the canonical provider and required shell tools for {} before retrying.",
                target_name
            ),
            RuntimeErrorKind::HostUnavailable => {
                "Re-select a reachable host or restore the missing native/WSL execution path before retrying."
                    .into()
            }
            RuntimeErrorKind::VersionUnsupported => format!(
                "Choose a concrete, provider-supported {} version label for the selected host, then retry.",
                target_name
            ),
            RuntimeErrorKind::ActiveConflict => format!(
                "Switch {} to a different active version first, then retry the requested mutation.",
                target_name
            ),
            RuntimeErrorKind::UnsupportedOperation => format!(
                "{} is not supported through its canonical provider on the selected host yet.",
                target_name
            ),
            RuntimeErrorKind::Unknown => format!(
                "Inspect the selected host and canonical provider state for {}, then retry once the root cause is understood.",
                target_name
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RuntimeMutation {
    Install,
    Activate,
    Remove,
}

pub fn execute_provider_bootstrap(host_id: &str, family: &str) -> Result<String, RuntimeError> {
    match family {
        "Python" => bootstrap_python_provider(host_id),
        "Node.js" => bootstrap_node_provider(host_id),
        "Rust" => bootstrap_rust_provider(host_id),
        "Java" => bootstrap_java_provider(host_id),
        "Go" => bootstrap_go_provider(host_id),
        other => Err(RuntimeError::new(
            RuntimeErrorKind::UnsupportedOperation,
            format!("Unsupported provider bootstrap family: {other}"),
        )),
    }
}

pub fn execute_runtime_mutation(
    host_id: &str,
    family: &str,
    version: &str,
    mutation: RuntimeMutation,
) -> Result<String, RuntimeError> {
    match family {
        "Python" => execute_python(host_id, version, mutation),
        "Node.js" => execute_node(host_id, version, mutation),
        "Rust" => execute_rust(host_id, version, mutation),
        "Java" => execute_java(host_id, version, mutation),
        "Go" => execute_go(host_id, version, mutation),
        ".NET" => execute_dotnet(host_id, version, mutation),
        "C/C++" => execute_cpp(host_id, version, mutation),
        "PHP" => execute_php(host_id, version, mutation),
        "Ruby" => execute_ruby(host_id, version, mutation),
        other => Err(RuntimeError::new(
            RuntimeErrorKind::UnsupportedOperation,
            format!("Unsupported runtime family: {other}"),
        )),
    }
}

fn execute_python(
    host_id: &str,
    version: &str,
    mutation: RuntimeMutation,
) -> Result<String, RuntimeError> {
    require_binary(
        host_id,
        "pyenv",
        "pyenv is required to manage Python runtimes.",
    )?;

    match mutation {
        RuntimeMutation::Install => {
            run(host_id, "pyenv", &["install", version])?;
            Ok(format!(
                "Installed Python {version} via pyenv on {host_id}."
            ))
        }
        RuntimeMutation::Activate => {
            run(host_id, "pyenv", &["global", version])?;
            Ok(format!(
                "Activated Python {version} globally via pyenv on {host_id}."
            ))
        }
        RuntimeMutation::Remove => {
            if active_pyenv_version(host_id).as_deref() == Some(version) {
                run(host_id, "pyenv", &["global", "system"])?;
            }
            run(host_id, "pyenv", &["uninstall", "-f", version])?;
            Ok(format!("Removed Python {version} via pyenv on {host_id}."))
        }
    }
}

fn execute_node(
    host_id: &str,
    version: &str,
    mutation: RuntimeMutation,
) -> Result<String, RuntimeError> {
    require_binary(
        host_id,
        "volta",
        "Volta is required to manage Node.js runtimes.",
    )?;
    let normalized = normalize_node_version(version);

    match mutation {
        RuntimeMutation::Install | RuntimeMutation::Activate => {
            let spec = format!("node@{normalized}");
            run(host_id, "volta", &["install", &spec])?;
            if matches!(mutation, RuntimeMutation::Install) {
                Ok(format!(
                    "Installed Node.js {normalized} via Volta on {host_id}."
                ))
            } else {
                Ok(format!(
                    "Activated Node.js {normalized} via Volta on {host_id}."
                ))
            }
        }
        RuntimeMutation::Remove => {
            let active = hosts::run_capture_on_host(host_id, "node", &["--version"])
                .unwrap_or_default()
                .trim_start_matches('v')
                .to_string();
            if !active.is_empty() && normalize_node_version(&active) != normalized {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::ActiveConflict,
                    format!(
                        "Volta exposes one active default Node.js toolchain. Active version is {active}; activate {normalized} first if you want to replace it."
                    ),
                ));
            }
            run(host_id, "volta", &["uninstall", "node"])?;
            Ok(format!(
                "Removed the active Node.js toolchain from Volta on {host_id} ({normalized})."
            ))
        }
    }
}

fn execute_rust(
    host_id: &str,
    version: &str,
    mutation: RuntimeMutation,
) -> Result<String, RuntimeError> {
    require_binary(
        host_id,
        "rustup",
        "rustup is required to manage Rust toolchains.",
    )?;

    match mutation {
        RuntimeMutation::Install => {
            run(host_id, "rustup", &["toolchain", "install", version])?;
            Ok(format!(
                "Installed Rust toolchain {version} via rustup on {host_id}."
            ))
        }
        RuntimeMutation::Activate => {
            run(host_id, "rustup", &["default", version])?;
            Ok(format!(
                "Activated Rust toolchain {version} via rustup on {host_id}."
            ))
        }
        RuntimeMutation::Remove => {
            let active =
                hosts::run_capture_on_host(host_id, "rustup", &["show", "active-toolchain"])
                    .and_then(|value| value.split_whitespace().next().map(ToOwned::to_owned))
                    .unwrap_or_default();
            if active == version {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::ActiveConflict,
                    format!(
                        "Cannot remove the active default Rust toolchain {version}. Switch to another toolchain first."
                    ),
                ));
            }
            run(host_id, "rustup", &["toolchain", "uninstall", version])?;
            Ok(format!(
                "Removed Rust toolchain {version} via rustup on {host_id}."
            ))
        }
    }
}

fn execute_java(
    host_id: &str,
    version: &str,
    mutation: RuntimeMutation,
) -> Result<String, RuntimeError> {
    let home = hosts::home_dir_for_host(host_id).ok_or_else(|| {
        RuntimeError::new(
            RuntimeErrorKind::HostUnavailable,
            format!("Cannot resolve home directory for host {host_id}."),
        )
    })?;
    let init = format!("{home}/.sdkman/bin/sdkman-init.sh");
    if !hosts::path_exists_on_host(host_id, Path::new(&init)) {
        return Err(RuntimeError::new(
            RuntimeErrorKind::ToolUnavailable,
            "SDKMAN! is required to manage Java runtimes on this host.",
        ));
    }

    let resolved = resolve_sdkman_java_version(host_id, version, &init)?;
    match mutation {
        RuntimeMutation::Install => {
            run_shell(
                host_id,
                &format!(". \"{init}\" && printf 'n\\n' | sdk install java \"{resolved}\""),
            )?;
            Ok(format!(
                "Installed Java {resolved} via SDKMAN! on {host_id}."
            ))
        }
        RuntimeMutation::Activate => {
            run_shell(
                host_id,
                &format!(". \"{init}\" && sdk default java \"{resolved}\""),
            )?;
            Ok(format!(
                "Activated Java {resolved} via SDKMAN! on {host_id}."
            ))
        }
        RuntimeMutation::Remove => {
            run_shell(
                host_id,
                &format!(". \"{init}\" && sdk uninstall java \"{resolved}\""),
            )?;
            Ok(format!("Removed Java {resolved} via SDKMAN! on {host_id}."))
        }
    }
}

fn execute_go(
    host_id: &str,
    version: &str,
    mutation: RuntimeMutation,
) -> Result<String, RuntimeError> {
    let home = hosts::home_dir_for_host(host_id).ok_or_else(|| {
        RuntimeError::new(
            RuntimeErrorKind::HostUnavailable,
            format!("Cannot resolve home directory for host {host_id}."),
        )
    })?;
    let init = format!("{home}/.gvm/scripts/gvm");
    if !hosts::path_exists_on_host(host_id, Path::new(&init)) {
        return Err(RuntimeError::new(
            RuntimeErrorKind::ToolUnavailable,
            "gvm is required to install or switch managed Go runtimes on this host.",
        ));
    }

    let normalized = normalize_go_requested_version(version);
    match mutation {
        RuntimeMutation::Install => {
            run_shell(host_id, &format!(". \"{init}\" && gvm install \"{normalized}\" -B"))?;
            Ok(format!("Installed Go {normalized} via gvm on {host_id}."))
        }
        RuntimeMutation::Activate => {
            run_shell(host_id, &format!(". \"{init}\" && gvm use \"{normalized}\" --default"))?;
            Ok(format!("Activated Go {normalized} via gvm on {host_id}."))
        }
        RuntimeMutation::Remove => Err(RuntimeError::new(
            RuntimeErrorKind::UnsupportedOperation,
            "Per-version Go removal is not implemented yet. The official gvm README documents `install`, `use`, `list`, and full `implode`, but not a supported single-version uninstall flow.",
        )),
    }
}

fn execute_ruby(
    host_id: &str,
    version: &str,
    mutation: RuntimeMutation,
) -> Result<String, RuntimeError> {
    require_binary(
        host_id,
        "rbenv",
        "rbenv is required to manage Ruby runtimes.",
    )?;

    match mutation {
        RuntimeMutation::Install => {
            require_rbenv_install_support(host_id)?;
            run(host_id, "rbenv", &["install", "-s", version])?;
            let _ = run(host_id, "rbenv", &["rehash"]);
            Ok(format!("Installed Ruby {version} via rbenv on {host_id}."))
        }
        RuntimeMutation::Activate => {
            let installed = hosts::run_capture_on_host(host_id, "rbenv", &["versions", "--bare"])
                .unwrap_or_default();
            let found = installed
                .lines()
                .map(str::trim)
                .any(|line| !line.is_empty() && line == version);
            if !found {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::VersionUnsupported,
                    format!(
                        "Ruby {version} is not installed under rbenv on {host_id}. Install it externally first, then retry activation."
                    ),
                ));
            }

            run(host_id, "rbenv", &["global", version])?;
            let _ = run(host_id, "rbenv", &["rehash"]);
            Ok(format!(
                "Activated Ruby {version} globally via rbenv on {host_id}."
            ))
        }
        RuntimeMutation::Remove => {
            require_rbenv_uninstall_support(host_id)?;
            let active = hosts::run_capture_on_host(host_id, "rbenv", &["version-name"])
                .and_then(|value| value.split_whitespace().next().map(ToOwned::to_owned))
                .unwrap_or_default();
            if active == version {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::ActiveConflict,
                    format!(
                        "Cannot remove the active default Ruby version {version}. Switch to another version first."
                    ),
                ));
            }
            run(host_id, "rbenv", &["uninstall", "-f", version])?;
            let _ = run(host_id, "rbenv", &["rehash"]);
            Ok(format!("Removed Ruby {version} via rbenv on {host_id}."))
        }
    }
}

fn execute_php(
    host_id: &str,
    version: &str,
    mutation: RuntimeMutation,
) -> Result<String, RuntimeError> {
    require_binary(
        host_id,
        "phpbrew",
        "phpbrew is required to manage PHP runtimes.",
    )?;

    match mutation {
        RuntimeMutation::Install => {
            let resolved = resolve_phpbrew_version(host_id, version)?;
            run(host_id, "phpbrew", &["install", &resolved, "+default"])?;
            Ok(format!(
                "Installed PHP {resolved} via phpbrew on {host_id}."
            ))
        }
        RuntimeMutation::Activate => {
            let installed =
                hosts::run_capture_on_host(host_id, "phpbrew", &["list"]).unwrap_or_default();
            let found = installed.lines().map(str::trim).any(|line| {
                if line.is_empty() {
                    return false;
                }
                let normalized = line
                    .trim_start_matches('*')
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .trim_start_matches("php-");
                !normalized.is_empty() && normalized == version.trim_start_matches("php-")
            });
            if !found {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::VersionUnsupported,
                    format!(
                        "PHP {version} is not installed under phpbrew on {host_id}. Install it externally first, then retry activation."
                    ),
                ));
            }

            run(host_id, "phpbrew", &["switch", version])?;
            Ok(format!(
                "Activated PHP {version} globally via phpbrew on {host_id}."
            ))
        }
        RuntimeMutation::Remove => {
            require_phpbrew_remove_support(host_id)?;
            let active = active_phpbrew_version(host_id).unwrap_or_default();
            let normalized = normalize_php_requested_version(version);
            if active == normalized {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::ActiveConflict,
                    format!(
                        "Cannot remove the active default PHP version {normalized}. Switch to another version first."
                    ),
                ));
            }
            run(
                host_id,
                "phpbrew",
                &["remove", &format!("php-{normalized}")],
            )?;
            Ok(format!(
                "Removed PHP {normalized} via phpbrew on {host_id}."
            ))
        }
    }
}

fn execute_dotnet(
    host_id: &str,
    version: &str,
    mutation: RuntimeMutation,
) -> Result<String, RuntimeError> {
    match mutation {
        RuntimeMutation::Install => {
            if !supports_dotnet_script_install(host_id) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::UnsupportedOperation,
                    ".NET install is currently wired through the official Unix shell installer. Use a Unix-like or WSL host for managed SDK installation.",
                ));
            }
            require_binary(
                host_id,
                "curl",
                "curl is required to install .NET SDKs through the official dotnet-install script.",
            )?;

            let script_args = dotnet_install_script_args(version);
            run_shell(
                host_id,
                &format!(
                    "curl -fsSL https://dot.net/v1/dotnet-install.sh | bash /dev/stdin {script_args}"
                ),
            )?;
            Ok(format!(
                "Installed .NET SDK {} via the official dotnet-install script on {host_id}.",
                dotnet_install_display(version)
            ))
        }
        RuntimeMutation::Activate => Err(RuntimeError::new(
            RuntimeErrorKind::UnsupportedOperation,
            ".NET activation is project-scoped through global.json today. Forge Env does not rewrite global SDK selection globally yet.",
        )),
        RuntimeMutation::Remove => Err(RuntimeError::new(
            RuntimeErrorKind::UnsupportedOperation,
            ".NET remove is not wired yet. Forge Env can install SDKs, but SDK removal remains follow-up work.",
        )),
    }
}

fn execute_cpp(
    host_id: &str,
    version: &str,
    mutation: RuntimeMutation,
) -> Result<String, RuntimeError> {
    match mutation {
        RuntimeMutation::Install => deps::install_cpp_toolchain_for_host(host_id, version)
            .map_err(|error| RuntimeError::new(map_dependency_error_kind(error.kind), error.message)),
        RuntimeMutation::Activate => Err(RuntimeError::new(
            RuntimeErrorKind::UnsupportedOperation,
            "C/C++ activation is not modeled as a global switch. Install the desired compiler template and choose it per project or build invocation.",
        )),
        RuntimeMutation::Remove => Err(RuntimeError::new(
            RuntimeErrorKind::UnsupportedOperation,
            "C/C++ remove is not wired yet. Use the host package manager directly if a compiler template needs to be uninstalled.",
        )),
    }
}

fn run(host_id: &str, command: &str, args: &[&str]) -> Result<(), RuntimeError> {
    hosts::run_checked_on_host(host_id, command, args)
        .map(|_| ())
        .map_err(RuntimeError::classify)
}

fn run_shell(host_id: &str, script: &str) -> Result<(), RuntimeError> {
    hosts::run_shell_script_on_host(host_id, script)
        .map(|_| ())
        .map_err(RuntimeError::classify)
}

fn bootstrap_python_provider(host_id: &str) -> Result<String, RuntimeError> {
    if hosts::run_capture_on_host(host_id, "brew", &["--version"]).is_some() {
        run(host_id, "brew", &["install", "pyenv"])?;
        return Ok(format!("Installed pyenv via Homebrew on {host_id}."));
    }

    run_provider_install_script(
        host_id,
        "curl -fsSL https://pyenv.run | bash",
        "Installed pyenv via the official installer",
    )
}

fn bootstrap_node_provider(host_id: &str) -> Result<String, RuntimeError> {
    run_provider_install_script(
        host_id,
        "curl -fsSL https://get.volta.sh | bash",
        "Installed Volta via the official installer",
    )
}

fn bootstrap_rust_provider(host_id: &str) -> Result<String, RuntimeError> {
    run_provider_install_script(
        host_id,
        "curl https://sh.rustup.rs -sSf | sh -s -- -y",
        "Installed rustup via the official installer",
    )
}

fn bootstrap_java_provider(host_id: &str) -> Result<String, RuntimeError> {
    run_provider_install_script(
        host_id,
        "curl -fsSL https://get.sdkman.io | bash",
        "Installed SDKMAN! via the official installer",
    )
}

fn bootstrap_go_provider(host_id: &str) -> Result<String, RuntimeError> {
    run_provider_install_script(
        host_id,
        "curl -fsSL https://raw.githubusercontent.com/moovweb/gvm/master/binscripts/gvm-installer | bash",
        "Installed gvm via the official installer",
    )
}

fn run_provider_install_script(
    host_id: &str,
    script: &str,
    message: &str,
) -> Result<String, RuntimeError> {
    if hosts::run_capture_on_host(host_id, "curl", &["--version"]).is_none() {
        return Err(RuntimeError::new(
            RuntimeErrorKind::ToolUnavailable,
            format!(
                "curl is required to bootstrap the canonical provider on {host_id}. Install curl first, then retry."
            ),
        ));
    }

    run_shell(host_id, script)?;
    Ok(format!(
        "{message} on {host_id}. Refresh runtime detection to confirm readiness."
    ))
}

fn require_binary(host_id: &str, command: &str, message: &str) -> Result<(), RuntimeError> {
    if hosts::run_capture_on_host(
        host_id,
        "sh",
        &[
            "-lc",
            &format!("command -v {command} >/dev/null 2>&1 && echo found"),
        ],
    )
    .is_some()
    {
        Ok(())
    } else {
        Err(RuntimeError::new(
            RuntimeErrorKind::ToolUnavailable,
            message.to_string(),
        ))
    }
}

fn require_rbenv_install_support(host_id: &str) -> Result<(), RuntimeError> {
    let commands = hosts::run_capture_on_host(host_id, "rbenv", &["commands"]).unwrap_or_default();
    let install_supported = rbenv_supports_command_output(&commands, "install");

    if install_supported {
        Ok(())
    } else {
        Err(RuntimeError::new(
            RuntimeErrorKind::ToolUnavailable,
            "rbenv install requires the ruby-build plugin on this host. Install ruby-build first, then retry.",
        ))
    }
}

fn require_rbenv_uninstall_support(host_id: &str) -> Result<(), RuntimeError> {
    let commands = hosts::run_capture_on_host(host_id, "rbenv", &["commands"]).unwrap_or_default();
    let uninstall_supported = rbenv_supports_command_output(&commands, "uninstall");

    if uninstall_supported {
        Ok(())
    } else {
        Err(RuntimeError::new(
            RuntimeErrorKind::ToolUnavailable,
            "rbenv uninstall requires the ruby-build plugin on this host. Install ruby-build first, then retry.",
        ))
    }
}

fn rbenv_supports_command_output(output: &str, command: &str) -> bool {
    output.lines().map(str::trim).any(|line| line == command)
}

fn active_pyenv_version(host_id: &str) -> Option<String> {
    hosts::run_capture_on_host(host_id, "pyenv", &["version-name"])
        .and_then(|value| value.split_whitespace().next().map(ToOwned::to_owned))
}

fn normalize_node_version(version: &str) -> String {
    let cleaned = version
        .trim()
        .trim_start_matches("node@")
        .trim_end_matches(" LTS")
        .trim();
    let without_v = cleaned.trim_start_matches('v');
    without_v.to_string()
}

fn normalize_go_requested_version(version: &str) -> String {
    let cleaned = version.trim().trim_start_matches('v').trim();
    if cleaned.starts_with("go") {
        cleaned.to_string()
    } else {
        format!("go{cleaned}")
    }
}

fn normalize_php_requested_version(version: &str) -> String {
    version.trim().trim_start_matches("php-").to_string()
}

fn normalize_dotnet_requested_version(version: &str) -> String {
    version.trim().trim_end_matches(" LTS").to_string()
}

fn supports_dotnet_script_install(host_id: &str) -> bool {
    host_id.starts_with("wsl:") || !cfg!(target_os = "windows")
}

fn dotnet_install_display(version: &str) -> String {
    normalize_dotnet_requested_version(version)
}

fn dotnet_install_script_args(version: &str) -> String {
    let normalized = normalize_dotnet_requested_version(version);
    if normalized.split('.').count() >= 3
        && normalized
            .chars()
            .all(|ch| ch.is_ascii_digit() || ch == '.')
    {
        format!("--version {normalized}")
    } else {
        format!("--channel {normalized}")
    }
}

fn resolve_phpbrew_version(host_id: &str, version: &str) -> Result<String, RuntimeError> {
    let normalized = normalize_php_requested_version(version);
    if normalized.chars().filter(|ch| *ch == '.').count() >= 2 {
        return Ok(normalized);
    }

    let known = hosts::run_capture_on_host(host_id, "phpbrew", &["known", "--more"])
        .or_else(|| hosts::run_capture_on_host(host_id, "phpbrew", &["known"]))
        .ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::ToolUnavailable,
                "phpbrew could not enumerate known PHP versions on this host.",
            )
        })?;

    resolve_phpbrew_version_from_known_output(&known, &normalized).ok_or_else(|| {
        RuntimeError::new(
            RuntimeErrorKind::VersionUnsupported,
            format!(
                "Could not resolve a concrete phpbrew version for `{normalized}`. Try an exact patch release like `8.2.24`."
            ),
        )
    })
}

fn resolve_phpbrew_version_from_known_output(output: &str, requested: &str) -> Option<String> {
    output
        .lines()
        .map(str::trim)
        .find_map(|line| {
            let (series, versions) = line.split_once(':')?;
            (series.trim() == requested).then_some(versions)
        })
        .and_then(|versions| {
            versions
                .split(',')
                .map(str::trim)
                .find(|entry| !entry.is_empty())
                .map(ToOwned::to_owned)
        })
}

fn active_phpbrew_version(host_id: &str) -> Option<String> {
    hosts::run_capture_on_host(host_id, "phpbrew", &["list"]).and_then(|output| {
        output.lines().map(str::trim).find_map(|line| {
            if !line.starts_with('*') {
                return None;
            }
            line.trim_start_matches('*')
                .trim()
                .split_whitespace()
                .next()
                .map(|value| value.trim_start_matches("php-").to_string())
        })
    })
}

fn require_phpbrew_remove_support(host_id: &str) -> Result<(), RuntimeError> {
    let help = hosts::run_capture_on_host(host_id, "phpbrew", &["help"]).unwrap_or_default();
    if phpbrew_supports_remove_output(&help) {
        Ok(())
    } else {
        Err(RuntimeError::new(
            RuntimeErrorKind::ToolUnavailable,
            "phpbrew remove is not available on this host. Upgrade phpbrew or remove the runtime manually, then refresh detection.",
        ))
    }
}

fn phpbrew_supports_remove_output(output: &str) -> bool {
    output.lines().map(str::trim).any(|line| {
        let token = line
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .trim_matches(|ch: char| matches!(ch, ':' | ',' | '|'));
        token == "remove"
    })
}

fn map_dependency_error_kind(kind: deps::DependencyErrorKind) -> RuntimeErrorKind {
    match kind {
        deps::DependencyErrorKind::PermissionDenied => RuntimeErrorKind::PermissionDenied,
        deps::DependencyErrorKind::ToolUnavailable => RuntimeErrorKind::ToolUnavailable,
        deps::DependencyErrorKind::HostUnavailable => RuntimeErrorKind::HostUnavailable,
        deps::DependencyErrorKind::UnsupportedPlatform => RuntimeErrorKind::UnsupportedOperation,
        deps::DependencyErrorKind::Unknown => RuntimeErrorKind::Unknown,
    }
}

fn resolve_sdkman_java_version(
    host_id: &str,
    version: &str,
    init_script: &str,
) -> Result<String, RuntimeError> {
    let requested = version.trim();
    if requested.contains('-') && requested.chars().any(|ch| ch.is_ascii_alphabetic()) {
        return Ok(requested.to_string());
    }

    let output =
        hosts::run_shell_script_on_host(host_id, &format!(". \"{init_script}\" && sdk list java"))
            .map_err(RuntimeError::classify)?;
    let prefix = if requested.ends_with('.') || requested.ends_with('-') {
        requested.to_string()
    } else {
        format!("{requested}.")
    };

    output
        .stdout
        .split_whitespace()
        .map(|token| token.trim_matches(|ch: char| matches!(ch, '>' | '*' | '+' | '|' | ',')))
        .find(|token| {
            let starts_major = token.starts_with(&prefix) || token.starts_with(&format!("{requested}-"));
            starts_major && token.contains('-') && token.chars().next().is_some_and(|ch| ch.is_ascii_digit())
        })
        .map(str::to_string)
        .ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::VersionUnsupported,
                format!(
                "Could not resolve a concrete SDKMAN! Java version for `{requested}`. Try an exact candidate like `21.0.4-tem`."
                ),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::{
        dotnet_install_script_args, normalize_go_requested_version, normalize_node_version,
        phpbrew_supports_remove_output, rbenv_supports_command_output,
        resolve_phpbrew_version_from_known_output, resolve_sdkman_java_version,
    };

    #[test]
    fn normalizes_node_labels() {
        assert_eq!(normalize_node_version("20 LTS"), "20");
        assert_eq!(normalize_node_version("node@22.5.1"), "22.5.1");
        assert_eq!(normalize_node_version("v18.20.4"), "18.20.4");
    }

    #[test]
    fn normalizes_go_request_labels() {
        assert_eq!(normalize_go_requested_version("1.22.5"), "go1.22.5");
        assert_eq!(normalize_go_requested_version("go1.21.0"), "go1.21.0");
    }

    #[test]
    fn accepts_exact_sdkman_candidate_labels() {
        assert_eq!(
            resolve_sdkman_java_version("native", "21.0.4-tem", "/tmp/unused").unwrap(),
            "21.0.4-tem"
        );
    }

    #[test]
    fn resolves_phpbrew_minor_to_latest_patch() {
        let output = "8.3: 8.3.12, 8.3.11\n8.2: 8.2.24, 8.2.23\n";
        assert_eq!(
            resolve_phpbrew_version_from_known_output(output, "8.2"),
            Some("8.2.24".into())
        );
        assert_eq!(
            resolve_phpbrew_version_from_known_output(output, "8.1"),
            None
        );
    }

    #[test]
    fn parses_phpbrew_remove_support() {
        let help = "install\nlist\nremove\nswitch\nuse\n";
        assert!(phpbrew_supports_remove_output(help));
        assert!(!phpbrew_supports_remove_output(
            "install\nlist\nswitch\nuse\n"
        ));
    }

    #[test]
    fn parses_rbenv_command_support() {
        let output = "global\ninstall\nlocal\nuninstall\nversions\n";
        assert!(rbenv_supports_command_output(output, "install"));
        assert!(rbenv_supports_command_output(output, "uninstall"));
        assert!(!rbenv_supports_command_output(output, "shell"));
    }

    #[test]
    fn resolves_dotnet_install_args() {
        assert_eq!(dotnet_install_script_args("8.0 LTS"), "--channel 8.0");
        assert_eq!(dotnet_install_script_args("9.0"), "--channel 9.0");
        assert_eq!(dotnet_install_script_args("8.0.303"), "--version 8.0.303");
    }
}
