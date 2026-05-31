//! Integration tests for the provider system.
//!
//! These tests exercise the public detection API (`detect_runtimes` and
//! `detect_runtimes_for_host`) together with the data structures returned by
//! each provider.
//!
//! # Mock-environment tests
//!
//! Several tests create temporary directories with mock executables and
//! manipulate the process-wide `PATH` and `HOME` environment variables to
//! simulate different runtime environments.  Because env-var changes are
//! process-global, **those tests must run serially**:
//!
//! ```sh
//! cargo test --test provider_integration -- --test-threads=1
//! ```
//!
//! Tests that do NOT manipulate environment variables are safe to run in
//! parallel and will execute by default.

use forge_env_lib::core::models::{
    RuntimeCapabilities, RuntimeFamilyState, RuntimeInstallation,
};
use forge_env_lib::core::provider::{detect_runtimes, detect_runtimes_for_host};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Process-wide lock used by mock-environment tests to serialize access to
/// `PATH` / `HOME`.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// RAII guard that saves and restores `PATH` and `HOME` across a test scope.
struct EnvGuard {
    original_path: Option<String>,
    original_home: Option<String>,
}

impl EnvGuard {
    /// Snapshot the current `PATH` and `HOME`, then set them to the supplied
    /// values.  Call `.abort()` or let the instance drop to restore originals.
    fn new(path: &str, home: &str) -> Self {
        let original_path = std::env::var("PATH").ok();
        let original_home = std::env::var("HOME").ok();

        // SAFETY: we hold ENV_LOCK so no other mock-env test can race.
        unsafe {
            std::env::set_var("PATH", path);
            std::env::set_var("HOME", home);
        }

        Self {
            original_path,
            original_home,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.original_path {
                Some(v) => std::env::set_var("PATH", v),
                None => std::env::remove_var("PATH"),
            }
            match &self.original_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
    }
}

/// Create a unique temporary directory under the system temp root.
fn create_temp_dir(label: &str) -> PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("forge_test_{label}_{ts}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Write a file, creating parent directories as needed.
fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

/// Create an executable shell script at `path` that runs the supplied body.
/// The script will be `#!/bin/bash` followed by the body.
fn create_mock_bin(path: &Path, body: &str) {
    write_file(path, &format!("#!/bin/bash\n{body}\n"));
    make_executable(path);
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let perm = fs::Permissions::from_mode(0o755);
    fs::set_permissions(path, perm).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {
    // On non-Unix platforms, file permissions are not set this way.
    // The tests that depend on executable permissions are #[cfg(unix)] only.
}

/// Create a directory entry (used for SDKMAN candidate dirs).
fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

/// Create a symlink.
#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) {
    if link.exists() || link.symlink_metadata().is_ok() {
        fs::remove_file(link).unwrap();
    }
    std::os::unix::fs::symlink(target, link).unwrap();
}

// ===========================================================================
// SECTION 1: Structural contract tests (parallel-safe)
// ===========================================================================

/// `detect_runtimes()` must return exactly 9 provider states.
#[test]
fn detect_runtimes_returns_nine_providers() {
    let states = detect_runtimes();
    assert_eq!(states.len(), 9, "expected 9 runtime families");
}

/// `detect_runtimes_for_host("native")` must return the same count as
/// `detect_runtimes()`.
#[test]
fn detect_runtimes_for_host_native_matches_detect_runtimes() {
    let all = detect_runtimes();
    let native = detect_runtimes_for_host("native");
    assert_eq!(all.len(), native.len());
    for (a, b) in all.iter().zip(native.iter()) {
        assert_eq!(a.family, b.family);
    }
}

/// Every provider state must have a non-empty family name.
#[test]
fn all_providers_have_nonempty_family() {
    for state in detect_runtimes() {
        assert!(
            !state.family.is_empty(),
            "family must not be empty for a provider state"
        );
    }
}

/// Every provider state must have a non-empty provider name.
#[test]
fn all_providers_have_nonempty_provider_name() {
    for state in detect_runtimes() {
        assert!(
            !state.provider.is_empty(),
            "provider name must not be empty for family={}",
            state.family
        );
    }
}

/// The expected families must appear (in any order) in detection results.
#[test]
fn expected_families_are_present() {
    let states = detect_runtimes();
    let families: Vec<&str> = states.iter().map(|s| s.family.as_str()).collect();
    let expected = [
        "Python",
        "Node.js",
        "Rust",
        "Java",
        "Go",
        ".NET",
        "PHP",
        "Ruby",
        "C/C++",
    ];
    for name in &expected {
        assert!(
            families.contains(name),
            "missing expected family: {name}\n  found: {families:?}"
        );
    }
}

/// provider_status must be one of the recognized values.
#[test]
fn provider_status_values_are_valid() {
    let valid = ["ready", "missing", "fallback-system", "inspect-only"];
    for state in detect_runtimes() {
        assert!(
            valid.contains(&state.provider_status.as_str()),
            "invalid provider_status '{}' for family={}",
            state.provider_status,
            state.family
        );
    }
}

/// health must be one of the recognized values.
#[test]
fn health_values_are_valid() {
    let valid = ["good", "attention", "missing"];
    for state in detect_runtimes() {
        assert!(
            valid.contains(&state.health.as_str()),
            "invalid health '{}' for family={}",
            state.health,
            state.family
        );
    }
}

/// If provider_status is "missing", health must also be "missing".
#[test]
fn missing_provider_implies_missing_health() {
    for state in detect_runtimes() {
        if state.provider_status == "missing" {
            assert_eq!(
                state.health, "missing",
                "provider_status=missing but health={} for family={}",
                state.health, state.family
            );
        }
    }
}

/// If provider_status is "ready", health must not be "missing".
#[test]
fn ready_provider_implies_non_missing_health() {
    for state in detect_runtimes() {
        if state.provider_status == "ready" {
            assert_ne!(
                state.health, "missing",
                "ready provider should not have missing health for family={}",
                state.family
            );
        }
    }
}

/// Every provider must have at least one recommended version.
#[test]
fn all_providers_have_recommended_versions() {
    for state in detect_runtimes() {
        assert!(
            !state.recommended_versions.is_empty(),
            "family={} must have at least one recommended version",
            state.family
        );
    }
}

/// Every provider must have at least one package tool.
#[test]
fn all_providers_have_package_tools() {
    for state in detect_runtimes() {
        assert!(
            !state.package_tools.is_empty(),
            "family={} must have at least one package tool",
            state.family
        );
    }
}

/// Every provider must have at least one mirror.
#[test]
fn all_providers_have_mirrors() {
    for state in detect_runtimes() {
        assert!(
            !state.mirrors.is_empty(),
            "family={} must have at least one mirror",
            state.family
        );
    }
}

/// Installed versions must have non-empty version strings.
#[test]
fn installed_versions_are_nonempty() {
    for state in detect_runtimes() {
        for inst in &state.installed {
            assert!(
                !inst.version.is_empty(),
                "installed version must not be empty for family={}, source={}",
                state.family,
                inst.source
            );
        }
    }
}

/// Installed versions must have non-empty channel strings.
#[test]
fn installed_versions_have_nonempty_channel() {
    for state in detect_runtimes() {
        for inst in &state.installed {
            assert!(
                !inst.channel.is_empty(),
                "installed channel must not be empty for family={}, version={}",
                state.family,
                inst.version
            );
        }
    }
}

/// Installed versions must have non-empty source strings.
#[test]
fn installed_versions_have_nonempty_source() {
    for state in detect_runtimes() {
        for inst in &state.installed {
            assert!(
                !inst.source.is_empty(),
                "installed source must not be empty for family={}, version={}",
                state.family,
                inst.version
            );
        }
    }
}

/// Notes must not contain empty strings.
#[test]
fn notes_do_not_contain_empty_strings() {
    for state in detect_runtimes() {
        for (i, note) in state.notes.iter().enumerate() {
            assert!(
                !note.is_empty(),
                "note[{i}] is empty for family={}",
                state.family
            );
        }
    }
}

// ===========================================================================
// SECTION 2: Capability invariant tests (parallel-safe)
// ===========================================================================

/// If provider_status is "missing", can_activate and can_remove should be false.
/// Note: some providers (like .NET) allow can_install even when "missing" because
/// the SDK can be installed independently.
#[test]
fn missing_provider_has_no_capabilities() {
    for state in detect_runtimes() {
        if state.provider_status == "missing" {
            assert!(
                !state.capabilities.can_activate,
                "missing provider family={} should not can_activate",
                state.family
            );
            assert!(
                !state.capabilities.can_remove,
                "missing provider family={} should not can_remove",
                state.family
            );
        }
    }
}

/// If provider_status is "ready", can_activate must be true for version-managed
/// runtimes (Python, Node, Rust, Java, Go, PHP, Ruby).  C/C++ is a toolchain
/// family so can_activate is legitimately false.
#[test]
fn ready_providers_can_activate() {
    let managed_families = ["Python", "Node.js", "Rust", "Java", "Go", "PHP", "Ruby"];
    for state in detect_runtimes() {
        if state.provider_status == "ready" && managed_families.contains(&state.family.as_str())
        {
            assert!(
                state.capabilities.can_activate,
                "ready managed provider family={} must can_activate",
                state.family
            );
        }
    }
}

/// If provider_status is "ready", can_install must be true for version-managed
/// runtimes (except C/C++ which delegates to the system package manager).
#[test]
fn ready_providers_can_install() {
    let managed_families = ["Python", "Node.js", "Rust", "Java", "Go", "PHP", "Ruby"];
    for state in detect_runtimes() {
        if state.provider_status == "ready" && managed_families.contains(&state.family.as_str())
        {
            assert!(
                state.capabilities.can_install,
                "ready managed provider family={} must can_install",
                state.family
            );
        }
    }
}

/// If installed is non-empty and provider is NOT ready, health must be
/// "attention" (system fallback with something to offer).
#[test]
fn nonready_with_installations_has_attention_health() {
    for state in detect_runtimes() {
        if !state.installed.is_empty()
            && state.provider_status != "ready"
            && state.provider_status != "missing"
        {
            assert_eq!(
                state.health, "attention",
                "non-ready provider with installations should have attention health, family={}",
                state.family
            );
        }
    }
}

/// If health is "good", installed must be non-empty.
#[test]
fn good_health_implies_installed() {
    for state in detect_runtimes() {
        if state.health == "good" {
            assert!(
                !state.installed.is_empty(),
                "health=good but no installations for family={}",
                state.family
            );
        }
    }
}

/// If health is "missing", installed must be empty.
#[test]
fn missing_health_implies_no_installations() {
    for state in detect_runtimes() {
        if state.health == "missing" {
            assert!(
                state.installed.is_empty(),
                "health=missing but found {} installations for family={}",
                state.installed.len(),
                state.family
            );
        }
    }
}

/// At most one installed version should be marked active per provider.
#[test]
fn at_most_one_active_version_per_provider() {
    for state in detect_runtimes() {
        let active_count = state.installed.iter().filter(|i| i.active).count();
        assert!(
            active_count <= 1,
            "family={} has {active_count} active versions (expected at most 1)",
            state.family
        );
    }
}

// ===========================================================================
// SECTION 3: Serde round-trip tests (parallel-safe)
// ===========================================================================

/// RuntimeFamilyState survives a JSON serialize-deserialize round-trip.
#[test]
fn runtime_family_state_serde_round_trip() {
    let state = RuntimeFamilyState {
        family: "Python".into(),
        provider: "pyenv".into(),
        provider_status: "ready".into(),
        detected_binary: Some("/usr/local/bin/pyenv".into()),
        health: "good".into(),
        package_tools: vec!["pip".into(), "poetry".into()],
        mirrors: vec!["PyPI".into()],
        recommended_versions: vec!["3.12".into()],
        capabilities: RuntimeCapabilities {
            can_install: true,
            can_activate: true,
            can_remove: true,
        },
        notes: vec!["test note".into()],
        installed: vec![RuntimeInstallation {
            version: "3.12.4".into(),
            channel: "managed".into(),
            active: true,
            source: "pyenv".into(),
            tools: vec!["pip".into()],
        }],
    };

    let json = serde_json::to_string(&state).unwrap();
    let deserialized: RuntimeFamilyState = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.family, "Python");
    assert_eq!(deserialized.provider, "pyenv");
    assert_eq!(deserialized.provider_status, "ready");
    assert_eq!(
        deserialized.detected_binary.as_deref(),
        Some("/usr/local/bin/pyenv")
    );
    assert_eq!(deserialized.health, "good");
    assert_eq!(deserialized.package_tools, vec!["pip", "poetry"]);
    assert_eq!(deserialized.capabilities.can_install, true);
    assert_eq!(deserialized.capabilities.can_activate, true);
    assert_eq!(deserialized.capabilities.can_remove, true);
    assert_eq!(deserialized.installed.len(), 1);
    assert_eq!(deserialized.installed[0].version, "3.12.4");
    assert!(deserialized.installed[0].active);
}

/// RuntimeFamilyState with no detected_binary (missing provider).
#[test]
fn runtime_family_state_missing_provider_serde() {
    let state = RuntimeFamilyState {
        family: "Python".into(),
        provider: "pyenv".into(),
        provider_status: "missing".into(),
        detected_binary: None,
        health: "missing".into(),
        package_tools: vec!["pip".into()],
        mirrors: vec![],
        recommended_versions: vec!["3.12".into()],
        capabilities: RuntimeCapabilities {
            can_install: false,
            can_activate: false,
            can_remove: false,
        },
        notes: vec![],
        installed: vec![],
    };

    let json = serde_json::to_string(&state).unwrap();
    assert!(
        json.contains("null"),
        "missing detected_binary should serialize as null"
    );

    let deserialized: RuntimeFamilyState = serde_json::from_str(&json).unwrap();
    assert!(deserialized.detected_binary.is_none());
    assert!(deserialized.installed.is_empty());
}

/// RuntimeFamilyState with multiple installations.
#[test]
fn runtime_family_state_multiple_installations_serde() {
    let state = RuntimeFamilyState {
        family: "Node.js".into(),
        provider: "Volta".into(),
        provider_status: "ready".into(),
        detected_binary: Some("/usr/local/bin/volta".into()),
        health: "good".into(),
        package_tools: vec!["npm".into(), "yarn".into(), "pnpm".into()],
        mirrors: vec!["npm".into()],
        recommended_versions: vec!["18 LTS".into(), "20 LTS".into()],
        capabilities: RuntimeCapabilities {
            can_install: true,
            can_activate: true,
            can_remove: true,
        },
        notes: vec![],
        installed: vec![
            RuntimeInstallation {
                version: "18.19.0".into(),
                channel: "managed".into(),
                active: false,
                source: "Volta".into(),
                tools: vec!["npm".into()],
            },
            RuntimeInstallation {
                version: "20.11.0".into(),
                channel: "managed".into(),
                active: true,
                source: "Volta".into(),
                tools: vec!["npm".into(), "yarn".into()],
            },
            RuntimeInstallation {
                version: "22.0.0".into(),
                channel: "managed".into(),
                active: false,
                source: "Volta".into(),
                tools: vec!["npm".into()],
            },
        ],
    };

    let json = serde_json::to_string(&state).unwrap();
    let deserialized: RuntimeFamilyState = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.installed.len(), 3);
    assert!(!deserialized.installed[0].active);
    assert!(deserialized.installed[1].active);
    assert!(!deserialized.installed[2].active);
}

/// RuntimeCapabilities serde round-trip.
#[test]
fn runtime_capabilities_serde_round_trip() {
    let caps = RuntimeCapabilities {
        can_install: true,
        can_activate: false,
        can_remove: true,
    };
    let json = serde_json::to_string(&caps).unwrap();
    assert!(json.contains("\"canInstall\""));
    assert!(json.contains("\"canActivate\""));
    assert!(json.contains("\"canRemove\""));

    let deserialized: RuntimeCapabilities = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.can_install, true);
    assert_eq!(deserialized.can_activate, false);
    assert_eq!(deserialized.can_remove, true);
}

/// RuntimeInstallation serde round-trip with all fields populated.
#[test]
fn runtime_installation_serde_round_trip() {
    let inst = RuntimeInstallation {
        version: "3.12.4".into(),
        channel: "stable".into(),
        active: true,
        source: "pyenv".into(),
        tools: vec!["pip".into(), "poetry".into()],
    };
    let json = serde_json::to_string(&inst).unwrap();
    let deserialized: RuntimeInstallation = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.version, "3.12.4");
    assert_eq!(deserialized.channel, "stable");
    assert!(deserialized.active);
    assert_eq!(deserialized.source, "pyenv");
    assert_eq!(deserialized.tools, vec!["pip", "poetry"]);
}

/// JSON from a real detection run can be deserialized.
#[test]
fn real_detection_output_is_deserializable() {
    let states = detect_runtimes();
    let json = serde_json::to_string(&states).unwrap();
    let deserialized: Vec<RuntimeFamilyState> = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.len(), states.len());
    for (orig, deser) in states.iter().zip(deserialized.iter()) {
        assert_eq!(orig.family, deser.family);
        assert_eq!(orig.provider, deser.provider);
    }
}

/// Empty Vec<RuntimeFamilyState> serializes and deserializes correctly.
#[test]
fn empty_runtime_family_states_serde() {
    let empty: Vec<RuntimeFamilyState> = vec![];
    let json = serde_json::to_string(&empty).unwrap();
    assert_eq!(json, "[]");
    let deserialized: Vec<RuntimeFamilyState> = serde_json::from_str(&json).unwrap();
    assert!(deserialized.is_empty());
}

/// RuntimeInstallation with empty tools list.
#[test]
fn runtime_installation_empty_tools_serde() {
    let inst = RuntimeInstallation {
        version: "1.0.0".into(),
        channel: "system".into(),
        active: true,
        source: "system".into(),
        tools: vec![],
    };
    let json = serde_json::to_string(&inst).unwrap();
    let deserialized: RuntimeInstallation = serde_json::from_str(&json).unwrap();
    assert!(deserialized.tools.is_empty());
}

// ===========================================================================
// SECTION 4: Data construction and field semantics (parallel-safe)
// ===========================================================================

/// RuntimeCapabilities: all-false constructor.
#[test]
fn capabilities_all_false() {
    let caps = RuntimeCapabilities {
        can_install: false,
        can_activate: false,
        can_remove: false,
    };
    assert!(!caps.can_install);
    assert!(!caps.can_activate);
    assert!(!caps.can_remove);
}

/// RuntimeCapabilities: all-true constructor.
#[test]
fn capabilities_all_true() {
    let caps = RuntimeCapabilities {
        can_install: true,
        can_activate: true,
        can_remove: true,
    };
    assert!(caps.can_install);
    assert!(caps.can_activate);
    assert!(caps.can_remove);
}

/// RuntimeInstallation: active field semantics.
#[test]
fn installation_active_semantics() {
    let active_inst = RuntimeInstallation {
        version: "3.12".into(),
        channel: "managed".into(),
        active: true,
        source: "pyenv".into(),
        tools: vec![],
    };
    let inactive_inst = RuntimeInstallation {
        version: "3.11".into(),
        channel: "managed".into(),
        active: false,
        source: "pyenv".into(),
        tools: vec![],
    };
    assert!(active_inst.active);
    assert!(!inactive_inst.active);
}

/// RuntimeFamilyState: provider_status / health consistency matrix.
#[test]
fn provider_status_health_consistency() {
    let states = detect_runtimes();
    for state in &states {
        // Rule 1: "missing" status => "missing" health
        if state.provider_status == "missing" {
            assert_eq!(state.health, "missing",
                "family={}: missing status should imply missing health", state.family);
        }
        // Rule 2: "ready" status => health is "good" or "attention" (not "missing")
        if state.provider_status == "ready" {
            assert!(state.health == "good" || state.health == "attention",
                "family={}: ready status should have good or attention health, got {}",
                state.family, state.health);
        }
        // Rule 3: if health is "missing", installed must be empty
        if state.health == "missing" {
            assert!(state.installed.is_empty(),
                "family={}: missing health but found {} installations",
                state.family, state.installed.len());
        }
        // Rule 4: if health is "good", installed must be non-empty
        if state.health == "good" {
            assert!(!state.installed.is_empty(),
                "family={}: good health but no installations", state.family);
        }
    }
}

/// Each installed version has a consistent source string.
#[test]
fn installed_source_matches_provider() {
    let states = detect_runtimes();
    for state in &states {
        for inst in &state.installed {
            // The source must be either the provider name, "system", or a known
            // variant (e.g. "Volta" for Node.js with Volta).
            let known_sources = [
                state.provider.as_str(),
                "system",
                "pyenv",
                "Volta",
                "rustup",
                "SDKMAN!",
                "gvm",
                "phpbrew",
                "rbenv",
                "dotnet",
                "clang",
                "gcc",
                "MSVC",
            ];
            assert!(
                known_sources.contains(&inst.source.as_str()),
                "family={}: unexpected source '{}' for version '{}'",
                state.family,
                inst.source,
                inst.version
            );
        }
    }
}

// ===========================================================================
// SECTION 5: Mock-environment tests
//
// These tests manipulate PATH and HOME to simulate different runtime
// environments.  They must run with --test-threads=1.
// ===========================================================================

/// An empty HOME and minimal PATH should cause every provider to report
/// "missing" status with no installed versions.
#[cfg(unix)]
#[test]
fn empty_home_all_providers_missing() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("empty_home");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    // Minimal PATH that has only core Unix utilities (ls, etc.) but no
    // runtime provider binaries.
    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    for state in &states {
        assert_eq!(
            state.provider_status, "missing",
            "family={} should be missing in empty HOME, got provider_status={}",
            state.family, state.provider_status
        );
        assert!(
            state.installed.is_empty(),
            "family={} should have no installations in empty HOME",
            state.family
        );
    }

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a pyenv-managed Python environment with two installed versions
/// and one active.
#[cfg(unix)]
#[test]
fn pyenv_managed_python_detection() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("pyenv_python");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    // Mock pyenv that knows about two versions.
    create_mock_bin(
        &bin_dir.join("pyenv"),
        r#"
case "$1" in
  version-name) echo "3.12.4" ;;
  versions)
    case "$2" in
      --bare) printf '3.11.8\n3.12.4\n' ;;
    esac
    ;;
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let python = states
        .iter()
        .find(|s| s.family == "Python")
        .expect("Python provider must be present");

    assert_eq!(python.provider_status, "ready");
    assert_eq!(python.health, "good");
    assert_eq!(python.detected_binary.is_some(), true);
    assert!(python.capabilities.can_install);
    assert!(python.capabilities.can_activate);
    assert!(python.capabilities.can_remove);

    assert_eq!(python.installed.len(), 2, "should have 2 managed versions");

    let v311 = python
        .installed
        .iter()
        .find(|i| i.version == "3.11.8")
        .expect("3.11.8 should be installed");
    assert!(!v311.active);
    assert_eq!(v311.channel, "managed");
    assert_eq!(v311.source, "pyenv");

    let v312 = python
        .installed
        .iter()
        .find(|i| i.version == "3.12.4")
        .expect("3.12.4 should be installed");
    assert!(v312.active);
    assert_eq!(v312.channel, "managed");
    assert_eq!(v312.source, "pyenv");

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate pyenv installed but with no managed versions.
#[cfg(unix)]
#[test]
fn pyenv_empty_no_versions() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("pyenv_empty");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    // Mock pyenv that returns empty versions list.
    create_mock_bin(
        &bin_dir.join("pyenv"),
        r#"
case "$1" in
  version-name) echo "system" ;;
  versions) ;;  # empty output
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let python = states
        .iter()
        .find(|s| s.family == "Python")
        .expect("Python provider must be present");

    assert_eq!(python.provider_status, "ready");
    assert_eq!(python.health, "attention", "pyenv present but no versions should be attention");
    assert!(python.installed.is_empty());

    // Should contain a note about no managed versions.
    let has_empty_note = python.notes.iter().any(|n| n.contains("no managed Python"));
    assert!(has_empty_note, "should have note about no managed versions");

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a system-only Python (no pyenv, but python3 is on PATH).
#[cfg(unix)]
#[test]
fn system_only_python_fallback() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("system_python");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    // No pyenv, but python3 exists.
    create_mock_bin(&bin_dir.join("python3"), "echo 'Python 3.11.7'");

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let python = states
        .iter()
        .find(|s| s.family == "Python")
        .expect("Python provider must be present");

    assert_eq!(python.provider_status, "fallback-system");
    assert_eq!(python.health, "attention");
    assert_eq!(python.installed.len(), 1);
    assert_eq!(python.installed[0].version, "3.11.7");
    assert_eq!(python.installed[0].channel, "system");
    assert_eq!(python.installed[0].source, "system");
    assert!(python.installed[0].active);

    // System fallback should not have install/activate/remove capabilities.
    assert!(!python.capabilities.can_install);
    assert!(!python.capabilities.can_activate);
    assert!(!python.capabilities.can_remove);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a Volta-managed Node.js environment.
#[cfg(unix)]
#[test]
fn volta_managed_node_detection() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("volta_node");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    // Mock volta (just needs to exist and be executable).
    create_mock_bin(&bin_dir.join("volta"), "exit 0");

    // Mock node that reports its version.
    create_mock_bin(&bin_dir.join("node"), "echo 'v20.11.0'");

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let node = states
        .iter()
        .find(|s| s.family == "Node.js")
        .expect("Node.js provider must be present");

    assert_eq!(node.provider_status, "ready");
    assert_eq!(node.health, "good");
    assert_eq!(node.provider, "Volta");
    assert!(node.capabilities.can_install);
    assert!(node.capabilities.can_activate);
    assert!(node.capabilities.can_remove);

    assert_eq!(node.installed.len(), 1);
    assert_eq!(node.installed[0].version, "20.11.0", "v-prefix should be stripped");
    assert_eq!(node.installed[0].channel, "managed");
    assert_eq!(node.installed[0].source, "Volta");
    assert!(node.installed[0].active);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate Volta installed but node returns no version (edge case).
#[cfg(unix)]
#[test]
fn volta_node_no_version_output() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("volta_no_ver");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(&bin_dir.join("volta"), "exit 0");
    // node exists but produces no output for --version.
    create_mock_bin(&bin_dir.join("node"), "exit 0");

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let node = states
        .iter()
        .find(|s| s.family == "Node.js")
        .expect("Node.js provider must be present");

    assert_eq!(node.provider_status, "ready");
    // When node --version produces no output, the version falls back to "managed".
    assert_eq!(node.installed.len(), 1);
    assert_eq!(node.installed[0].version, "managed");

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a system-only Node.js (no volta, but node is on PATH).
#[cfg(unix)]
#[test]
fn system_only_node_fallback() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("system_node");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(&bin_dir.join("node"), "echo 'v18.19.0'");

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let node = states
        .iter()
        .find(|s| s.family == "Node.js")
        .expect("Node.js provider must be present");

    assert_eq!(node.provider_status, "fallback-system");
    assert_eq!(node.health, "attention");
    assert_eq!(node.installed.len(), 1);
    assert_eq!(node.installed[0].version, "18.19.0");
    assert_eq!(node.installed[0].channel, "system");
    assert!(!node.capabilities.can_install);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a rustup-managed Rust environment with two toolchains.
#[cfg(unix)]
#[test]
fn rustup_managed_rust_detection() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("rustup_rust");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("rustup"),
        r#"
case "$1" in
  show)
    case "$2" in
      active-toolchain) echo "stable-aarch64-apple-darwin (default)" ;;
    esac
    ;;
  toolchain)
    case "$2" in
      list) printf 'stable-aarch64-apple-darwin (default)\nnightly-2024-01-15\n' ;;
    esac
    ;;
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let rust = states
        .iter()
        .find(|s| s.family == "Rust")
        .expect("Rust provider must be present");

    assert_eq!(rust.provider_status, "ready");
    assert_eq!(rust.health, "good");
    assert_eq!(rust.provider, "rustup");
    assert!(rust.capabilities.can_install);
    assert!(rust.capabilities.can_activate);
    assert!(rust.capabilities.can_remove);

    assert_eq!(rust.installed.len(), 2, "should have 2 toolchains");

    let stable = rust
        .installed
        .iter()
        .find(|i| i.version == "stable-aarch64-apple-darwin")
        .expect("stable toolchain should be present");
    assert!(stable.active);
    assert_eq!(stable.channel, "stable-aarch64-apple-darwin");
    assert_eq!(stable.source, "rustup");

    let nightly = rust
        .installed
        .iter()
        .find(|i| i.version == "nightly-2024-01-15")
        .expect("nightly toolchain should be present");
    assert!(!nightly.active);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a system-only Rust (cargo present, no rustup).
#[cfg(unix)]
#[test]
fn system_only_rust_fallback() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("system_rust");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(&bin_dir.join("cargo"), "echo 'cargo 1.75.0 (1d8b05fdd 2023-11-20)'");

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let rust = states
        .iter()
        .find(|s| s.family == "Rust")
        .expect("Rust provider must be present");

    assert_eq!(rust.provider_status, "fallback-system");
    assert_eq!(rust.health, "attention");
    assert_eq!(rust.installed.len(), 1);
    // Version should have "cargo " prefix stripped.
    assert_eq!(rust.installed[0].version, "1.75.0");
    assert_eq!(rust.installed[0].channel, "system");
    assert!(!rust.capabilities.can_install);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate rustup installed but with no toolchains.
#[cfg(unix)]
#[test]
fn rustup_empty_no_toolchains() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("rustup_empty");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("rustup"),
        r#"
case "$1" in
  show)
    case "$2" in
      active-toolchain) echo "no active toolchain" ;;
    esac
    ;;
  toolchain)
    case "$2" in
      list) ;;  # empty
    esac
    ;;
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let rust = states
        .iter()
        .find(|s| s.family == "Rust")
        .expect("Rust provider must be present");

    assert_eq!(rust.provider_status, "ready");
    assert!(rust.installed.is_empty());

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a system-only PHP (no phpbrew, but php is on PATH).
#[cfg(unix)]
#[test]
fn system_only_php_fallback() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("system_php");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("php"),
        "echo 'PHP 8.3.12 (cli) (built: Aug  1 2024)'",
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let php = states
        .iter()
        .find(|s| s.family == "PHP")
        .expect("PHP provider must be present");

    assert_eq!(php.provider_status, "fallback-system");
    assert_eq!(php.health, "attention");
    assert_eq!(php.installed.len(), 1);
    assert_eq!(php.installed[0].version, "8.3.12");
    assert_eq!(php.installed[0].channel, "system");
    assert!(!php.capabilities.can_install);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a system-only Ruby (no rbenv, but ruby is on PATH).
#[cfg(unix)]
#[test]
fn system_only_ruby_fallback() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("system_ruby");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("ruby"),
        "echo 'ruby 3.2.2p53 (2023-03-30 revision e51014f9c0) [arm64-darwin22]'",
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let ruby = states
        .iter()
        .find(|s| s.family == "Ruby")
        .expect("Ruby provider must be present");

    assert_eq!(ruby.provider_status, "fallback-system");
    assert_eq!(ruby.health, "attention");
    assert_eq!(ruby.installed.len(), 1);
    assert_eq!(ruby.installed[0].version, "3.2.2p53");
    assert_eq!(ruby.installed[0].channel, "system");
    assert!(!ruby.capabilities.can_install);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a system-only Go (no gvm, but go is on PATH).
#[cfg(unix)]
#[test]
fn system_only_go_fallback() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("system_go");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(&bin_dir.join("go"), "echo 'go version go1.22.5 darwin/arm64'");

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let go = states
        .iter()
        .find(|s| s.family == "Go")
        .expect("Go provider must be present");

    assert_eq!(go.provider_status, "fallback-system");
    assert_eq!(go.health, "attention");
    assert_eq!(go.installed.len(), 1);
    assert_eq!(go.installed[0].version, "1.22.5", "go prefix should be stripped");
    assert_eq!(go.installed[0].channel, "system");
    assert!(!go.capabilities.can_install);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a system-only Java (no SDKMAN, but java is on PATH).
#[cfg(unix)]
#[test]
fn system_only_java_fallback() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("system_java");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("java"),
        "echo 'openjdk version \"21.0.3\" 2024-04-16'",
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let java = states
        .iter()
        .find(|s| s.family == "Java")
        .expect("Java provider must be present");

    assert_eq!(java.provider_status, "fallback-system");
    assert_eq!(java.health, "attention");
    assert_eq!(java.installed.len(), 1);
    assert_eq!(java.installed[0].version, "21.0.3");
    assert_eq!(java.installed[0].channel, "system");
    assert!(!java.capabilities.can_install);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate SDKMAN-managed Java with two installed JDKs and a symlink.
#[cfg(all(unix, target_os = "macos"))]
#[test]
fn sdkman_managed_java_detection() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("sdkman_java");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    // Create SDKMAN! file structure under HOME.
    let sdkman_root = tmp.join(".sdkman");
    let sdkman_init = sdkman_root.join("bin").join("sdkman-init.sh");
    let java_root = sdkman_root.join("candidates").join("java");
    let java_21 = java_root.join("21.0.3");
    let java_17 = java_root.join("17.0.9");
    let java_current = java_root.join("current");

    create_dir(&java_21);
    create_dir(&java_17);
    write_file(&sdkman_init, ""); // empty init script

    // current -> 21.0.3 (active version)
    create_symlink(Path::new("21.0.3"), &java_current);

    // Mock java binary (needed for command_path detection, though SDKMAN path
    // doesn't execute java).
    create_mock_bin(
        &bin_dir.join("java"),
        "echo 'openjdk version \"21.0.3\" 2024-04-16'",
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let java = states
        .iter()
        .find(|s| s.family == "Java")
        .expect("Java provider must be present");

    assert_eq!(java.provider_status, "ready");
    assert_eq!(java.health, "good");
    assert_eq!(java.provider, "SDKMAN!");
    assert!(java.capabilities.can_install);
    assert!(java.capabilities.can_activate);
    assert!(java.capabilities.can_remove);

    assert_eq!(java.installed.len(), 2, "should have 2 JDK installations");

    let j21 = java
        .installed
        .iter()
        .find(|i| i.version == "21.0.3")
        .expect("JDK 21.0.3 should be installed");
    assert!(j21.active, "JDK 21.0.3 should be active (current symlink)");
    assert_eq!(j21.channel, "managed");
    assert_eq!(j21.source, "SDKMAN!");

    let j17 = java
        .installed
        .iter()
        .find(|i| i.version == "17.0.9")
        .expect("JDK 17.0.9 should be installed");
    assert!(!j17.active, "JDK 17.0.9 should not be active");

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate SDKMAN present but no Java candidates installed.
#[cfg(all(unix, target_os = "macos"))]
#[test]
fn sdkman_java_empty() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("sdkman_java_empty");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    // Create SDKMAN! structure but no Java candidates.
    let sdkman_root = tmp.join(".sdkman");
    let sdkman_init = sdkman_root.join("bin").join("sdkman-init.sh");
    let java_root = sdkman_root.join("candidates").join("java");

    create_dir(&java_root);
    write_file(&sdkman_init, "");

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let java = states
        .iter()
        .find(|s| s.family == "Java")
        .expect("Java provider must be present");

    assert_eq!(java.provider_status, "ready");
    assert_eq!(java.health, "attention", "SDKMAN present but no Java should be attention");
    assert!(java.installed.is_empty());

    let has_empty_note = java.notes.iter().any(|n| n.contains("no managed Java"));
    assert!(has_empty_note, "should note that no Java candidates were found");

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate gvm-managed Go with two versions.
#[cfg(all(unix, target_os = "macos"))]
#[test]
fn gvm_managed_go_detection() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("gvm_go");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    // Create gvm script structure under HOME.
    let gvm_script = tmp.join(".gvm").join("scripts").join("gvm");
    write_file(
        &gvm_script,
        r#"
gvm() {
  case "$1" in
    list) printf '=> go1.21.0\n   go1.20.5\n' ;;
  esac
}
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let go = states
        .iter()
        .find(|s| s.family == "Go")
        .expect("Go provider must be present");

    assert_eq!(go.provider_status, "ready");
    assert_eq!(go.health, "good");
    assert_eq!(go.provider, "gvm");
    assert!(go.capabilities.can_install);
    assert!(go.capabilities.can_activate);
    // gvm can_remove is false in the code.
    assert!(!go.capabilities.can_remove);

    assert_eq!(go.installed.len(), 2, "should have 2 Go versions");

    let v21 = go
        .installed
        .iter()
        .find(|i| i.version == "1.21.0")
        .expect("go1.21.0 should be present");
    assert!(v21.active, "go1.21.0 should be active (marked with =>)");
    assert_eq!(v21.channel, "managed");
    assert_eq!(v21.source, "gvm");

    let v20 = go
        .installed
        .iter()
        .find(|i| i.version == "1.20.5")
        .expect("go1.20.5 should be present");
    assert!(!v20.active);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate gvm present but no Go versions installed.
#[cfg(all(unix, target_os = "macos"))]
#[test]
fn gvm_empty_no_versions() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("gvm_empty");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    let gvm_script = tmp.join(".gvm").join("scripts").join("gvm");
    write_file(
        &gvm_script,
        r#"
gvm() {
  case "$1" in
    list) ;;  # empty
  esac
}
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let go = states
        .iter()
        .find(|s| s.family == "Go")
        .expect("Go provider must be present");

    assert_eq!(go.provider_status, "ready");
    assert_eq!(go.health, "attention");
    assert!(go.installed.is_empty());

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a phpbrew-managed PHP environment.
#[cfg(unix)]
#[test]
fn phpbrew_managed_php_detection() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("phpbrew_php");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("phpbrew"),
        r#"
case "$1" in
  list) printf '* php-8.2.24\n  php-8.3.12\n' ;;
  help) printf 'install\nlist\nremove\nswitch\nuse\n' ;;
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let php = states
        .iter()
        .find(|s| s.family == "PHP")
        .expect("PHP provider must be present");

    assert_eq!(php.provider_status, "ready");
    assert_eq!(php.health, "good");
    assert_eq!(php.provider, "phpbrew");
    assert!(php.capabilities.can_install);
    assert!(php.capabilities.can_activate);
    assert!(php.capabilities.can_remove, "phpbrew help lists 'remove'");

    assert_eq!(php.installed.len(), 2);

    let v82 = php
        .installed
        .iter()
        .find(|i| i.version == "8.2.24")
        .expect("php-8.2.24 should be installed");
    assert!(v82.active, "php-8.2.24 is marked active with *");
    assert_eq!(v82.channel, "managed");
    assert_eq!(v82.source, "phpbrew");

    let v83 = php
        .installed
        .iter()
        .find(|i| i.version == "8.3.12")
        .expect("php-8.3.12 should be installed");
    assert!(!v83.active);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate phpbrew without the 'remove' command.
#[cfg(unix)]
#[test]
fn phpbrew_no_remove_support() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("phpbrew_norem");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("phpbrew"),
        r#"
case "$1" in
  list) printf '* php-8.2.24\n  php-8.3.12\n' ;;
  help) printf 'install\nlist\nswitch\nuse\n' ;;  # no 'remove'
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let php = states
        .iter()
        .find(|s| s.family == "PHP")
        .expect("PHP provider must be present");

    assert_eq!(php.provider_status, "ready");
    assert!(php.capabilities.can_install);
    assert!(php.capabilities.can_activate);
    assert!(
        !php.capabilities.can_remove,
        "remove should be disabled when phpbrew help lacks 'remove'"
    );

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate rbenv-managed Ruby with ruby-build plugin.
#[cfg(unix)]
#[test]
fn rbenv_managed_ruby_with_install() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("rbenv_ruby");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("rbenv"),
        r#"
case "$1" in
  commands) printf 'global\ninstall\nlocal\nversions\n' ;;
  version-name) echo "3.2.2" ;;
  versions)
    case "$2" in
      --bare) printf '3.1.6\n3.2.2\n' ;;
    esac
    ;;
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let ruby = states
        .iter()
        .find(|s| s.family == "Ruby")
        .expect("Ruby provider must be present");

    assert_eq!(ruby.provider_status, "ready");
    assert_eq!(ruby.health, "good");
    assert_eq!(ruby.provider, "rbenv");
    assert!(ruby.capabilities.can_install, "ruby-build is available");
    assert!(ruby.capabilities.can_activate);

    assert_eq!(ruby.installed.len(), 2);

    let v32 = ruby
        .installed
        .iter()
        .find(|i| i.version == "3.2.2")
        .expect("Ruby 3.2.2 should be installed");
    assert!(v32.active);
    assert_eq!(v32.channel, "managed");
    assert_eq!(v32.source, "rbenv");

    let v31 = ruby
        .installed
        .iter()
        .find(|i| i.version == "3.1.6")
        .expect("Ruby 3.1.6 should be installed");
    assert!(!v31.active);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate rbenv without ruby-build (install not supported).
#[cfg(unix)]
#[test]
fn rbenv_without_ruby_build() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("rbenv_nobuild");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("rbenv"),
        r#"
case "$1" in
  commands) printf 'global\nlocal\nversions\n' ;;  # no 'install'
  version-name) echo "3.2.2" ;;
  versions)
    case "$2" in
      --bare) printf '3.2.2\n' ;;
    esac
    ;;
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let ruby = states
        .iter()
        .find(|s| s.family == "Ruby")
        .expect("Ruby provider must be present");

    // Without ruby-build, status should be "inspect-only" and can_install
    // should be false.
    assert_eq!(ruby.provider_status, "inspect-only");
    assert!(!ruby.capabilities.can_install, "no ruby-build => no install");
    assert!(ruby.capabilities.can_activate, "can still activate existing versions");

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate a .NET SDK with two installed SDKs.
#[cfg(unix)]
#[test]
fn dotnet_sdk_detection() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("dotnet_sdk");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    // Mock dotnet that reports two SDKs.
    create_mock_bin(
        &bin_dir.join("dotnet"),
        r#"
case "$1" in
  --version) echo "8.0.303" ;;
  --list-sdks) printf '8.0.303 [/usr/share/dotnet/sdk]\n9.0.100 [/usr/share/dotnet/sdk]\n' ;;
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let dotnet = states
        .iter()
        .find(|s| s.family == ".NET")
        .expect(".NET provider must be present");

    assert_eq!(dotnet.provider_status, "fallback-system");
    assert_eq!(dotnet.installed.len(), 2);

    let sdk_8 = dotnet
        .installed
        .iter()
        .find(|i| i.version == "8.0.303")
        .expect("SDK 8.0.303 should be present");
    assert!(sdk_8.active, "active SDK should match --version output");

    let sdk_9 = dotnet
        .installed
        .iter()
        .find(|i| i.version == "9.0.100")
        .expect("SDK 9.0.100 should be present");
    assert!(!sdk_9.active);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// .NET: dotnet exists but --list-sdks fails; falls back to --version.
#[cfg(unix)]
#[test]
fn dotnet_sdk_fallback_to_active_version() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("dotnet_fb");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("dotnet"),
        r#"
case "$1" in
  --version) echo "8.0.303" ;;
  --list-sdks) exit 1 ;;  # fails
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let dotnet = states
        .iter()
        .find(|s| s.family == ".NET")
        .expect(".NET provider must be present");

    // When --list-sdks fails, the code falls back to the active SDK version.
    assert_eq!(dotnet.installed.len(), 1);
    assert_eq!(dotnet.installed[0].version, "8.0.303");
    assert!(dotnet.installed[0].active);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate C/C++ toolchain detection with both clang and gcc.
#[cfg(unix)]
#[test]
fn cpp_toolchain_dual_compilers() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("cpp_dual");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("clang"),
        "echo 'Apple clang version 15.0.0 (clang-1500.3.9.4)'",
    );
    create_mock_bin(
        &bin_dir.join("gcc"),
        "echo 'gcc (Homebrew GCC 13.2.0) 13.2.0'",
    );
    create_mock_bin(&bin_dir.join("cmake"), "echo 'cmake version 3.28.1'");
    create_mock_bin(&bin_dir.join("make"), "echo 'GNU Make 3.81'");

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let cpp = states
        .iter()
        .find(|s| s.family == "C/C++")
        .expect("C/C++ provider must be present");

    assert_eq!(cpp.installed.len(), 2, "should detect both clang and gcc");

    let clang = cpp
        .installed
        .iter()
        .find(|i| i.source == "clang")
        .expect("clang should be detected");
    assert!(clang.active, "clang should be first detected and marked active");
    assert!(clang.version.contains("Apple clang"));

    let gcc = cpp
        .installed
        .iter()
        .find(|i| i.source == "gcc")
        .expect("gcc should be detected");
    assert!(!gcc.active, "gcc is second and should not be active");

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Empty C/C++ toolchain (no compilers at all).
#[cfg(unix)]
#[test]
fn cpp_no_compilers() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("cpp_empty");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let cpp = states
        .iter()
        .find(|s| s.family == "C/C++")
        .expect("C/C++ provider must be present");

    assert_eq!(cpp.provider_status, "missing");
    assert_eq!(cpp.health, "missing");
    assert!(cpp.installed.is_empty());

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

// ===========================================================================
// SECTION 6: Edge cases and error scenarios
// ===========================================================================

/// detect_runtimes_for_host with a bogus host_id should not panic.
#[test]
fn detect_runtimes_for_bogus_host_does_not_panic() {
    // For "native"-like host IDs that aren't "native", the native detection
    // path is used but may not find binaries.  The important thing is that
    // it doesn't panic.
    let result = std::panic::catch_unwind(|| {
        let _ = detect_runtimes_for_host("nonexistent-host-id");
    });
    assert!(result.is_ok(), "detect_runtimes_for_host should not panic on unknown host id");
}

/// detect_runtimes_for_host with an empty host_id should not panic.
#[test]
fn detect_runtimes_for_empty_host_id_does_not_panic() {
    let result = std::panic::catch_unwind(|| {
        let _ = detect_runtimes_for_host("");
    });
    assert!(result.is_ok(), "detect_runtimes_for_host should not panic on empty host id");
}

/// detect_runtimes_for_host with a WSL-prefixed host_id should not panic
/// (even on non-Windows systems where WSL is unavailable).
#[test]
fn detect_runtimes_for_wsl_host_does_not_panic() {
    let result = std::panic::catch_unwind(|| {
        let _ = detect_runtimes_for_host("wsl:Ubuntu");
    });
    assert!(result.is_ok(), "detect_runtimes_for_host should not panic on WSL host id");
}

/// Version strings in installed entries must not contain newlines.
#[test]
fn version_strings_have_no_newlines() {
    for state in detect_runtimes() {
        for inst in &state.installed {
            assert!(
                !inst.version.contains('\n'),
                "version must not contain newline: family={}, version={:?}",
                state.family,
                inst.version
            );
        }
    }
}

/// Channel strings must not contain spaces.
#[test]
fn channel_strings_have_no_spaces() {
    for state in detect_runtimes() {
        for inst in &state.installed {
            assert!(
                !inst.channel.contains(' '),
                "channel must not contain spaces: family={}, version={}, channel={:?}",
                state.family,
                inst.version,
                inst.channel
            );
        }
    }
}

/// If a provider has can_remove=true, it must also have can_install=true.
/// (You cannot remove what you could not install.)
#[test]
fn can_remove_implies_can_install() {
    for state in detect_runtimes() {
        if state.capabilities.can_remove {
            assert!(
                state.capabilities.can_install,
                "family={}: can_remove=true but can_install=false",
                state.family
            );
        }
    }
}

/// If a provider has can_activate=true, it must also have can_install=true.
/// (You cannot activate what you could not install.)
#[test]
fn can_activate_implies_can_install() {
    // Exception: C/C++ toolchain detection doesn't install individual versions.
    // But for all other families, this invariant should hold.
    let managed_families = ["Python", "Node.js", "Rust", "Java", "Go", "PHP", "Ruby"];
    for state in detect_runtimes() {
        if state.capabilities.can_activate && managed_families.contains(&state.family.as_str()) {
            assert!(
                state.capabilities.can_install,
                "family={}: can_activate=true but can_install=false",
                state.family
            );
        }
    }
}

/// detected_binary, when present, should be a non-empty string.
#[test]
fn detected_binary_when_present_is_nonempty() {
    for state in detect_runtimes() {
        if let Some(ref binary) = state.detected_binary {
            assert!(
                !binary.is_empty(),
                "detected_binary is empty for family={}",
                state.family
            );
        }
    }
}

/// Recommended versions should not contain empty strings.
#[test]
fn recommended_versions_are_nonempty() {
    for state in detect_runtimes() {
        for (i, ver) in state.recommended_versions.iter().enumerate() {
            assert!(
                !ver.is_empty(),
                "recommended_versions[{i}] is empty for family={}",
                state.family
            );
        }
    }
}

/// Package tools should not contain empty strings.
#[test]
fn package_tools_are_nonempty() {
    for state in detect_runtimes() {
        for (i, tool) in state.package_tools.iter().enumerate() {
            assert!(
                !tool.is_empty(),
                "package_tools[{i}] is empty for family={}",
                state.family
            );
        }
    }
}

/// Mirror names should not contain empty strings.
#[test]
fn mirrors_are_nonempty() {
    for state in detect_runtimes() {
        for (i, mirror) in state.mirrors.iter().enumerate() {
            assert!(
                !mirror.is_empty(),
                "mirrors[{i}] is empty for family={}",
                state.family
            );
        }
    }
}

/// All installed entries for a given provider should share the same source,
/// except C/C++ which may have multiple compilers (clang, gcc).
#[test]
fn installed_entries_share_source() {
    for state in detect_runtimes() {
        // C/C++ provider intentionally has multiple compilers with different sources
        if state.family == "C/C++" {
            continue;
        }
        if state.installed.len() > 1 {
            let first_source = &state.installed[0].source;
            for inst in &state.installed[1..] {
                assert_eq!(
                    inst.source, *first_source,
                    "mixed sources in family={}: {} vs {}",
                    state.family, first_source, inst.source
                );
            }
        }
    }
}

/// Installed entries with source "system" should have channel "system".
#[test]
fn system_source_implies_system_channel() {
    for state in detect_runtimes() {
        for inst in &state.installed {
            if inst.source == "system" {
                assert_eq!(
                    inst.channel, "system",
                    "family={}, version={}: source=system but channel={}",
                    state.family, inst.version, inst.channel
                );
            }
        }
    }
}

/// Installed entries with source matching the provider should have
/// channel "managed", except rustup which uses toolchain names as channels.
#[test]
fn managed_source_implies_managed_channel() {
    let managed_sources = ["pyenv", "Volta", "SDKMAN!", "gvm", "phpbrew", "rbenv"];
    for state in detect_runtimes() {
        for inst in &state.installed {
            if managed_sources.contains(&inst.source.as_str()) {
                assert_eq!(
                    inst.channel, "managed",
                    "family={}, version={}: source={} but channel={}",
                    state.family, inst.version, inst.source, inst.channel
                );
            }
        }
    }
}

/// Tools list in installed entries should be non-empty.
#[test]
fn installed_tools_are_nonempty() {
    for state in detect_runtimes() {
        for inst in &state.installed {
            assert!(
                !inst.tools.is_empty(),
                "tools list is empty for family={}, version={}",
                state.family,
                inst.version
            );
        }
    }
}

/// Multiple calls to detect_runtimes should return structurally consistent
/// results: same families, same provider names, same tool/mirror counts.
/// Transient differences in provider_status or health can occur when
/// system commands behave inconsistently across invocations.
#[test]
fn detect_runtimes_is_structurally_consistent() {
    let first = detect_runtimes();
    let second = detect_runtimes();
    assert_eq!(first.len(), second.len(), "family count must be stable");
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.family, b.family, "family name must be stable");
        assert_eq!(a.provider, b.provider, "provider name must be stable");
        assert_eq!(
            a.package_tools.len(),
            b.package_tools.len(),
            "package_tools count must be stable for {}",
            a.family
        );
        assert_eq!(
            a.mirrors.len(),
            b.mirrors.len(),
            "mirrors count must be stable for {}",
            a.family
        );
        assert_eq!(
            a.recommended_versions.len(),
            b.recommended_versions.len(),
            "recommended_versions count must be stable for {}",
            a.family
        );
    }
}

// ===========================================================================
// SECTION 7: php brew / rbenv version parsing edge cases via mock
// ===========================================================================

/// Simulate phpbrew listing with unusual formatting (extra spaces, tabs).
#[cfg(unix)]
#[test]
fn phpbrew_unusual_formatting() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("phpbrew_fmt");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("phpbrew"),
        r#"
case "$1" in
  list) printf '  *   php-8.2.24\n    php-8.3.12\n' ;;
  help) printf 'install\nlist\nremove\nswitch\n' ;;
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let php = states
        .iter()
        .find(|s| s.family == "PHP")
        .expect("PHP provider must be present");

    assert_eq!(php.installed.len(), 2);
    assert_eq!(php.installed[0].version, "8.2.24");
    assert!(php.installed[0].active);
    assert_eq!(php.installed[1].version, "8.3.12");
    assert!(!php.installed[1].active);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate rbenv versions output with empty lines mixed in.
#[cfg(unix)]
#[test]
fn rbenv_versions_with_blank_lines() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("rbenv_blank");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("rbenv"),
        r#"
case "$1" in
  commands) printf 'global\ninstall\nlocal\nversions\n' ;;
  version-name) echo "3.2.2" ;;
  versions)
    case "$2" in
      --bare) printf '3.1.6\n\n3.2.2\n\n' ;;
    esac
    ;;
esac
"#,
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let ruby = states
        .iter()
        .find(|s| s.family == "Ruby")
        .expect("Ruby provider must be present");

    // Blank lines should be filtered out.
    assert_eq!(ruby.installed.len(), 2);
    assert_eq!(ruby.installed[0].version, "3.1.6");
    assert_eq!(ruby.installed[1].version, "3.2.2");
    assert!(ruby.installed[1].active);

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Simulate Go version string with unusual prefix.
#[cfg(unix)]
#[test]
fn go_version_unusual_format() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("go_fmt");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    // go version output with extra text.
    create_mock_bin(
        &bin_dir.join("go"),
        "echo 'go version go1.21.0 linux/amd64'",
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let go = states
        .iter()
        .find(|s| s.family == "Go")
        .expect("Go provider must be present");

    assert_eq!(go.installed.len(), 1);
    assert_eq!(go.installed[0].version, "1.21.0");

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Node version with v-prefix is correctly normalized.
#[cfg(unix)]
#[test]
fn node_version_v_prefix_stripped() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("node_vprefix");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(&bin_dir.join("node"), "echo 'v16.20.2'");

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let node = states
        .iter()
        .find(|s| s.family == "Node.js")
        .expect("Node.js provider must be present");

    assert_eq!(node.installed.len(), 1);
    assert_eq!(node.installed[0].version, "16.20.2", "v-prefix should be stripped");

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Java version string normalization via system java mock.
#[cfg(unix)]
#[test]
fn java_version_normalization() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("java_norm");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(
        &bin_dir.join("java"),
        "echo 'openjdk version \"17.0.9\" 2023-10-17'",
    );

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let java = states
        .iter()
        .find(|s| s.family == "Java")
        .expect("Java provider must be present");

    assert_eq!(java.installed.len(), 1);
    assert_eq!(java.installed[0].version, "17.0.9");

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}

/// Rust version normalization via system cargo mock.
#[cfg(unix)]
#[test]
fn rust_version_normalization_from_cargo() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let tmp = create_temp_dir("rust_norm");
    let bin_dir = tmp.join("bin");
    create_dir(&bin_dir);

    create_mock_bin(&bin_dir.join("cargo"), "echo 'cargo 1.74.1 (a5d01c34c 2023-11-20)'");

    let guard = EnvGuard::new(
        &bin_dir.to_string_lossy(),
        &tmp.to_string_lossy(),
    );

    let states = detect_runtimes_for_host("native");
    let rust = states
        .iter()
        .find(|s| s.family == "Rust")
        .expect("Rust provider must be present");

    assert_eq!(rust.installed.len(), 1);
    // cargo version prefix is stripped, taking only the version number.
    assert_eq!(rust.installed[0].version, "1.74.1");

    drop(guard);
    let _ = fs::remove_dir_all(&tmp);
}
