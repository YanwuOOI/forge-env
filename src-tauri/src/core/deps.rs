use super::{hosts, models::SystemDependencyState, shell};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyErrorKind {
    PermissionDenied,
    ToolUnavailable,
    HostUnavailable,
    UnsupportedPlatform,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DependencyError {
    pub kind: DependencyErrorKind,
    pub message: String,
}

impl DependencyError {
    fn new(kind: DependencyErrorKind, message: impl Into<String>) -> Self {
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
            DependencyErrorKind::PermissionDenied
        } else if lowered.contains("unknown host")
            || lowered.contains("wsl command is not available")
        {
            DependencyErrorKind::HostUnavailable
        } else if lowered.contains("no supported system package manager")
            || lowered.contains("unsupported")
        {
            DependencyErrorKind::UnsupportedPlatform
        } else if lowered.contains("command not found") || lowered.contains("not found") {
            DependencyErrorKind::ToolUnavailable
        } else {
            DependencyErrorKind::Unknown
        };
        Self { kind, message }
    }

    pub fn outcome_title(&self) -> &'static str {
        match self.kind {
            DependencyErrorKind::PermissionDenied => "Permission denied",
            DependencyErrorKind::ToolUnavailable => "Package manager unavailable",
            DependencyErrorKind::HostUnavailable => "Host unavailable",
            DependencyErrorKind::UnsupportedPlatform => "Unsupported package manager",
            DependencyErrorKind::Unknown => "Dependency install failed",
        }
    }

    pub fn next_step(&self, target_name: &str) -> String {
        match self.kind {
            DependencyErrorKind::PermissionDenied => format!(
                "Grant the required package manager permission or rerun the {} install through a user that can modify the host toolchain.",
                target_name
            ),
            DependencyErrorKind::ToolUnavailable => format!(
                "Install or expose a supported package manager for {} before retrying.",
                target_name
            ),
            DependencyErrorKind::HostUnavailable => {
                "Re-select a reachable host or restore the missing native/WSL execution path before retrying."
                    .into()
            }
            DependencyErrorKind::UnsupportedPlatform => format!(
                "Use a supported host package manager for {} or install these dependencies manually.",
                target_name
            ),
            DependencyErrorKind::Unknown => format!(
                "Inspect host package manager output for {}, then retry once the failure cause is understood.",
                target_name
            ),
        }
    }
}

const DEPENDENCIES: &[(&str, &str, &[&str], &str)] = &[
    (
        "Git",
        "git",
        &["--version"],
        "Required for source control and most language installers.",
    ),
    (
        "SSH",
        "ssh",
        &["-V"],
        "Needed for Git auth and remote host access.",
    ),
    (
        "OpenSSL",
        "openssl",
        &["version"],
        "Common TLS dependency for language builds.",
    ),
    (
        "curl",
        "curl",
        &["--version"],
        "Universal download helper for installers.",
    ),
    (
        "wget",
        "wget",
        &["--version"],
        "Alternative downloader on Linux-centric setups.",
    ),
    (
        "CMake",
        "cmake",
        &["--version"],
        "Needed for native library and C/C++ builds.",
    ),
    (
        "GCC",
        "gcc",
        &["--version"],
        "Common compiler for Python/Ruby native extensions.",
    ),
    (
        "Clang",
        "clang",
        &["--version"],
        "Primary compiler on Apple platforms and many Linux toolchains.",
    ),
    (
        "pkg-config",
        "pkg-config",
        &["--version"],
        "Discovers native libraries for build scripts and language bindings.",
    ),
    (
        "FFmpeg",
        "ffmpeg",
        &["-version"],
        "Common multimedia dependency used by toolchains, CLIs, and SDK samples.",
    ),
];

pub fn detect_system_dependencies() -> Vec<SystemDependencyState> {
    detect_system_dependencies_for_host("native")
}

pub fn detect_system_dependencies_for_host(host_id: &str) -> Vec<SystemDependencyState> {
    DEPENDENCIES
        .iter()
        .map(|(name, command, args, hint)| {
            let version = hosts::run_capture_on_host(host_id, command, args).map(|output| {
                output
                    .lines()
                    .next()
                    .unwrap_or(output.as_str())
                    .trim()
                    .to_string()
            });

            SystemDependencyState {
                name: (*name).to_string(),
                command: (*command).to_string(),
                installed: version.is_some() || host_command_exists(host_id, command),
                version,
                source_hint: (*hint).to_string(),
            }
        })
        .collect()
}

pub fn install_dependency_template_for_host(
    host_id: &str,
    requested: &[String],
) -> Result<String, DependencyError> {
    if requested.is_empty() {
        return Err(DependencyError::new(
            DependencyErrorKind::UnsupportedPlatform,
            "No dependencies were selected for installation.",
        ));
    }

    let plan = package_manager_plan_for_host(host_id)?;
    let arg_refs = plan.args.iter().map(String::as_str).collect::<Vec<_>>();
    let result = hosts::run_checked_on_host(host_id, plan.command, &arg_refs)
        .map_err(DependencyError::classify)?;
    let output = summarize_output(&result.stdout, &result.stderr);

    let selected = requested.join(", ");
    if output.is_empty() {
        Ok(format!(
            "Installed dependency template via {} for: {selected}.",
            plan.label
        ))
    } else {
        Ok(format!(
            "Installed dependency template via {} for: {selected}. {}",
            plan.label, output
        ))
    }
}

pub fn supports_cpp_toolchain_install_for_host(host_id: &str) -> bool {
    host_command_exists(host_id, "brew")
        || host_command_exists(host_id, "apt")
        || host_command_exists(host_id, "dnf")
        || host_command_exists(host_id, "yum")
        || host_command_exists(host_id, "pacman")
        || host_command_exists(host_id, "zypper")
}

pub fn install_cpp_toolchain_for_host(
    host_id: &str,
    toolchain: &str,
) -> Result<String, DependencyError> {
    let plan = cpp_toolchain_plan_for_host(host_id, toolchain)?;
    let arg_refs = plan.args.iter().map(String::as_str).collect::<Vec<_>>();
    let result = hosts::run_checked_on_host(host_id, plan.command, &arg_refs)
        .map_err(DependencyError::classify)?;
    let output = summarize_output(&result.stdout, &result.stderr);

    if output.is_empty() {
        Ok(format!(
            "Installed C/C++ {} toolchain template via {}.",
            toolchain, plan.label
        ))
    } else {
        Ok(format!(
            "Installed C/C++ {} toolchain template via {}. {}",
            toolchain, plan.label, output
        ))
    }
}

struct PackageManagerPlan {
    label: &'static str,
    command: &'static str,
    args: Vec<String>,
}

fn cpp_toolchain_plan_for_host(
    host_id: &str,
    toolchain: &str,
) -> Result<PackageManagerPlan, DependencyError> {
    let requested = toolchain.trim();
    if requested.eq_ignore_ascii_case("MSVC") {
        return Err(DependencyError::new(
            DependencyErrorKind::UnsupportedPlatform,
            "Managed MSVC installation is not wired yet. Use Visual Studio Build Tools or the Visual Studio installer on Windows hosts.",
        ));
    }

    if host_command_exists(host_id, "brew") {
        let args = if requested.eq_ignore_ascii_case("Clang") {
            vec!["install", "llvm", "cmake", "ninja"]
        } else {
            vec!["install", "gcc", "cmake", "make"]
        };
        return Ok(PackageManagerPlan {
            label: "Homebrew",
            command: "brew",
            args: args.into_iter().map(str::to_string).collect(),
        });
    }

    if host_command_exists(host_id, "apt") {
        let args = if requested.eq_ignore_ascii_case("Clang") {
            vec![
                "install",
                "-y",
                "clang",
                "cmake",
                "ninja-build",
                "build-essential",
            ]
        } else {
            vec![
                "install",
                "-y",
                "build-essential",
                "g++",
                "cmake",
                "make",
                "ninja-build",
            ]
        };
        return Ok(PackageManagerPlan {
            label: "apt",
            command: "apt",
            args: args.into_iter().map(str::to_string).collect(),
        });
    }

    if host_command_exists(host_id, "dnf") {
        let args = if requested.eq_ignore_ascii_case("Clang") {
            vec![
                "install",
                "-y",
                "clang",
                "cmake",
                "ninja-build",
                "gcc",
                "gcc-c++",
            ]
        } else {
            vec![
                "install",
                "-y",
                "gcc",
                "gcc-c++",
                "cmake",
                "make",
                "ninja-build",
            ]
        };
        return Ok(PackageManagerPlan {
            label: "dnf",
            command: "dnf",
            args: args.into_iter().map(str::to_string).collect(),
        });
    }

    if host_command_exists(host_id, "yum") {
        let args = if requested.eq_ignore_ascii_case("Clang") {
            vec![
                "install",
                "-y",
                "clang",
                "cmake",
                "ninja-build",
                "gcc",
                "gcc-c++",
            ]
        } else {
            vec![
                "install",
                "-y",
                "gcc",
                "gcc-c++",
                "cmake",
                "make",
                "ninja-build",
            ]
        };
        return Ok(PackageManagerPlan {
            label: "yum",
            command: "yum",
            args: args.into_iter().map(str::to_string).collect(),
        });
    }

    if host_command_exists(host_id, "pacman") {
        let args = if requested.eq_ignore_ascii_case("Clang") {
            vec!["-S", "--noconfirm", "clang", "cmake", "ninja", "base-devel"]
        } else {
            vec![
                "-S",
                "--noconfirm",
                "gcc",
                "cmake",
                "make",
                "ninja",
                "base-devel",
            ]
        };
        return Ok(PackageManagerPlan {
            label: "pacman",
            command: "pacman",
            args: args.into_iter().map(str::to_string).collect(),
        });
    }

    if host_command_exists(host_id, "zypper") {
        let args = if requested.eq_ignore_ascii_case("Clang") {
            vec!["install", "-y", "clang", "cmake", "ninja", "gcc", "gcc-c++"]
        } else {
            vec!["install", "-y", "gcc", "gcc-c++", "cmake", "make", "ninja"]
        };
        return Ok(PackageManagerPlan {
            label: "zypper",
            command: "zypper",
            args: args.into_iter().map(str::to_string).collect(),
        });
    }

    Err(DependencyError::new(
        DependencyErrorKind::UnsupportedPlatform,
        "No supported Unix-like package manager was detected for C/C++ toolchain installation.",
    ))
}

fn package_manager_plan_for_host(host_id: &str) -> Result<PackageManagerPlan, DependencyError> {
    if host_command_exists(host_id, "brew") {
        return Ok(PackageManagerPlan {
            label: "Homebrew",
            command: "brew",
            args: [
                "install",
                "git",
                "openssh",
                "openssl@3",
                "curl",
                "wget",
                "cmake",
                "gcc",
                "llvm",
                "pkg-config",
                "ffmpeg",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        });
    }

    if host_command_exists(host_id, "apt") {
        return Ok(PackageManagerPlan {
            label: "apt",
            command: "apt",
            args: [
                "install",
                "-y",
                "git",
                "openssh-client",
                "libssl-dev",
                "curl",
                "wget",
                "cmake",
                "build-essential",
                "clang",
                "pkg-config",
                "zlib1g-dev",
                "ffmpeg",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        });
    }

    if host_command_exists(host_id, "dnf") {
        return Ok(PackageManagerPlan {
            label: "dnf",
            command: "dnf",
            args: [
                "install",
                "-y",
                "git",
                "openssh-clients",
                "openssl-devel",
                "curl",
                "wget",
                "cmake",
                "gcc",
                "gcc-c++",
                "clang",
                "pkgconf-pkg-config",
                "zlib-devel",
                "ffmpeg",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        });
    }

    if host_command_exists(host_id, "yum") {
        return Ok(PackageManagerPlan {
            label: "yum",
            command: "yum",
            args: [
                "install",
                "-y",
                "git",
                "openssh-clients",
                "openssl-devel",
                "curl",
                "wget",
                "cmake",
                "gcc",
                "gcc-c++",
                "clang",
                "pkgconfig",
                "zlib-devel",
                "ffmpeg",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        });
    }

    if host_command_exists(host_id, "pacman") {
        return Ok(PackageManagerPlan {
            label: "pacman",
            command: "pacman",
            args: [
                "-S",
                "--noconfirm",
                "git",
                "openssh",
                "openssl",
                "curl",
                "wget",
                "cmake",
                "gcc",
                "clang",
                "pkgconf",
                "zlib",
                "ffmpeg",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        });
    }

    if host_command_exists(host_id, "zypper") {
        return Ok(PackageManagerPlan {
            label: "zypper",
            command: "zypper",
            args: [
                "install",
                "-y",
                "git",
                "openssh",
                "libopenssl-devel",
                "curl",
                "wget",
                "cmake",
                "gcc",
                "gcc-c++",
                "clang",
                "pkg-config",
                "zlib-devel",
                "ffmpeg",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        });
    }

    if host_command_exists(host_id, "winget") {
        return Ok(PackageManagerPlan {
            label: "winget",
            command: "winget",
            args: [
                "install",
                "--accept-source-agreements",
                "--accept-package-agreements",
                "Git.Git",
                "Kitware.CMake",
                "LLVM.LLVM",
                "Gyan.FFmpeg",
                "ShiningLight.OpenSSL.Light",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        });
    }

    Err(DependencyError::new(
        DependencyErrorKind::UnsupportedPlatform,
        "No supported system package manager was detected for dependency template installation.",
    ))
}

fn host_command_exists(host_id: &str, command: &str) -> bool {
    if host_id.starts_with("wsl:") {
        hosts::run_capture_on_host(
            host_id,
            "sh",
            &[
                "-lc",
                &format!("command -v {command} >/dev/null 2>&1 && echo found"),
            ],
        )
        .is_some()
    } else {
        shell::command_exists(command)
    }
}

fn summarize_output(stdout: &str, stderr: &str) -> String {
    let source = if stdout.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };

    source
        .lines()
        .take(2)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
