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
