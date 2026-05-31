use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostSummary {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub architecture: String,
    pub shell: String,
    pub status: String,
    pub recommended_package_manager: String,
    pub path_preview: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostDetail {
    pub summary: HostSummary,
    pub os_version: String,
    pub cwd: String,
    pub home_dir: String,
    pub path_entries_count: usize,
    pub shell_profiles: Vec<String>,
    pub package_managers: Vec<String>,
    pub mirrors_supported: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInstallation {
    pub version: String,
    pub channel: String,
    pub active: bool,
    pub source: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCapabilities {
    pub can_install: bool,
    pub can_activate: bool,
    pub can_remove: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeFamilyState {
    pub family: String,
    pub provider: String,
    pub provider_status: String,
    pub detected_binary: Option<String>,
    pub health: String,
    pub package_tools: Vec<String>,
    pub mirrors: Vec<String>,
    pub recommended_versions: Vec<String>,
    pub capabilities: RuntimeCapabilities,
    pub notes: Vec<String>,
    pub installed: Vec<RuntimeInstallation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedRuntime {
    pub family: String,
    pub version: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProfile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub markers: Vec<String>,
    pub suggested_runtimes: Vec<SuggestedRuntime>,
    pub risk_flags: Vec<String>,
    pub health: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRuntimePolicyOptions {
    #[serde(default)]
    pub dotnet: Option<DotnetProjectPolicyOptions>,
    #[serde(default)]
    pub cpp: Option<CppProjectPolicyOptions>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DotnetProjectPolicyOptions {
    #[serde(default)]
    pub roll_forward: Option<String>,
    #[serde(default)]
    pub allow_prerelease: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CppProjectPolicyOptions {
    #[serde(default)]
    pub generator: Option<String>,
    #[serde(default)]
    pub binary_dir: Option<String>,
    #[serde(default)]
    pub toolchain_file: Option<String>,
    #[serde(default)]
    pub build_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRecord {
    pub id: String,
    pub label: String,
    pub status: String,
    pub family: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub target_name: Option<String>,
    #[serde(default)]
    pub outcome_title: Option<String>,
    #[serde(default)]
    pub outcome_detail: Option<String>,
    #[serde(default)]
    pub next_step: Option<String>,
    pub timestamp: String,
    #[serde(default)]
    pub progress: Option<f32>,
    #[serde(default)]
    pub progress_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBundle {
    pub file_name: String,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub accepted: bool,
    pub ready_to_apply: bool,
    pub runtime_count: usize,
    pub host_count: usize,
    pub planned_count: usize,
    pub applied_count: usize,
    pub issues: Vec<String>,
    pub actions: Vec<ImportAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPreferences {
    pub applied_mirror_preset: Option<String>,
    pub selected_host_id: Option<String>,
    pub last_env_target_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemDependencyState {
    pub name: String,
    pub command: String,
    pub installed: bool,
    pub version: Option<String>,
    pub source_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceState {
    pub name: String,
    pub kind: String,
    pub installed: bool,
    pub running: bool,
    pub health: String,
    pub version: Option<String>,
    pub manager: String,
    pub port: Option<u16>,
    pub data_dir: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceConfigState {
    pub service_name: String,
    pub config_path: Option<String>,
    pub port: Option<u16>,
    pub data_dir: Option<String>,
    pub can_edit_port: bool,
    pub can_edit_data_dir: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceArtifact {
    pub service_name: String,
    pub kind: String,
    pub path: String,
    pub size_bytes: Option<u64>,
    pub created_at: Option<String>,
    pub managed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxySettings {
    pub enabled: bool,
    pub scheme: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password_saved: bool,
    pub secure_store: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVariableSuggestion {
    pub key: String,
    pub value: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvProfileTarget {
    pub path: String,
    pub exists: bool,
    pub managed_by_forge_env: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvPlan {
    pub target_profile: String,
    pub available_profiles: Vec<EnvProfileTarget>,
    pub path_entries: Vec<String>,
    pub variables: Vec<EnvVariableSuggestion>,
    pub managed_block: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSnapshot {
    pub schema_version: u8,
    pub generated_at: String,
    pub hosts: Vec<HostSummary>,
    #[serde(default)]
    pub host_snapshots: Vec<HostTemplateSnapshot>,
    pub runtimes: Vec<RuntimeFamilyState>,
    pub system_dependencies: Vec<SystemDependencyState>,
    pub applied_mirror_preset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostTemplateSnapshot {
    pub host: HostSummary,
    pub runtimes: Vec<RuntimeFamilyState>,
    pub system_dependencies: Vec<SystemDependencyState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAction {
    pub id: String,
    pub host_id: String,
    pub host_label: String,
    pub kind: String,
    pub family: Option<String>,
    pub status: String,
    pub selected: bool,
    pub label: String,
    pub reason: Option<String>,
    pub outcome_title: Option<String>,
    pub next_step: Option<String>,
    pub remediation: Option<String>,
    pub repair_available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_summary_serde_round_trip() {
        let host = HostSummary {
            id: "macos-native".to_string(),
            label: "macOS".to_string(),
            kind: "macos".to_string(),
            architecture: "arm64".to_string(),
            shell: "zsh".to_string(),
            status: "ready".to_string(),
            recommended_package_manager: "Homebrew".to_string(),
            path_preview: vec!["/opt/homebrew/bin".to_string()],
        };
        let json = serde_json::to_string(&host).unwrap();
        assert!(json.contains("\"id\""), "should use camelCase keys");
        assert!(json.contains("\"recommendedPackageManager\""), "should rename snake_case to camelCase");
        let deserialized: HostSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "macos-native");
        assert_eq!(deserialized.recommended_package_manager, "Homebrew");
    }

    #[test]
    fn job_record_partial_json_deserialization() {
        // Missing optional fields should deserialize with defaults
        let json = r#"{"id":"j1","label":"test","status":"completed","timestamp":"2024-01-01"}"#;
        let job: JobRecord = serde_json::from_str(json).unwrap();
        assert_eq!(job.id, "j1");
        assert!(job.category.is_none());
        assert!(job.target_name.is_none());
        assert!(job.outcome_title.is_none());
        assert!(job.family.is_none());
    }

    #[test]
    fn job_record_full_json_deserialization() {
        let json = r#"{
            "id": "j2",
            "label": "Install Python",
            "status": "completed",
            "family": "Python",
            "version": "3.12",
            "category": "runtime",
            "targetName": "Python",
            "outcomeTitle": "Installed",
            "outcomeDetail": "Python 3.12 installed via pyenv",
            "nextStep": "Activate Python 3.12",
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;
        let job: JobRecord = serde_json::from_str(json).unwrap();
        assert_eq!(job.family.as_deref(), Some("Python"));
        assert_eq!(job.category.as_deref(), Some("runtime"));
        assert_eq!(job.target_name.as_deref(), Some("Python"));
    }

    #[test]
    fn project_runtime_policy_options_default() {
        let opts = ProjectRuntimePolicyOptions::default();
        assert!(opts.dotnet.is_none());
        assert!(opts.cpp.is_none());
    }

    #[test]
    fn project_runtime_policy_options_serde_round_trip() {
        let opts = ProjectRuntimePolicyOptions {
            dotnet: Some(DotnetProjectPolicyOptions {
                roll_forward: Some("latestFeature".to_string()),
                allow_prerelease: Some(false),
            }),
            cpp: None,
        };
        let json = serde_json::to_string(&opts).unwrap();
        assert!(json.contains("\"rollForward\""));
        let deserialized: ProjectRuntimePolicyOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.dotnet.unwrap().roll_forward.as_deref(), Some("latestFeature"));
    }

    #[test]
    fn runtime_family_state_serde_round_trip() {
        let state = RuntimeFamilyState {
            family: "Python".to_string(),
            provider: "pyenv".to_string(),
            provider_status: "ready".to_string(),
            detected_binary: Some("/usr/bin/python3".to_string()),
            health: "good".to_string(),
            package_tools: vec!["pip".to_string()],
            mirrors: vec![],
            recommended_versions: vec!["3.12".to_string()],
            capabilities: RuntimeCapabilities {
                can_install: true,
                can_activate: true,
                can_remove: false,
            },
            notes: vec![],
            installed: vec![RuntimeInstallation {
                version: "3.12.4".to_string(),
                channel: "stable".to_string(),
                active: true,
                source: "pyenv".to_string(),
                tools: vec!["pip".to_string()],
            }],
        };
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: RuntimeFamilyState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.family, "Python");
        assert_eq!(deserialized.installed.len(), 1);
        assert!(deserialized.installed[0].active);
    }

    #[test]
    fn template_snapshot_partial_json() {
        // host_snapshots has #[serde(default)], so missing should be empty vec
        let json = r#"{
            "schemaVersion": 2,
            "generatedAt": "2024-01-01",
            "hosts": [],
            "runtimes": [],
            "systemDependencies": []
        }"#;
        let snapshot: TemplateSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.schema_version, 2);
        assert!(snapshot.host_snapshots.is_empty());
        assert!(snapshot.applied_mirror_preset.is_none());
    }

    #[test]
    fn service_state_serde_round_trip() {
        let service = ServiceState {
            name: "Redis".to_string(),
            kind: "cache".to_string(),
            installed: true,
            running: true,
            health: "good".to_string(),
            version: Some("7.2".to_string()),
            manager: "brew".to_string(),
            port: Some(6379),
            data_dir: Some("/var/lib/redis".to_string()),
            notes: vec!["test note".to_string()],
        };
        let json = serde_json::to_string(&service).unwrap();
        assert!(json.contains("\"serviceName\"") == false, "should use camelCase");
        assert!(json.contains("\"dataDir\""));
        let deserialized: ServiceState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.port, Some(6379));
    }

    #[test]
    fn proxy_settings_serde_round_trip() {
        let settings = ProxySettings {
            enabled: true,
            scheme: "http".to_string(),
            host: "127.0.0.1".to_string(),
            port: "7890".to_string(),
            username: "user".to_string(),
            password_saved: true,
            secure_store: "Keychain".to_string(),
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"passwordSaved\""));
        assert!(json.contains("\"secureStore\""));
        let deserialized: ProxySettings = serde_json::from_str(&json).unwrap();
        assert!(deserialized.enabled);
    }

    #[test]
    fn export_bundle_serde_round_trip() {
        let bundle = ExportBundle {
            file_name: "test.json".to_string(),
            payload: "{}".to_string(),
        };
        let json = serde_json::to_string(&bundle).unwrap();
        assert!(json.contains("\"fileName\""));
        let deserialized: ExportBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.file_name, "test.json");
    }

    #[test]
    fn import_result_serde_round_trip() {
        let result = ImportResult {
            accepted: true,
            ready_to_apply: false,
            runtime_count: 2,
            host_count: 1,
            planned_count: 3,
            applied_count: 1,
            issues: vec!["test issue".to_string()],
            actions: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"runtimeCount\""));
        assert!(json.contains("\"readyToApply\""));
        let deserialized: ImportResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.runtime_count, 2);
        assert_eq!(deserialized.issues.len(), 1);
    }

    #[test]
    fn import_action_serde_round_trip() {
        let action = ImportAction {
            id: "a1".to_string(),
            host_id: "h1".to_string(),
            host_label: "macOS".to_string(),
            kind: "install-runtime".to_string(),
            family: Some("Python".to_string()),
            status: "planned".to_string(),
            selected: true,
            label: "Install Python 3.12".to_string(),
            reason: None,
            outcome_title: None,
            next_step: None,
            remediation: None,
            repair_available: false,
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"hostId\""));
        assert!(json.contains("\"repairAvailable\""));
        let deserialized: ImportAction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.family.as_deref(), Some("Python"));
    }

    #[test]
    fn app_preferences_serde_round_trip() {
        let prefs = AppPreferences {
            applied_mirror_preset: Some("Tsinghua".to_string()),
            selected_host_id: Some("native".to_string()),
            last_env_target_profile: Some("~/.zshrc".to_string()),
        };
        let json = serde_json::to_string(&prefs).unwrap();
        assert!(json.contains("\"appliedMirrorPreset\""));
        let deserialized: AppPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.applied_mirror_preset.as_deref(),
            Some("Tsinghua")
        );
    }

    #[test]
    fn app_preferences_partial_json() {
        let json = r#"{}"#;
        let prefs: AppPreferences = serde_json::from_str(json).unwrap();
        assert!(prefs.applied_mirror_preset.is_none());
        assert!(prefs.selected_host_id.is_none());
    }

    #[test]
    fn env_plan_serde_round_trip() {
        let plan = EnvPlan {
            target_profile: "~/.zshrc".to_string(),
            available_profiles: vec![],
            path_entries: vec!["/opt/homebrew/bin".to_string()],
            variables: vec![EnvVariableSuggestion {
                key: "PYENV_ROOT".to_string(),
                value: "$HOME/.pyenv".to_string(),
                reason: "pyenv root".to_string(),
            }],
            managed_block: "# >>> forge-env >>>\n# <<< forge-env <<<".to_string(),
            notes: vec!["test".to_string()],
        };
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("\"targetProfile\""));
        assert!(json.contains("\"managedBlock\""));
        let deserialized: EnvPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.path_entries.len(), 1);
        assert_eq!(deserialized.variables.len(), 1);
    }

    #[test]
    fn service_config_state_serde_round_trip() {
        let config = ServiceConfigState {
            service_name: "Redis".to_string(),
            config_path: Some("/etc/redis.conf".to_string()),
            port: Some(6379),
            data_dir: Some("/var/lib/redis".to_string()),
            can_edit_port: true,
            can_edit_data_dir: false,
            notes: vec!["test".to_string()],
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"serviceName\""));
        assert!(json.contains("\"canEditPort\""));
        let deserialized: ServiceConfigState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.port, Some(6379));
    }

    #[test]
    fn service_artifact_serde_round_trip() {
        let artifact = ServiceArtifact {
            service_name: "Redis".to_string(),
            kind: "backup".to_string(),
            path: "/tmp/redis.tar.gz".to_string(),
            size_bytes: Some(1024),
            created_at: Some("12345".to_string()),
            managed: true,
        };
        let json = serde_json::to_string(&artifact).unwrap();
        assert!(json.contains("\"sizeBytes\""));
        let deserialized: ServiceArtifact = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.size_bytes, Some(1024));
    }

    #[test]
    fn system_dependency_state_serde_round_trip() {
        let dep = SystemDependencyState {
            name: "Git".to_string(),
            command: "git --version".to_string(),
            installed: true,
            version: Some("2.42.0".to_string()),
            source_hint: "Homebrew".to_string(),
        };
        let json = serde_json::to_string(&dep).unwrap();
        assert!(json.contains("\"sourceHint\""));
        let deserialized: SystemDependencyState = serde_json::from_str(&json).unwrap();
        assert!(deserialized.installed);
    }

    #[test]
    fn runtime_installation_serde_round_trip() {
        let inst = RuntimeInstallation {
            version: "3.12.4".to_string(),
            channel: "stable".to_string(),
            active: true,
            source: "pyenv".to_string(),
            tools: vec!["pip".to_string()],
        };
        let json = serde_json::to_string(&inst).unwrap();
        let deserialized: RuntimeInstallation = serde_json::from_str(&json).unwrap();
        assert!(deserialized.active);
        assert_eq!(deserialized.tools.len(), 1);
    }

    #[test]
    fn host_detail_serde_round_trip() {
        let detail = HostDetail {
            summary: HostSummary {
                id: "test".to_string(),
                label: "Test".to_string(),
                kind: "macos".to_string(),
                architecture: "arm64".to_string(),
                shell: "zsh".to_string(),
                status: "ready".to_string(),
                recommended_package_manager: "Homebrew".to_string(),
                path_preview: vec![],
            },
            os_version: "macOS 14.0".to_string(),
            cwd: "/Users/test".to_string(),
            home_dir: "/Users/test".to_string(),
            path_entries_count: 5,
            shell_profiles: vec!["~/.zshrc".to_string()],
            package_managers: vec!["Homebrew".to_string()],
            mirrors_supported: vec!["npm".to_string()],
            notes: vec![],
        };
        let json = serde_json::to_string(&detail).unwrap();
        assert!(json.contains("\"osVersion\""));
        assert!(json.contains("\"shellProfiles\""));
        let deserialized: HostDetail = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.summary.id, "test");
    }
}
