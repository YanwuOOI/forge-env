use std::{collections::HashSet, env, fs, path::Path, path::PathBuf};

use super::{
    models::{HostDetail, HostSummary},
    shell::{self, CommandResult},
};

const WSL_PREFIX: &str = "wsl:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostErrorKind {
    HostUnavailable,
    UnsupportedPlatform,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct HostError {
    pub kind: HostErrorKind,
    pub message: String,
}

impl HostError {
    fn new(kind: HostErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    fn classify(message: impl Into<String>) -> Self {
        let message = message.into();
        let lowered = message.to_lowercase();
        let kind = if lowered.contains("unknown host")
            || lowered.contains("unknown wsl distro")
            || lowered.contains("wsl command is not available")
        {
            HostErrorKind::HostUnavailable
        } else if lowered.contains("only available on windows") {
            HostErrorKind::UnsupportedPlatform
        } else {
            HostErrorKind::Unknown
        };
        Self { kind, message }
    }

    pub fn outcome_title(&self) -> &'static str {
        match self.kind {
            HostErrorKind::HostUnavailable => "Host unavailable",
            HostErrorKind::UnsupportedPlatform => "Unsupported host platform",
            HostErrorKind::Unknown => "Host inspection failed",
        }
    }

    pub fn next_step(&self) -> &'static str {
        match self.kind {
            HostErrorKind::HostUnavailable => {
                "Re-select a reachable native or WSL host before requesting host details again."
            }
            HostErrorKind::UnsupportedPlatform => {
                "Use this host inspection path only on a platform that supports the requested host type."
            }
            HostErrorKind::Unknown => {
                "Retry host inspection after confirming the selected host topology is still available."
            }
        }
    }
}

pub fn discover_hosts() -> Vec<HostSummary> {
    let mut hosts = vec![native_host_summary()];
    hosts.extend(discover_wsl_hosts());
    hosts
}

pub fn inspect_host(host_id: &str) -> Result<HostDetail, HostError> {
    if let Some(distro) = host_id.strip_prefix(WSL_PREFIX) {
        return inspect_wsl_host(distro).map_err(HostError::classify);
    }

    let host = native_host_summary();
    if host.id != host_id {
        return Err(HostError::new(
            HostErrorKind::HostUnavailable,
            format!("Unknown host id: {host_id}"),
        ));
    }

    Ok(HostDetail {
        summary: host,
        os_version: detect_native_os_version(),
        cwd: env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .to_string(),
        home_dir: env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .unwrap_or_else(|_| "~".into()),
        path_entries_count: env::var("PATH")
            .map(|value| env::split_paths(&value).count())
            .unwrap_or(0),
        shell_profiles: detect_native_shell_profiles(),
        package_managers: detect_native_package_managers(),
        mirrors_supported: vec![
            "npm".into(),
            "pip".into(),
            "cargo".into(),
            "system-package-manager".into(),
        ],
        notes: vec![
            "Host modeling separates native OS shells from WSL distros.".into(),
            "PATH application remains a confirmed step; it is not silently mutated.".into(),
            "Windows Native and each WSL distro are treated as separate hosts to avoid state bleed."
                .into(),
        ],
    })
}

pub fn detect_native_shell_profiles() -> Vec<String> {
    let shell = env::var("SHELL").unwrap_or_default();
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| "~".into());

    shell_profiles_for_shell(&shell, &home)
}

pub fn detect_shell_profiles_for_host(host_id: &str) -> Vec<String> {
    if let Some(distro) = host_id.strip_prefix(WSL_PREFIX) {
        let shell_name =
            run_wsl_shell(distro, "basename \"${SHELL:-/bin/sh}\"").unwrap_or_else(|| "sh".into());
        let home_dir = run_wsl_shell(distro, "printf %s \"$HOME\"").unwrap_or_else(|| "~".into());
        return shell_profiles_for_shell(&shell_name, &home_dir);
    }

    detect_native_shell_profiles()
}

pub fn home_dir_for_host(host_id: &str) -> Option<String> {
    if let Some(distro) = host_id.strip_prefix(WSL_PREFIX) {
        return run_wsl_shell(distro, "printf %s \"$HOME\"");
    }

    shell::home_dir()
}

pub fn path_entries_for_host(host_id: &str) -> HashSet<String> {
    if let Some(distro) = host_id.strip_prefix(WSL_PREFIX) {
        return run_wsl_shell(distro, "printf %s \"$PATH\"")
            .map(|value| {
                value
                    .split(':')
                    .filter(|entry| !entry.is_empty())
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default();
    }

    env::var_os("PATH")
        .map(|value| {
            env::split_paths(&value)
                .map(|entry| entry.to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default()
}

pub fn run_checked_on_host(
    host_id: &str,
    command: &str,
    args: &[&str],
) -> Result<CommandResult, String> {
    if let Some(distro) = host_id.strip_prefix(WSL_PREFIX) {
        let wsl = wsl_command().ok_or_else(|| "WSL command is not available.".to_string())?;
        let mut owned = vec![
            "-d".to_string(),
            distro.to_string(),
            "--".to_string(),
            command.to_string(),
        ];
        owned.extend(args.iter().map(|value| value.to_string()));
        let refs = owned.iter().map(String::as_str).collect::<Vec<_>>();
        return shell::run_checked(wsl, &refs);
    }

    shell::run_checked(command, args)
}

pub fn run_capture_on_host(host_id: &str, command: &str, args: &[&str]) -> Option<String> {
    run_checked_on_host(host_id, command, args)
        .ok()
        .and_then(|result| {
            if result.stdout.is_empty() {
                (!result.stderr.is_empty()).then_some(result.stderr)
            } else {
                Some(result.stdout)
            }
        })
}

pub fn run_shell_script_on_host(host_id: &str, script: &str) -> Result<CommandResult, String> {
    if let Some(distro) = host_id.strip_prefix(WSL_PREFIX) {
        let wsl = wsl_command().ok_or_else(|| "WSL command is not available.".to_string())?;
        return shell::run_checked(wsl, &["-d", distro, "sh", "-lc", script]);
    }

    shell::run_shell_script(script)
}

pub fn path_exists_on_host(host_id: &str, path: &Path) -> bool {
    if let Some(distro) = host_id.strip_prefix(WSL_PREFIX) {
        let quoted = shell_quote_single(path.to_string_lossy().as_ref());
        return run_wsl_shell(distro, &format!("[ -e '{quoted}' ] && echo found")).is_some();
    }

    shell::path_exists(path)
}

pub fn create_dir_all_on_host(host_id: &str, path: &Path) -> Result<(), String> {
    if host_id.starts_with(WSL_PREFIX) {
        let quoted = shell_quote_single(path.to_string_lossy().as_ref());
        return run_shell_script_on_host(host_id, &format!("mkdir -p '{quoted}'")).map(|_| ());
    }

    fs::create_dir_all(path).map_err(|error| error.to_string())
}

pub fn read_file_on_host(host_id: &str, path: &Path) -> Result<String, String> {
    if host_id.starts_with(WSL_PREFIX) {
        let quoted = shell_quote_single(path.to_string_lossy().as_ref());
        let output = run_shell_script_on_host(
            host_id,
            &format!("if [ -f '{quoted}' ]; then cat '{quoted}'; fi"),
        )?;
        return Ok(output.stdout);
    }

    Ok(fs::read_to_string(path).unwrap_or_default())
}

pub fn write_file_on_host(host_id: &str, path: &Path, contents: &str) -> Result<(), String> {
    if host_id.starts_with(WSL_PREFIX) {
        let quoted_path = shell_quote_single(path.to_string_lossy().as_ref());
        let quoted_contents = shell_quote_single(contents);
        return run_shell_script_on_host(
            host_id,
            &format!("mkdir -p \"$(dirname '{quoted_path}')\" && printf '%s' '{quoted_contents}' > '{quoted_path}'"),
        )
        .map(|_| ());
    }

    fs::write(path, contents).map_err(|error| error.to_string())
}

fn native_host_summary() -> HostSummary {
    let kind = env::consts::OS.to_string();
    let shell_name = env::var("SHELL")
        .ok()
        .and_then(|value| value.rsplit('/').next().map(ToOwned::to_owned))
        .unwrap_or_else(|| "unknown".to_string());
    let path_preview = env::var("PATH")
        .map(|value| {
            env::split_paths(&value)
                .take(4)
                .map(|entry| entry.to_string_lossy().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    HostSummary {
        id: format!("{}-native", kind),
        label: match env::consts::OS {
            "macos" => "macOS Native Host".to_string(),
            "linux" => "Linux Native Host".to_string(),
            "windows" => "Windows Native Host".to_string(),
            _ => "Desktop Host".to_string(),
        },
        kind,
        architecture: env::consts::ARCH.to_string(),
        shell: shell_name,
        status: "ready".to_string(),
        recommended_package_manager: detect_native_package_managers()
            .first()
            .cloned()
            .unwrap_or_else(|| "system".into()),
        path_preview,
    }
}

fn detect_native_package_managers() -> Vec<String> {
    detect_package_managers_with(|command| shell::command_exists(command))
}

fn detect_native_os_version() -> String {
    if cfg!(target_os = "macos") {
        return shell::run_capture("sw_vers", &["-productVersion"])
            .map(|version| format!("macOS {version}"))
            .unwrap_or_else(|| "macOS".into());
    }

    if cfg!(target_os = "linux") {
        return shell::run_capture("uname", &["-r"]).unwrap_or_else(|| "Linux".into());
    }

    if cfg!(target_os = "windows") {
        return shell::run_capture("cmd", &["/C", "ver"]).unwrap_or_else(|| "Windows".into());
    }

    env::consts::OS.to_string()
}

fn discover_wsl_hosts() -> Vec<HostSummary> {
    if !cfg!(target_os = "windows") {
        return Vec::new();
    }

    let command = wsl_command();
    let Some(command) = command else {
        return Vec::new();
    };

    let distros = shell::run_capture(command, &["-l", "-q"])
        .map(|output| parse_wsl_list(&output))
        .unwrap_or_default();

    distros
        .into_iter()
        .map(|distro| {
            let architecture =
                run_wsl_shell(&distro, "uname -m").unwrap_or_else(|| "unknown".into());
            let package_managers = detect_wsl_package_managers(&distro);
            HostSummary {
                id: format!("{WSL_PREFIX}{distro}"),
                label: format!("WSL Distro · {distro}"),
                kind: "wsl".into(),
                architecture,
                shell: "wsl".into(),
                status: "ready".into(),
                recommended_package_manager: package_managers
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "system".into()),
                path_preview: Vec::new(),
            }
        })
        .collect()
}

fn inspect_wsl_host(distro: &str) -> Result<HostDetail, String> {
    if !cfg!(target_os = "windows") {
        return Err("WSL hosts are only available on Windows builds.".into());
    }

    let summary = discover_hosts()
        .into_iter()
        .find(|host| host.id == format!("{WSL_PREFIX}{distro}"))
        .ok_or_else(|| format!("Unknown WSL distro: {distro}"))?;

    let shell_name =
        run_wsl_shell(distro, "basename \"${SHELL:-/bin/sh}\"").unwrap_or_else(|| "sh".into());
    let home_dir = run_wsl_shell(distro, "printf %s \"$HOME\"").unwrap_or_else(|| "~".into());
    let cwd = run_wsl_shell(distro, "pwd").unwrap_or_else(|| "~".into());
    let path = run_wsl_shell(distro, "printf %s \"$PATH\"").unwrap_or_default();
    let path_entries = path.split(':').filter(|entry| !entry.is_empty()).count();
    let os_version = run_wsl_shell(
        distro,
        "if [ -f /etc/os-release ]; then . /etc/os-release && printf \"%s\" \"${PRETTY_NAME:-Linux}\"; else uname -sr; fi",
    )
    .unwrap_or_else(|| "Linux".into());

    Ok(HostDetail {
        summary,
        os_version,
        cwd,
        home_dir: home_dir.clone(),
        path_entries_count: path_entries,
        shell_profiles: shell_profiles_for_shell(&shell_name, &home_dir),
        package_managers: detect_wsl_package_managers(distro),
        mirrors_supported: vec![
            "npm".into(),
            "pip".into(),
            "cargo".into(),
            "system-package-manager".into(),
        ],
        notes: vec![
            "WSL distros are modeled separately from Windows Native so PATH and package managers do not mix."
                .into(),
            "Runtime management is still host-local; activating a toolchain in WSL does not affect Windows Native."
                .into(),
        ],
    })
}

fn detect_wsl_package_managers(distro: &str) -> Vec<String> {
    detect_package_managers_with(|command| {
        run_wsl_shell(
            distro,
            &format!("command -v {command} >/dev/null 2>&1 && echo found"),
        )
        .is_some()
    })
}

fn detect_package_managers_with<F>(mut command_exists: F) -> Vec<String>
where
    F: FnMut(&str) -> bool,
{
    let candidates = [
        ("brew", "Homebrew"),
        ("winget", "winget"),
        ("apt", "apt"),
        ("dnf", "dnf"),
        ("yum", "yum"),
        ("pacman", "pacman"),
        ("zypper", "zypper"),
    ];

    candidates
        .into_iter()
        .filter_map(|(command, label)| command_exists(command).then_some(label.to_string()))
        .collect()
}

fn shell_profiles_for_shell(shell: &str, home: &str) -> Vec<String> {
    if shell.ends_with("zsh") {
        return vec![format!("{home}/.zshrc"), format!("{home}/.zprofile")];
    }
    if shell.ends_with("bash") {
        return vec![format!("{home}/.bashrc"), format!("{home}/.bash_profile")];
    }
    if shell.ends_with("fish") {
        return vec![format!("{home}/.config/fish/config.fish")];
    }

    vec![format!("{home}/.profile")]
}

fn wsl_command() -> Option<&'static str> {
    if shell::command_exists("wsl.exe") {
        Some("wsl.exe")
    } else if shell::command_exists("wsl") {
        Some("wsl")
    } else {
        None
    }
}

fn run_wsl_shell(distro: &str, script: &str) -> Option<String> {
    let command = wsl_command()?;
    shell::run_capture(command, &["-d", distro, "sh", "-lc", script])
}

fn shell_quote_single(value: &str) -> String {
    value.replace('\'', "'\"'\"'")
}

fn parse_wsl_list(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|line| line.replace('\0', ""))
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_wsl_list;

    #[test]
    fn parses_wsl_list_output() {
        let output = "Ubuntu\nDebian\0\n\nArch\n";
        assert_eq!(parse_wsl_list(output), vec!["Ubuntu", "Debian", "Arch"]);
    }
}
