use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{json, Value};

use super::{
    hosts,
    models::{CppProjectPolicyOptions, DotnetProjectPolicyOptions, ProjectRuntimePolicyOptions},
    provider,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectPolicyErrorKind {
    InvalidPath,
    PathUnavailable,
    VersionUnsupported,
    UnsupportedFamily,
    SerializationFailed,
}

#[derive(Debug, Clone)]
pub struct ProjectPolicyError {
    pub kind: ProjectPolicyErrorKind,
    pub message: String,
}

impl ProjectPolicyError {
    fn new(kind: ProjectPolicyErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn outcome_title(&self) -> &'static str {
        match self.kind {
            ProjectPolicyErrorKind::InvalidPath => "Invalid project path",
            ProjectPolicyErrorKind::PathUnavailable => "Project file unavailable",
            ProjectPolicyErrorKind::VersionUnsupported => "Version unsupported",
            ProjectPolicyErrorKind::UnsupportedFamily => "Unsupported project policy",
            ProjectPolicyErrorKind::SerializationFailed => "Project policy write failed",
        }
    }

    pub fn next_step(&self, family: &str) -> String {
        match self.kind {
            ProjectPolicyErrorKind::InvalidPath => {
                "Choose a local project directory discovered by Forge Env and retry.".into()
            }
            ProjectPolicyErrorKind::PathUnavailable => format!(
                "Create the expected {} policy file path or confirm the project directory is writable, then retry.",
                family
            ),
            ProjectPolicyErrorKind::VersionUnsupported => format!(
                "Install or detect a compatible {} toolchain first, then retry the project policy update.",
                family
            ),
            ProjectPolicyErrorKind::UnsupportedFamily => format!(
                "Use project policy apply only for .NET SDK pinning or C/C++ CMake preset hints for now."
            ),
            ProjectPolicyErrorKind::SerializationFailed => {
                "Inspect the existing project policy file for malformed JSON, then retry.".into()
            }
        }
    }
}

pub fn apply_project_runtime_policy(
    host_id: &str,
    project_path: &str,
    family: &str,
    version: &str,
    options: Option<ProjectRuntimePolicyOptions>,
) -> Result<String, ProjectPolicyError> {
    let root = PathBuf::from(project_path);
    if !root.is_dir() {
        return Err(ProjectPolicyError::new(
            ProjectPolicyErrorKind::InvalidPath,
            format!("Project path does not exist or is not a directory: {project_path}"),
        ));
    }

    match family {
        ".NET" => apply_dotnet_global_json(
            host_id,
            &root,
            version,
            options.and_then(|entry| entry.dotnet),
        ),
        "C/C++" => {
            apply_cpp_cmake_presets(host_id, &root, version, options.and_then(|entry| entry.cpp))
        }
        other => Err(ProjectPolicyError::new(
            ProjectPolicyErrorKind::UnsupportedFamily,
            format!("Project runtime policy is not supported for {other}."),
        )),
    }
}

fn apply_dotnet_global_json(
    host_id: &str,
    root: &Path,
    version: &str,
    options: Option<DotnetProjectPolicyOptions>,
) -> Result<String, ProjectPolicyError> {
    let resolved = resolve_dotnet_sdk_version(host_id, version)?;
    let dotnet_options = options.unwrap_or_default();
    let path = root.join("global.json");
    let mut document = read_json_object_or_default(&path)?;
    let sdk = document
        .as_object_mut()
        .ok_or_else(|| {
            ProjectPolicyError::new(
                ProjectPolicyErrorKind::SerializationFailed,
                format!(
                    "global.json is not a JSON object: {}",
                    path.to_string_lossy()
                ),
            )
        })?
        .entry("sdk")
        .or_insert_with(|| json!({}));

    let sdk_object = sdk.as_object_mut().ok_or_else(|| {
        ProjectPolicyError::new(
            ProjectPolicyErrorKind::SerializationFailed,
            format!("The `sdk` field inside global.json must be a JSON object."),
        )
    })?;
    sdk_object.insert("version".into(), Value::String(resolved.clone()));
    let roll_forward = dotnet_options
        .roll_forward
        .as_deref()
        .map(normalize_non_empty)
        .transpose()?
        .unwrap_or_else(|| dotnet_roll_forward_policy(version).to_string());
    let allow_prerelease = dotnet_options
        .allow_prerelease
        .unwrap_or_else(|| resolved.contains('-'));
    sdk_object.insert("rollForward".into(), Value::String(roll_forward));
    sdk_object.insert("allowPrerelease".into(), Value::Bool(allow_prerelease));

    write_json_pretty(&path, &document)?;
    Ok(format!(
        "Pinned .NET SDK {} in {}.",
        resolved,
        path.to_string_lossy()
    ))
}

fn apply_cpp_cmake_presets(
    host_id: &str,
    root: &Path,
    version: &str,
    options: Option<CppProjectPolicyOptions>,
) -> Result<String, ProjectPolicyError> {
    let cmake_lists = root.join("CMakeLists.txt");
    if !cmake_lists.is_file() {
        return Err(ProjectPolicyError::new(
            ProjectPolicyErrorKind::PathUnavailable,
            format!(
                "CMakePresets.json policy requires a CMake project, but no CMakeLists.txt was found in {}.",
                root.to_string_lossy()
            ),
        ));
    }

    let compiler = resolve_cpp_compiler_hint(host_id, version)?;
    let cpp_options = options.unwrap_or_default();
    let generator = cpp_options
        .generator
        .as_deref()
        .map(normalize_non_empty)
        .transpose()?
        .unwrap_or_else(|| resolve_cpp_generator(host_id).to_string());
    let binary_dir = cpp_options
        .binary_dir
        .as_deref()
        .map(normalize_non_empty)
        .transpose()?
        .unwrap_or_else(|| "${sourceDir}/build/forge-env".to_string());
    let toolchain_file = cpp_options
        .toolchain_file
        .as_deref()
        .map(normalize_non_empty)
        .transpose()?;
    let build_type = cpp_options
        .build_type
        .as_deref()
        .map(normalize_non_empty)
        .transpose()?;
    let path = root.join("CMakePresets.json");
    let mut document = read_json_object_or_default(&path)?;
    let object = document.as_object_mut().ok_or_else(|| {
        ProjectPolicyError::new(
            ProjectPolicyErrorKind::SerializationFailed,
            format!(
                "CMakePresets.json is not a JSON object: {}",
                path.to_string_lossy()
            ),
        )
    })?;

    object.insert("version".into(), Value::Number(3.into()));
    let configure_presets = object
        .entry("configurePresets")
        .or_insert_with(|| Value::Array(Vec::new()));
    let presets = configure_presets.as_array_mut().ok_or_else(|| {
        ProjectPolicyError::new(
            ProjectPolicyErrorKind::SerializationFailed,
            "The `configurePresets` field inside CMakePresets.json must be an array.",
        )
    })?;

    let mut cache_variables = serde_json::Map::new();
    cache_variables.insert("CMAKE_C_COMPILER".into(), Value::String(compiler.0.into()));
    cache_variables.insert(
        "CMAKE_CXX_COMPILER".into(),
        Value::String(compiler.1.into()),
    );
    cache_variables.insert("CMAKE_EXPORT_COMPILE_COMMANDS".into(), Value::Bool(true));
    if let Some(ref build_type) = build_type {
        cache_variables.insert("CMAKE_BUILD_TYPE".into(), Value::String(build_type.clone()));
    }
    if let Some(ref toolchain_file) = toolchain_file {
        cache_variables.insert(
            "CMAKE_TOOLCHAIN_FILE".into(),
            Value::String(toolchain_file.clone()),
        );
    }

    let preset_value = json!({
        "name": "forge-env",
        "displayName": "Forge Env Toolchain",
        "generator": generator,
        "binaryDir": binary_dir,
        "cacheVariables": cache_variables
    });

    if let Some(existing) = presets.iter_mut().find(|preset| {
        preset
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| name == "forge-env")
    }) {
        *existing = preset_value;
    } else {
        presets.push(preset_value);
    }

    let build_presets = object
        .entry("buildPresets")
        .or_insert_with(|| Value::Array(Vec::new()));
    let build_preset_items = build_presets.as_array_mut().ok_or_else(|| {
        ProjectPolicyError::new(
            ProjectPolicyErrorKind::SerializationFailed,
            "The `buildPresets` field inside CMakePresets.json must be an array.",
        )
    })?;
    let build_preset = json!({
        "name": "forge-env-build",
        "displayName": "Forge Env Build",
        "configurePreset": "forge-env"
    });
    if let Some(existing) = build_preset_items.iter_mut().find(|preset| {
        preset
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| name == "forge-env-build")
    }) {
        *existing = build_preset;
    } else {
        build_preset_items.push(build_preset);
    }

    write_json_pretty(&path, &document)?;
    Ok(format!(
        "Wrote Forge Env compiler hints to {} using {} / {} with {}.",
        path.to_string_lossy(),
        compiler.0,
        compiler.1,
        generator
    ))
}

fn normalize_non_empty(value: &str) -> Result<String, ProjectPolicyError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ProjectPolicyError::new(
            ProjectPolicyErrorKind::InvalidPath,
            "Project policy options cannot be blank once provided.",
        ));
    }

    Ok(trimmed.to_string())
}

fn read_json_object_or_default(path: &Path) -> Result<Value, ProjectPolicyError> {
    if !path.exists() {
        return Ok(json!({}));
    }

    let content = fs::read_to_string(path).map_err(|error| {
        ProjectPolicyError::new(
            ProjectPolicyErrorKind::PathUnavailable,
            format!("Failed to read {}: {error}", path.to_string_lossy()),
        )
    })?;

    serde_json::from_str(&content).map_err(|error| {
        ProjectPolicyError::new(
            ProjectPolicyErrorKind::SerializationFailed,
            format!("Failed to parse {}: {error}", path.to_string_lossy()),
        )
    })
}

fn write_json_pretty(path: &Path, value: &Value) -> Result<(), ProjectPolicyError> {
    let content = serde_json::to_string_pretty(value).map_err(|error| {
        ProjectPolicyError::new(
            ProjectPolicyErrorKind::SerializationFailed,
            format!("Failed to serialize {}: {error}", path.to_string_lossy()),
        )
    })?;
    fs::write(path, format!("{content}\n")).map_err(|error| {
        ProjectPolicyError::new(
            ProjectPolicyErrorKind::PathUnavailable,
            format!("Failed to write {}: {error}", path.to_string_lossy()),
        )
    })
}

fn resolve_dotnet_sdk_version(
    host_id: &str,
    requested: &str,
) -> Result<String, ProjectPolicyError> {
    let normalized = requested.trim().trim_end_matches(" LTS");
    let runtime = provider::detect_runtimes_for_host(host_id)
        .into_iter()
        .find(|runtime| runtime.family == ".NET")
        .ok_or_else(|| {
            ProjectPolicyError::new(
                ProjectPolicyErrorKind::UnsupportedFamily,
                ".NET runtime detection is unavailable for the selected host.",
            )
        })?;

    runtime
        .installed
        .iter()
        .map(|installation| installation.version.clone())
        .filter(|version| {
            version == normalized
                || version.starts_with(&format!("{normalized}."))
                || version.starts_with(&format!("{normalized}-"))
        })
        .max_by(|left, right| compare_version_like(left, right))
        .ok_or_else(|| {
            ProjectPolicyError::new(
                ProjectPolicyErrorKind::VersionUnsupported,
                format!(
                    "No installed .NET SDK matched `{normalized}` on {host_id}. Install a matching SDK first."
                ),
            )
        })
}

fn resolve_cpp_compiler_hint(
    host_id: &str,
    requested: &str,
) -> Result<(&'static str, &'static str), ProjectPolicyError> {
    let normalized = requested.trim();
    if normalized.eq_ignore_ascii_case("Clang") {
        return Ok(("clang", "clang++"));
    }
    if normalized.eq_ignore_ascii_case("GCC") {
        return Ok(("gcc", "g++"));
    }
    if normalized.eq_ignore_ascii_case("MSVC") {
        return Ok(("cl", "cl"));
    }
    if normalized.eq_ignore_ascii_case("system toolchain") {
        let runtime = provider::detect_runtimes_for_host(host_id)
            .into_iter()
            .find(|runtime| runtime.family == "C/C++")
            .ok_or_else(|| {
                ProjectPolicyError::new(
                    ProjectPolicyErrorKind::UnsupportedFamily,
                    "C/C++ runtime detection is unavailable for the selected host.",
                )
            })?;
        if let Some(active) = runtime
            .installed
            .iter()
            .find(|installation| installation.active)
        {
            return match active.source.as_str() {
                "clang" => Ok(("clang", "clang++")),
                "gcc" => Ok(("gcc", "g++")),
                "MSVC" => Ok(("cl", "cl")),
                _ => Err(ProjectPolicyError::new(
                    ProjectPolicyErrorKind::VersionUnsupported,
                    "The active C/C++ compiler could not be mapped to a CMake compiler hint.",
                )),
            };
        }
    }

    Err(ProjectPolicyError::new(
        ProjectPolicyErrorKind::VersionUnsupported,
        format!(
            "Forge Env could not resolve a compiler hint for `{normalized}`. Choose Clang, GCC, MSVC, or system toolchain."
        ),
    ))
}

fn resolve_cpp_generator(host_id: &str) -> &'static str {
    if hosts::run_capture_on_host(host_id, "ninja", &["--version"]).is_some() {
        "Ninja"
    } else if host_id.starts_with("wsl:") || !cfg!(target_os = "windows") {
        "Unix Makefiles"
    } else {
        "NMake Makefiles"
    }
}

fn dotnet_roll_forward_policy(requested: &str) -> &'static str {
    let normalized = requested.trim().trim_end_matches(" LTS");
    if normalized.split('.').count() >= 3 && normalized.contains('.') {
        "disable"
    } else {
        "latestFeature"
    }
}

fn compare_version_like(left: &str, right: &str) -> std::cmp::Ordering {
    let parse = |value: &str| {
        value
            .split('.')
            .map(|segment| segment.parse::<u32>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    parse(left).cmp(&parse(right))
}

#[cfg(test)]
mod tests {
    use super::{
        apply_cpp_cmake_presets, apply_dotnet_global_json, compare_version_like,
        dotnet_roll_forward_policy, resolve_cpp_compiler_hint, write_json_pretty,
    };
    use crate::core::{
        models::{CppProjectPolicyOptions, DotnetProjectPolicyOptions},
        provider,
    };
    use serde_json::json;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("forge-env-{name}-{stamp}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn compares_version_like_segments() {
        assert!(compare_version_like("8.0.303", "8.0.204").is_gt());
        assert!(compare_version_like("8.0.100", "9.0.100").is_lt());
    }

    #[test]
    fn resolves_cpp_system_toolchain_aliases() {
        assert_eq!(
            resolve_cpp_compiler_hint("native", "Clang").unwrap(),
            ("clang", "clang++")
        );
        assert_eq!(
            resolve_cpp_compiler_hint("native", "GCC").unwrap(),
            ("gcc", "g++")
        );
    }

    #[test]
    fn writes_cmake_presets_file() {
        let root = temp_dir("cmake-presets");
        fs::write(
            root.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.20)\n",
        )
        .expect("write cmake");
        apply_cpp_cmake_presets("native", &root, "Clang", None).expect("apply presets");
        let contents = fs::read_to_string(root.join("CMakePresets.json")).expect("read presets");
        assert!(contents.contains("\"name\": \"forge-env\""));
        assert!(contents.contains("\"CMAKE_C_COMPILER\": \"clang\""));
        assert!(contents.contains("\"CMAKE_EXPORT_COMPILE_COMMANDS\": true"));
        assert!(contents.contains("\"name\": \"forge-env-build\""));
    }

    #[test]
    fn writes_cmake_presets_with_custom_options() {
        let root = temp_dir("cmake-presets-custom");
        fs::write(
            root.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.20)\n",
        )
        .expect("write cmake");
        apply_cpp_cmake_presets(
            "native",
            &root,
            "Clang",
            Some(CppProjectPolicyOptions {
                generator: Some("Unix Makefiles".into()),
                binary_dir: Some("${sourceDir}/build/debug".into()),
                toolchain_file: Some("cmake/toolchains/dev.cmake".into()),
                build_type: Some("Debug".into()),
            }),
        )
        .expect("apply presets");
        let contents = fs::read_to_string(root.join("CMakePresets.json")).expect("read presets");
        assert!(contents.contains("\"generator\": \"Unix Makefiles\""));
        assert!(contents.contains("\"binaryDir\": \"${sourceDir}/build/debug\""));
        assert!(contents.contains("\"CMAKE_BUILD_TYPE\": \"Debug\""));
        assert!(contents.contains("\"CMAKE_TOOLCHAIN_FILE\": \"cmake/toolchains/dev.cmake\""));
    }

    #[test]
    fn pretty_writer_appends_newline() {
        let root = temp_dir("pretty-json");
        let path = root.join("global.json");
        write_json_pretty(&path, &json!({"sdk":{"version":"8.0.303"}})).expect("write json");
        let contents = fs::read_to_string(path).expect("read json");
        assert!(contents.ends_with('\n'));
    }

    #[test]
    fn derives_dotnet_roll_forward_policy() {
        assert_eq!(dotnet_roll_forward_policy("8.0 LTS"), "latestFeature");
        assert_eq!(dotnet_roll_forward_policy("8.0.303"), "disable");
    }

    #[test]
    fn writes_global_json_with_custom_options() {
        let root = temp_dir("dotnet-global-json");
        let installed_version = provider::detect_runtimes_for_host("native")
            .into_iter()
            .find(|runtime| runtime.family == ".NET")
            .and_then(|runtime| runtime.installed.first().map(|entry| entry.version.clone()));
        let Some(installed_version) = installed_version else {
            return;
        };
        apply_dotnet_global_json(
            "native",
            &root,
            &installed_version,
            Some(DotnetProjectPolicyOptions {
                roll_forward: Some("latestMinor".into()),
                allow_prerelease: Some(true),
            }),
        )
        .expect("write global json");
        let contents = fs::read_to_string(root.join("global.json")).expect("read global json");
        assert!(contents.contains("\"rollForward\": \"latestMinor\""));
        assert!(contents.contains("\"allowPrerelease\": true"));
    }
}
