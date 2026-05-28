use super::{
    deps, hosts,
    models::{RuntimeCapabilities, RuntimeFamilyState, RuntimeInstallation},
    shell,
};
use std::path::Path;

pub trait RuntimeProvider {
    fn detect(&self) -> RuntimeFamilyState;
}

pub fn detect_runtimes() -> Vec<RuntimeFamilyState> {
    vec![
        PythonProvider.detect(),
        NodeProvider.detect(),
        RustProvider.detect(),
        JavaProvider.detect(),
        GoProvider.detect(),
        DotNetProvider.detect(),
        PhpProvider.detect(),
        RubyProvider.detect(),
        CppProvider.detect(),
    ]
}

pub fn detect_runtimes_for_host(host_id: &str) -> Vec<RuntimeFamilyState> {
    vec![
        PythonProvider.detect_for_host(host_id),
        NodeProvider.detect_for_host(host_id),
        RustProvider.detect_for_host(host_id),
        JavaProvider.detect_for_host(host_id),
        GoProvider.detect_for_host(host_id),
        DotNetProvider.detect_for_host(host_id),
        PhpProvider.detect_for_host(host_id),
        RubyProvider.detect_for_host(host_id),
        CppProvider.detect_for_host(host_id),
    ]
}

struct PythonProvider;
struct NodeProvider;
struct RustProvider;
struct JavaProvider;
struct GoProvider;
struct DotNetProvider;
struct PhpProvider;
struct RubyProvider;
struct CppProvider;

trait HostAwareRuntimeProvider {
    fn detect_for_host(&self, host_id: &str) -> RuntimeFamilyState;
}

fn runtime_capabilities(
    can_install: bool,
    can_activate: bool,
    can_remove: bool,
) -> RuntimeCapabilities {
    RuntimeCapabilities {
        can_install,
        can_activate,
        can_remove,
    }
}

impl RuntimeProvider for PythonProvider {
    fn detect(&self) -> RuntimeFamilyState {
        self.detect_for_host("native")
    }
}

impl HostAwareRuntimeProvider for PythonProvider {
    fn detect_for_host(&self, host_id: &str) -> RuntimeFamilyState {
        let pyenv_path = command_path_for_host(host_id, "pyenv");
        let python_path = command_path_for_host(host_id, "python3")
            .or_else(|| command_path_for_host(host_id, "python"));

        let mut notes = vec![
            "Canonical provider is pyenv with pip, pipenv, and poetry layered above it.".into(),
        ];
        let recommended_versions = vec!["3.11".into(), "3.12".into(), "3.13".into()];
        let package_tools = vec!["pip".into(), "pipenv".into(), "poetry".into()];
        let mirrors = vec!["PyPI".into(), "Tsinghua".into(), "Aliyun".into()];

        if let Some(path) = pyenv_path {
            let active_name = hosts::run_capture_on_host(host_id, "pyenv", &["version-name"]);
            let installed = hosts::run_capture_on_host(host_id, "pyenv", &["versions", "--bare"])
                .map(|value| {
                    value
                        .lines()
                        .map(str::trim)
                        .filter(|line| !line.is_empty())
                        .map(|line| RuntimeInstallation {
                            version: line.to_string(),
                            channel: "managed".into(),
                            active: active_name
                                .as_ref()
                                .map(|active| active.split_whitespace().next() == Some(line))
                                .unwrap_or(false),
                            source: "pyenv".into(),
                            tools: package_tools.clone(),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            if installed.is_empty() {
                notes.push(
                    "pyenv is installed but no managed Python versions were detected.".into(),
                );
            }

            return RuntimeFamilyState {
                family: "Python".into(),
                provider: "pyenv".into(),
                provider_status: "ready".into(),
                detected_binary: Some(path),
                health: if installed.is_empty() {
                    "attention".into()
                } else {
                    "good".into()
                },
                package_tools,
                mirrors,
                recommended_versions,
                capabilities: runtime_capabilities(true, true, true),
                notes,
                installed,
            };
        }

        let installed = python_path
            .as_ref()
            .and_then(|_| {
                hosts::run_capture_on_host(host_id, "python3", &["--version"])
                    .or_else(|| hosts::run_capture_on_host(host_id, "python", &["--version"]))
            })
            .map(|version| {
                vec![RuntimeInstallation {
                    version: version.replace("Python ", ""),
                    channel: "system".into(),
                    active: true,
                    source: "system".into(),
                    tools: vec!["pip".into()],
                }]
            })
            .unwrap_or_default();

        if installed.is_empty() {
            notes.push("Neither pyenv nor a system Python interpreter was detected.".into());
        } else {
            notes.push("System Python is available, but pyenv is not installed yet.".into());
        }

        RuntimeFamilyState {
            family: "Python".into(),
            provider: "pyenv".into(),
            provider_status: if installed.is_empty() {
                "missing".into()
            } else {
                "fallback-system".into()
            },
            detected_binary: python_path,
            health: if installed.is_empty() {
                "missing".into()
            } else {
                "attention".into()
            },
            package_tools,
            mirrors,
            recommended_versions,
            capabilities: runtime_capabilities(false, false, false),
            notes,
            installed,
        }
    }
}

impl RuntimeProvider for NodeProvider {
    fn detect(&self) -> RuntimeFamilyState {
        self.detect_for_host("native")
    }
}

impl HostAwareRuntimeProvider for NodeProvider {
    fn detect_for_host(&self, host_id: &str) -> RuntimeFamilyState {
        let volta_path = command_path_for_host(host_id, "volta");
        let node_path = command_path_for_host(host_id, "node");
        let mut notes = vec!["Canonical provider is Volta because it stays cross-platform across macOS, Linux, and Windows.".into()];
        let recommended_versions = vec!["18 LTS".into(), "20 LTS".into(), "22 LTS".into()];
        let package_tools = vec!["npm".into(), "yarn".into(), "pnpm".into()];
        let mirrors = vec!["npm".into(), "npmmirror".into()];

        if let Some(path) = volta_path {
            let node_version = hosts::run_capture_on_host(host_id, "node", &["--version"])
                .unwrap_or_else(|| "managed".into())
                .trim_start_matches('v')
                .to_string();
            notes.push("Volta is installed; active Node version is derived from the current shimmed node binary.".into());

            return RuntimeFamilyState {
                family: "Node.js".into(),
                provider: "Volta".into(),
                provider_status: "ready".into(),
                detected_binary: Some(path),
                health: "good".into(),
                package_tools: package_tools.clone(),
                mirrors,
                recommended_versions,
                capabilities: runtime_capabilities(true, true, true),
                notes,
                installed: vec![RuntimeInstallation {
                    version: node_version,
                    channel: "managed".into(),
                    active: true,
                    source: "Volta".into(),
                    tools: package_tools,
                }],
            };
        }

        let installed = node_path
            .as_ref()
            .and_then(|_| hosts::run_capture_on_host(host_id, "node", &["--version"]))
            .map(|version| {
                vec![RuntimeInstallation {
                    version: version.trim_start_matches('v').to_string(),
                    channel: "system".into(),
                    active: true,
                    source: "system".into(),
                    tools: vec!["npm".into()],
                }]
            })
            .unwrap_or_default();

        if installed.is_empty() {
            notes.push("Neither Volta nor a system Node.js installation was detected.".into());
        } else {
            notes.push("System Node.js is available, but Volta is not installed yet.".into());
        }

        RuntimeFamilyState {
            family: "Node.js".into(),
            provider: "Volta".into(),
            provider_status: if installed.is_empty() {
                "missing".into()
            } else {
                "fallback-system".into()
            },
            detected_binary: node_path,
            health: if installed.is_empty() {
                "missing".into()
            } else {
                "attention".into()
            },
            package_tools,
            mirrors,
            recommended_versions,
            capabilities: runtime_capabilities(false, false, false),
            notes,
            installed,
        }
    }
}

impl RuntimeProvider for RustProvider {
    fn detect(&self) -> RuntimeFamilyState {
        self.detect_for_host("native")
    }
}

impl HostAwareRuntimeProvider for RustProvider {
    fn detect_for_host(&self, host_id: &str) -> RuntimeFamilyState {
        let rustup_path = command_path_for_host(host_id, "rustup");
        let cargo_path = command_path_for_host(host_id, "cargo");
        let mut notes =
            vec!["Canonical provider is rustup with cargo as the built-in package tool.".into()];
        let recommended_versions = vec!["stable".into(), "beta".into(), "nightly".into()];
        let package_tools = vec!["cargo".into()];
        let mirrors = vec!["crates.io".into(), "rsproxy".into()];

        if let Some(path) = rustup_path {
            let active_toolchain =
                hosts::run_capture_on_host(host_id, "rustup", &["show", "active-toolchain"])
                    .and_then(|value| value.split_whitespace().next().map(ToOwned::to_owned));
            let installed = hosts::run_capture_on_host(host_id, "rustup", &["toolchain", "list"])
                .map(|value| {
                    value
                        .lines()
                        .map(str::trim)
                        .filter(|line| !line.is_empty())
                        .map(|line| line.split_whitespace().next().unwrap_or(line))
                        .map(|line| RuntimeInstallation {
                            version: line.to_string(),
                            channel: line.to_string(),
                            active: active_toolchain.as_deref() == Some(line),
                            source: "rustup".into(),
                            tools: package_tools.clone(),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            return RuntimeFamilyState {
                family: "Rust".into(),
                provider: "rustup".into(),
                provider_status: "ready".into(),
                detected_binary: Some(path),
                health: "good".into(),
                package_tools,
                mirrors,
                recommended_versions,
                capabilities: runtime_capabilities(true, true, true),
                notes,
                installed,
            };
        }

        let installed = cargo_path
            .as_ref()
            .and_then(|_| {
                hosts::run_capture_on_host(host_id, "rustc", &["--version"])
                    .or_else(|| hosts::run_capture_on_host(host_id, "cargo", &["--version"]))
            })
            .map(|version| {
                let normalized = version
                    .replace("rustc ", "")
                    .replace("cargo ", "")
                    .split_whitespace()
                    .next()
                    .unwrap_or("system")
                    .to_string();
                vec![RuntimeInstallation {
                    version: normalized,
                    channel: "system".into(),
                    active: true,
                    source: "system".into(),
                    tools: package_tools.clone(),
                }]
            })
            .unwrap_or_default();

        if installed.is_empty() {
            notes.push("Neither rustup nor a system Rust toolchain was detected.".into());
        } else {
            notes.push("System Rust tooling is available, but rustup is not installed yet.".into());
        }

        RuntimeFamilyState {
            family: "Rust".into(),
            provider: "rustup".into(),
            provider_status: if installed.is_empty() {
                "missing".into()
            } else {
                "fallback-system".into()
            },
            detected_binary: cargo_path,
            health: if installed.is_empty() {
                "missing".into()
            } else {
                "attention".into()
            },
            package_tools,
            mirrors,
            recommended_versions,
            capabilities: runtime_capabilities(false, false, false),
            notes,
            installed,
        }
    }
}

impl RuntimeProvider for JavaProvider {
    fn detect(&self) -> RuntimeFamilyState {
        self.detect_for_host("native")
    }
}

impl HostAwareRuntimeProvider for JavaProvider {
    fn detect_for_host(&self, host_id: &str) -> RuntimeFamilyState {
        let home = hosts::home_dir_for_host(host_id).unwrap_or_default();
        let sdkman_root = format!("{home}/.sdkman");
        let sdkman_java_root = format!("{sdkman_root}/candidates/java");
        let sdkman_init = format!("{sdkman_root}/bin/sdkman-init.sh");
        let sdkman_path =
            hosts::path_exists_on_host(host_id, Path::new(&sdkman_init)).then_some(sdkman_init);
        let java_path = command_path_for_host(host_id, "java");
        let javac_path = command_path_for_host(host_id, "javac");
        let mut notes = vec![
            "Canonical provider target is SDKMAN! on Unix-like hosts, with project-specific version hints layered above it."
                .into(),
            "Java install, switch, and remove are supported when SDKMAN! is present."
                .into(),
        ];
        let recommended_versions = vec!["8".into(), "11".into(), "17".into(), "21".into()];
        let package_tools = vec!["Maven".into(), "Gradle".into()];
        let mirrors = vec![
            "Maven Central".into(),
            "Aliyun Maven".into(),
            "Tsinghua Maven".into(),
        ];

        if let Some(path) = sdkman_path {
            let current = std::path::Path::new(&sdkman_java_root)
                .join("current")
                .read_link()
                .ok()
                .and_then(|target| {
                    target
                        .file_name()
                        .map(|value| value.to_string_lossy().to_string())
                });
            let installed = std::fs::read_dir(&sdkman_java_root)
                .ok()
                .into_iter()
                .flat_map(|entries| entries.flatten())
                .filter_map(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();
                    (name != "current" && entry.path().is_dir()).then_some(name)
                })
                .map(|version| RuntimeInstallation {
                    active: current.as_deref() == Some(version.as_str()),
                    channel: "managed".into(),
                    source: "SDKMAN!".into(),
                    tools: package_tools.clone(),
                    version,
                })
                .collect::<Vec<_>>();

            if installed.is_empty() {
                notes.push("SDKMAN! is present but no managed Java candidates were found.".into());
            }

            return RuntimeFamilyState {
                family: "Java".into(),
                provider: "SDKMAN!".into(),
                provider_status: "ready".into(),
                detected_binary: Some(path),
                health: if installed.is_empty() {
                    "attention".into()
                } else {
                    "good".into()
                },
                package_tools,
                mirrors,
                recommended_versions,
                capabilities: runtime_capabilities(true, true, true),
                notes,
                installed,
            };
        }

        let installed = java_path
            .as_ref()
            .and_then(|_| {
                shell::run_capture("java", &["-version"])
                    .or_else(|| shell::run_capture("javac", &["-version"]))
            })
            .map(|version| {
                let normalized = normalize_java_version(&version);
                vec![RuntimeInstallation {
                    version: normalized,
                    channel: "system".into(),
                    active: true,
                    source: "system".into(),
                    tools: package_tools.clone(),
                }]
            })
            .unwrap_or_default();

        if installed.is_empty() {
            notes.push("Neither SDKMAN! nor a system Java runtime was detected.".into());
        } else {
            notes.push(
                "System Java is available, but SDKMAN! is not configured on this host.".into(),
            );
        }
        if javac_path.is_none() && !installed.is_empty() {
            notes.push(
                "`javac` was not found; the current host may only have a JRE, not a full JDK."
                    .into(),
            );
        }

        RuntimeFamilyState {
            family: "Java".into(),
            provider: "SDKMAN!".into(),
            provider_status: if installed.is_empty() {
                "missing".into()
            } else {
                "fallback-system".into()
            },
            detected_binary: java_path.or(javac_path),
            health: if installed.is_empty() {
                "missing".into()
            } else {
                "attention".into()
            },
            package_tools,
            mirrors,
            recommended_versions,
            capabilities: runtime_capabilities(false, false, false),
            notes,
            installed,
        }
    }
}

impl RuntimeProvider for GoProvider {
    fn detect(&self) -> RuntimeFamilyState {
        self.detect_for_host("native")
    }
}

impl HostAwareRuntimeProvider for GoProvider {
    fn detect_for_host(&self, host_id: &str) -> RuntimeFamilyState {
        let home = hosts::home_dir_for_host(host_id).unwrap_or_default();
        let gvm_script = format!("{home}/.gvm/scripts/gvm");
        let gvm_path =
            hosts::path_exists_on_host(host_id, Path::new(&gvm_script)).then_some(gvm_script);
        let go_path = command_path_for_host(host_id, "go");
        let mut notes = vec![
            "Canonical provider target is the official Go toolchain; `gvm` remains optional rather than required."
                .into(),
            "Go install and switch are supported when gvm is present; per-version remove is still pending."
                .into(),
        ];
        let recommended_versions = vec!["1.21".into(), "1.22".into(), "1.23".into()];
        let package_tools = vec!["go mod".into()];
        let mirrors = vec![
            "proxy.golang.org".into(),
            "goproxy.cn".into(),
            "goproxy.io".into(),
        ];

        if let Some(path) = gvm_path {
            let installed = shell::run_shell_script(&format!(". \"{path}\" && gvm list"))
                .ok()
                .map(|value| parse_gvm_versions(&value.stdout, &package_tools))
                .unwrap_or_default();

            return RuntimeFamilyState {
                family: "Go".into(),
                provider: "gvm".into(),
                provider_status: "ready".into(),
                detected_binary: Some(path),
                health: if installed.is_empty() {
                    "attention".into()
                } else {
                    "good".into()
                },
                package_tools,
                mirrors,
                recommended_versions,
                capabilities: runtime_capabilities(true, true, false),
                notes,
                installed,
            };
        }

        let installed = go_path
            .as_ref()
            .and_then(|_| hosts::run_capture_on_host(host_id, "go", &["version"]))
            .map(|version| {
                vec![RuntimeInstallation {
                    version: normalize_go_version(&version),
                    channel: "system".into(),
                    active: true,
                    source: "system".into(),
                    tools: package_tools.clone(),
                }]
            })
            .unwrap_or_default();

        if installed.is_empty() {
            notes.push("No Go toolchain was detected on this host.".into());
        } else {
            notes.push("System Go is available; a version manager is not required for the second-batch baseline.".into());
        }

        RuntimeFamilyState {
            family: "Go".into(),
            provider: "Go toolchain".into(),
            provider_status: if installed.is_empty() {
                "missing".into()
            } else {
                "fallback-system".into()
            },
            detected_binary: go_path,
            health: if installed.is_empty() {
                "missing".into()
            } else {
                "attention".into()
            },
            package_tools,
            mirrors,
            recommended_versions,
            capabilities: runtime_capabilities(false, false, false),
            notes,
            installed,
        }
    }
}

impl RuntimeProvider for DotNetProvider {
    fn detect(&self) -> RuntimeFamilyState {
        self.detect_for_host("native")
    }
}

impl RuntimeProvider for PhpProvider {
    fn detect(&self) -> RuntimeFamilyState {
        self.detect_for_host("native")
    }
}

impl HostAwareRuntimeProvider for PhpProvider {
    fn detect_for_host(&self, host_id: &str) -> RuntimeFamilyState {
        let phpbrew_path = command_path_for_host(host_id, "phpbrew");
        let php_path = command_path_for_host(host_id, "php");
        let composer_path = command_path_for_host(host_id, "composer");
        let package_tools = vec!["composer".into()];
        let mirrors = vec!["Packagist".into(), "Alibaba Cloud Packagist".into()];
        let recommended_versions = vec!["8.1".into(), "8.2".into(), "8.3".into()];
        let mut notes = vec![
            "Canonical provider target is phpbrew, with Composer layered above the selected PHP runtime."
                .into(),
        ];

        if composer_path.is_none() {
            notes.push("Composer was not detected on this host.".into());
        }

        if let Some(path) = phpbrew_path {
            let can_remove = phpbrew_supports_remove_for_host(host_id);
            let installed = hosts::run_capture_on_host(host_id, "phpbrew", &["list"])
                .map(|value| parse_phpbrew_versions_output(&value, &package_tools))
                .unwrap_or_default();

            if installed.is_empty() {
                notes.push(
                    "phpbrew is present, but Forge Env could not enumerate managed PHP versions."
                        .into(),
                );
            } else {
                notes.push(
                    "Forge Env can install and activate phpbrew-managed PHP versions on this host."
                        .into(),
                );
            }
            if can_remove {
                notes.push(
                    "Forge Env can also remove inactive phpbrew-managed PHP versions on this host."
                        .into(),
                );
            } else {
                notes.push(
                    "This phpbrew build does not expose a managed remove command, so Forge Env keeps PHP removal disabled."
                        .into(),
                );
            }

            return RuntimeFamilyState {
                family: "PHP".into(),
                provider: "phpbrew".into(),
                provider_status: "ready".into(),
                detected_binary: Some(path),
                health: if installed.is_empty() {
                    "attention".into()
                } else {
                    "good".into()
                },
                package_tools,
                mirrors,
                recommended_versions,
                capabilities: runtime_capabilities(true, true, can_remove),
                notes,
                installed,
            };
        }

        let installed = php_path
            .as_ref()
            .and_then(|_| hosts::run_capture_on_host(host_id, "php", &["--version"]))
            .map(|version| {
                vec![RuntimeInstallation {
                    version: normalize_php_version(&version),
                    channel: "system".into(),
                    active: true,
                    source: "system".into(),
                    tools: package_tools.clone(),
                }]
            })
            .unwrap_or_default();

        if installed.is_empty() {
            notes.push("Neither phpbrew nor a system PHP runtime was detected.".into());
        } else {
            notes.push(
                "System PHP is available, but phpbrew is not configured on this host.".into(),
            );
        }

        RuntimeFamilyState {
            family: "PHP".into(),
            provider: "phpbrew".into(),
            provider_status: if installed.is_empty() {
                "missing".into()
            } else {
                "fallback-system".into()
            },
            detected_binary: php_path.or(composer_path),
            health: if installed.is_empty() {
                "missing".into()
            } else {
                "attention".into()
            },
            package_tools,
            mirrors,
            recommended_versions,
            capabilities: runtime_capabilities(false, false, false),
            notes,
            installed,
        }
    }
}

impl RuntimeProvider for RubyProvider {
    fn detect(&self) -> RuntimeFamilyState {
        self.detect_for_host("native")
    }
}

impl HostAwareRuntimeProvider for RubyProvider {
    fn detect_for_host(&self, host_id: &str) -> RuntimeFamilyState {
        let rbenv_path = command_path_for_host(host_id, "rbenv");
        let ruby_path = command_path_for_host(host_id, "ruby");
        let bundler_path = command_path_for_host(host_id, "bundle");
        let package_tools = vec!["gem".into(), "bundler".into()];
        let mirrors = vec!["rubygems.org".into(), "Ruby China".into()];
        let recommended_versions = vec!["2.7".into(), "3.1".into(), "3.2".into()];
        let mut notes = vec![
            "Canonical provider target is rbenv, with gems and Bundler layered above the selected Ruby runtime."
                .into(),
        ];

        if bundler_path.is_none() {
            notes.push("Bundler was not detected on this host.".into());
        }

        if let Some(path) = rbenv_path {
            let can_install = rbenv_supports_install_for_host(host_id);
            let can_remove = rbenv_supports_uninstall_for_host(host_id);
            let active_name = hosts::run_capture_on_host(host_id, "rbenv", &["version-name"]);
            let installed = hosts::run_capture_on_host(host_id, "rbenv", &["versions", "--bare"])
                .map(|value| {
                    value
                        .lines()
                        .map(str::trim)
                        .filter(|line| !line.is_empty())
                        .map(|line| RuntimeInstallation {
                            version: line.to_string(),
                            channel: "managed".into(),
                            active: active_name
                                .as_ref()
                                .map(|active| active.split_whitespace().next() == Some(line))
                                .unwrap_or(false),
                            source: "rbenv".into(),
                            tools: package_tools.clone(),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            if installed.is_empty() {
                notes.push("rbenv is present, but no managed Ruby versions were detected.".into());
            }
            if can_install {
                notes.push(
                    "Forge Env can install and activate rbenv-managed Ruby versions on this host because ruby-build is available."
                        .into(),
                );
            } else {
                notes.push(
                    "Forge Env can activate existing rbenv-managed Ruby versions on this host, but Ruby install still needs the ruby-build plugin."
                        .into(),
                );
            }
            if can_remove {
                notes.push(
                    "Forge Env can also remove inactive rbenv-managed Ruby versions on this host."
                        .into(),
                );
            }

            return RuntimeFamilyState {
                family: "Ruby".into(),
                provider: "rbenv".into(),
                provider_status: if can_install {
                    "ready".into()
                } else {
                    "inspect-only".into()
                },
                detected_binary: Some(path),
                health: if installed.is_empty() {
                    "attention".into()
                } else {
                    "good".into()
                },
                package_tools,
                mirrors,
                recommended_versions,
                capabilities: runtime_capabilities(can_install, true, can_remove),
                notes,
                installed,
            };
        }

        let installed = ruby_path
            .as_ref()
            .and_then(|_| hosts::run_capture_on_host(host_id, "ruby", &["--version"]))
            .map(|version| {
                vec![RuntimeInstallation {
                    version: normalize_ruby_version(&version),
                    channel: "system".into(),
                    active: true,
                    source: "system".into(),
                    tools: package_tools.clone(),
                }]
            })
            .unwrap_or_default();

        if installed.is_empty() {
            notes.push("Neither rbenv nor a system Ruby runtime was detected.".into());
        } else {
            notes
                .push("System Ruby is available, but rbenv is not configured on this host.".into());
        }

        RuntimeFamilyState {
            family: "Ruby".into(),
            provider: "rbenv".into(),
            provider_status: if installed.is_empty() {
                "missing".into()
            } else {
                "fallback-system".into()
            },
            detected_binary: ruby_path.or(bundler_path),
            health: if installed.is_empty() {
                "missing".into()
            } else {
                "attention".into()
            },
            package_tools,
            mirrors,
            recommended_versions,
            capabilities: runtime_capabilities(false, false, false),
            notes,
            installed,
        }
    }
}

impl RuntimeProvider for CppProvider {
    fn detect(&self) -> RuntimeFamilyState {
        self.detect_for_host("native")
    }
}

impl HostAwareRuntimeProvider for CppProvider {
    fn detect_for_host(&self, host_id: &str) -> RuntimeFamilyState {
        let clang_path = command_path_for_host(host_id, "clang");
        let gcc_path = command_path_for_host(host_id, "gcc");
        let cl_path = command_path_for_host(host_id, "cl");
        let cmake_path = command_path_for_host(host_id, "cmake");
        let ninja_path = command_path_for_host(host_id, "ninja");
        let make_path = command_path_for_host(host_id, "make");
        let package_tools = vec!["CMake".into(), "Make".into(), "Ninja".into()];
        let mirrors = vec!["system package manager".into()];
        let recommended_versions = vec!["Clang".into(), "GCC".into(), "MSVC".into()];
        let can_install = deps::supports_cpp_toolchain_install_for_host(host_id);
        let mut notes = vec![
            "C/C++ is modeled as a host toolchain family rather than a version-managed runtime."
                .into(),
        ];
        if can_install {
            notes.push(
                "Forge Env can install compiler and build-tool templates for Clang or GCC through the host package manager."
                    .into(),
            );
        } else {
            notes.push(
                "Forge Env currently inspects compiler and build tool availability, but does not mutate C/C++ toolchains on this host."
                    .into(),
            );
        }

        if cmake_path.is_none() {
            notes.push("CMake was not detected on this host.".into());
        }
        if ninja_path.is_none() && make_path.is_none() {
            notes.push("Neither Ninja nor Make was detected on this host.".into());
        }

        let mut installed = Vec::new();

        if let Some(version) = compiler_version(host_id, "clang", &["--version"]) {
            installed.push(RuntimeInstallation {
                version,
                channel: "system".into(),
                active: true,
                source: "clang".into(),
                tools: package_tools.clone(),
            });
        }

        if let Some(version) = compiler_version(host_id, "gcc", &["--version"]) {
            installed.push(RuntimeInstallation {
                version,
                channel: "system".into(),
                active: installed.is_empty(),
                source: "gcc".into(),
                tools: package_tools.clone(),
            });
        }

        if let Some(version) = compiler_version(host_id, "cl", &[]) {
            installed.push(RuntimeInstallation {
                version,
                channel: "system".into(),
                active: installed.is_empty(),
                source: "MSVC".into(),
                tools: package_tools.clone(),
            });
        }

        if installed.is_empty() {
            notes.push("No supported C/C++ compiler toolchain was detected on this host.".into());
        }

        RuntimeFamilyState {
            family: "C/C++".into(),
            provider: "system toolchain".into(),
            provider_status: if installed.is_empty() {
                "missing".into()
            } else if can_install {
                "ready".into()
            } else {
                "inspect-only".into()
            },
            detected_binary: clang_path.or(gcc_path).or(cl_path).or(cmake_path),
            health: if installed.is_empty() {
                "missing".into()
            } else {
                "attention".into()
            },
            package_tools,
            mirrors,
            recommended_versions,
            capabilities: runtime_capabilities(can_install, false, false),
            notes,
            installed,
        }
    }
}

impl HostAwareRuntimeProvider for DotNetProvider {
    fn detect_for_host(&self, host_id: &str) -> RuntimeFamilyState {
        let dotnet_path = command_path_for_host(host_id, "dotnet");
        let can_install = dotnet_supports_install_for_host(host_id);
        let notes = vec![
            "Canonical provider target is the .NET SDK toolchain with global.json controlling project-local SDK resolution."
                .into(),
        ];
        let recommended_versions = vec!["6.0 LTS".into(), "8.0 LTS".into(), "9.0".into()];
        let package_tools = vec![
            "NuGet".into(),
            "dotnet restore".into(),
            "dotnet tool".into(),
        ];
        let mirrors = vec![
            "nuget.org".into(),
            "Azure China NuGet".into(),
            "Tencent NuGet".into(),
        ];

        let Some(path) = dotnet_path else {
            return RuntimeFamilyState {
                family: ".NET".into(),
                provider: "dotnet SDK".into(),
                provider_status: "missing".into(),
                detected_binary: None,
                health: "missing".into(),
                package_tools,
                mirrors,
                recommended_versions,
                capabilities: runtime_capabilities(can_install, false, false),
                notes: {
                    let mut notes = notes;
                    notes.push("No dotnet SDK was detected on this host.".into());
                    if can_install {
                        notes.push(
                            "Forge Env can install .NET SDKs through the official dotnet-install script on this host."
                                .into(),
                        );
                    } else {
                        notes.push(
                            "Managed .NET installation is not available on this host through Forge Env yet."
                                .into(),
                        );
                    }
                    notes
                },
                installed: Vec::new(),
            };
        };

        let active_sdk = hosts::run_capture_on_host(host_id, "dotnet", &["--version"])
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let mut installed = hosts::run_capture_on_host(host_id, "dotnet", &["--list-sdks"])
            .map(|value| parse_dotnet_sdks(&value, active_sdk.as_deref(), &package_tools))
            .unwrap_or_default();

        if installed.is_empty() {
            if let Some(version) = active_sdk.clone() {
                installed.push(RuntimeInstallation {
                    version,
                    channel: "system".into(),
                    active: true,
                    source: "dotnet".into(),
                    tools: package_tools.clone(),
                });
            }
        }

        let mut notes = notes;
        if installed.is_empty() {
            notes.push(
                "dotnet exists on PATH, but Forge Env could not enumerate installed SDKs.".into(),
            );
        } else {
            notes.push("Use global.json to pin a project SDK. Forge Env can install additional SDKs, but global activation remains project-scoped.".into());
        }

        RuntimeFamilyState {
            family: ".NET".into(),
            provider: "dotnet SDK".into(),
            provider_status: "fallback-system".into(),
            detected_binary: Some(path),
            health: if installed.is_empty() {
                "attention".into()
            } else {
                "attention".into()
            },
            package_tools,
            mirrors,
            recommended_versions,
            capabilities: runtime_capabilities(can_install, false, false),
            notes,
            installed,
        }
    }
}

fn command_path_for_host(host_id: &str, command: &str) -> Option<String> {
    if host_id.starts_with("wsl:") {
        hosts::run_shell_script_on_host(
            host_id,
            &format!("command -v {command} 2>/dev/null || true"),
        )
        .ok()
        .and_then(|result| {
            (!result.stdout.trim().is_empty()).then_some(result.stdout.trim().to_string())
        })
    } else {
        shell::command_path(command)
    }
}

fn dotnet_supports_install_for_host(host_id: &str) -> bool {
    (host_id.starts_with("wsl:") || !cfg!(target_os = "windows"))
        && command_path_for_host(host_id, "curl").is_some()
}

fn normalize_java_version(version: &str) -> String {
    version
        .lines()
        .next()
        .unwrap_or(version)
        .replace("openjdk version ", "")
        .replace("java version ", "")
        .replace("javac ", "")
        .replace('"', "")
        .split_whitespace()
        .next()
        .unwrap_or("system")
        .to_string()
}

fn normalize_go_version(version: &str) -> String {
    version
        .split_whitespace()
        .find(|token| token.starts_with("go") && token.len() > 2)
        .map(|token| token.trim_start_matches("go").to_string())
        .unwrap_or_else(|| "system".into())
}

fn normalize_php_version(version: &str) -> String {
    version
        .lines()
        .next()
        .unwrap_or(version)
        .split_whitespace()
        .nth(1)
        .unwrap_or("system")
        .to_string()
}

fn parse_phpbrew_versions_output(
    output: &str,
    package_tools: &[String],
) -> Vec<RuntimeInstallation> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let active = line.starts_with('*');
            let version = line
                .trim_start_matches('*')
                .trim()
                .split_whitespace()
                .next()?
                .trim_start_matches("php-")
                .to_string();
            (!version.is_empty()).then_some(RuntimeInstallation {
                version,
                channel: "managed".into(),
                active,
                source: "phpbrew".into(),
                tools: package_tools.to_vec(),
            })
        })
        .collect()
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

fn phpbrew_supports_remove_for_host(host_id: &str) -> bool {
    hosts::run_capture_on_host(host_id, "phpbrew", &["help"])
        .map(|value| phpbrew_supports_remove_output(&value))
        .unwrap_or(false)
}

fn normalize_ruby_version(version: &str) -> String {
    version
        .split_whitespace()
        .nth(1)
        .unwrap_or("system")
        .to_string()
}

fn rbenv_supports_install_output(output: &str) -> bool {
    output.lines().map(str::trim).any(|line| line == "install")
}

fn rbenv_supports_install_for_host(host_id: &str) -> bool {
    hosts::run_capture_on_host(host_id, "rbenv", &["commands"])
        .map(|value| rbenv_supports_install_output(&value))
        .unwrap_or(false)
}

fn rbenv_supports_uninstall_output(output: &str) -> bool {
    output
        .lines()
        .map(str::trim)
        .any(|line| line == "uninstall")
}

fn rbenv_supports_uninstall_for_host(host_id: &str) -> bool {
    hosts::run_capture_on_host(host_id, "rbenv", &["commands"])
        .map(|value| rbenv_supports_uninstall_output(&value))
        .unwrap_or(false)
}

fn compiler_version(host_id: &str, command: &str, args: &[&str]) -> Option<String> {
    hosts::run_capture_on_host(host_id, command, args).map(|output| {
        output
            .lines()
            .next()
            .unwrap_or(output.as_str())
            .trim()
            .to_string()
    })
}

fn parse_gvm_versions(output: &str, package_tools: &[String]) -> Vec<RuntimeInstallation> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && line.contains("go"))
        .filter_map(|line| {
            let active = line.starts_with("=>");
            let normalized = line
                .trim_start_matches("=>")
                .trim_start_matches('*')
                .trim()
                .split_whitespace()
                .find(|token| token.starts_with("go"))
                .map(|token| token.trim_start_matches("go").to_string())?;
            Some(RuntimeInstallation {
                version: normalized,
                channel: "managed".into(),
                active,
                source: "gvm".into(),
                tools: package_tools.to_vec(),
            })
        })
        .collect()
}

fn parse_dotnet_sdks(
    output: &str,
    active_sdk: Option<&str>,
    package_tools: &[String],
) -> Vec<RuntimeInstallation> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let version = line.split_whitespace().next()?.trim().to_string();
            Some(RuntimeInstallation {
                version: version.clone(),
                channel: "system".into(),
                active: active_sdk == Some(version.as_str()),
                source: "dotnet".into(),
                tools: package_tools.to_vec(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_go_version, normalize_java_version, normalize_php_version,
        normalize_ruby_version, parse_dotnet_sdks, parse_phpbrew_versions_output,
        phpbrew_supports_remove_output, rbenv_supports_install_output,
        rbenv_supports_uninstall_output,
    };

    #[test]
    fn normalizes_java_versions() {
        assert_eq!(
            normalize_java_version("openjdk version \"21.0.3\" 2024-04-16"),
            "21.0.3"
        );
        assert_eq!(normalize_java_version("javac 17.0.12"), "17.0.12");
    }

    #[test]
    fn normalizes_go_versions() {
        assert_eq!(
            normalize_go_version("go version go1.22.5 darwin/arm64"),
            "1.22.5"
        );
    }

    #[test]
    fn parses_dotnet_sdks() {
        let tools = vec!["NuGet".into()];
        let parsed = parse_dotnet_sdks(
            "8.0.303 [/usr/local/share/dotnet/sdk]\n9.0.100 [/usr/local/share/dotnet/sdk]",
            Some("8.0.303"),
            &tools,
        );
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].version, "8.0.303");
        assert!(parsed[0].active);
        assert_eq!(parsed[1].version, "9.0.100");
        assert!(!parsed[1].active);
    }

    #[test]
    fn normalizes_php_versions() {
        assert_eq!(
            normalize_php_version("PHP 8.3.12 (cli) (built: Aug  1 2024)"),
            "8.3.12"
        );
    }

    #[test]
    fn parses_phpbrew_versions() {
        let tools = vec!["composer".into()];
        let parsed = parse_phpbrew_versions_output("* php-8.2.24\n  php-8.3.12", &tools);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].version, "8.2.24");
        assert!(parsed[0].active);
        assert_eq!(parsed[1].version, "8.3.12");
        assert!(!parsed[1].active);
    }

    #[test]
    fn parses_phpbrew_remove_support() {
        assert!(phpbrew_supports_remove_output(
            "install\nlist\nremove\nswitch\nuse\n"
        ));
        assert!(!phpbrew_supports_remove_output(
            "install\nlist\nswitch\nuse\n"
        ));
    }

    #[test]
    fn normalizes_ruby_versions() {
        assert_eq!(
            normalize_ruby_version(
                "ruby 3.2.2p53 (2023-03-30 revision e51014f9c0) [arm64-darwin22]"
            ),
            "3.2.2p53"
        );
    }

    #[test]
    fn parses_rbenv_install_support() {
        assert!(rbenv_supports_install_output(
            "global\ninstall\nlocal\nversions\n"
        ));
        assert!(!rbenv_supports_install_output("global\nlocal\nversions\n"));
    }

    #[test]
    fn parses_rbenv_uninstall_support() {
        assert!(rbenv_supports_uninstall_output(
            "global\ninstall\nlocal\nuninstall\nversions\n"
        ));
        assert!(!rbenv_supports_uninstall_output(
            "global\ninstall\nlocal\nversions\n"
        ));
    }
}
