use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::models::{RuntimeCapabilities, RuntimeFamilyState, RuntimeInstallation};
use super::provider::RuntimeProvider;

/// Plugin manifest format (plugin.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    /// Unique plugin identifier (e.g., "com.example.bun-provider")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Plugin version (semver)
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Author name or organization
    pub author: String,
    /// Runtime family this provider manages (e.g., "Bun", "Deno")
    pub family: String,
    /// Provider name (e.g., "bun", "deno")
    pub provider: String,
    /// Minimum Forge Env version required
    pub min_forge_env_version: String,
    /// Capabilities this provider supports
    pub capabilities: PluginCapabilities,
    /// Path to the dynamic library (.dylib/.so/.dll)
    pub library_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilities {
    pub can_install: bool,
    pub can_activate: bool,
    pub can_remove: bool,
    pub can_mirror: bool,
}

/// Information about a loaded plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub loaded: bool,
}

/// Plugin registry managing discovered and loaded plugins
pub struct PluginRegistry {
    plugins_dir: PathBuf,
    discovered: HashMap<String, PluginManifest>,
    loaded: Vec<PluginInfo>,
}

impl PluginRegistry {
    /// Create a new plugin registry, scanning the plugins directory
    pub fn new() -> Self {
        let plugins_dir = Self::default_plugins_dir();
        let mut registry = Self {
            plugins_dir,
            discovered: HashMap::new(),
            loaded: Vec::new(),
        };
        registry.discover();
        registry
    }

    /// Create a registry with a custom plugins directory
    pub fn with_dir(plugins_dir: PathBuf) -> Self {
        let mut registry = Self {
            plugins_dir,
            discovered: HashMap::new(),
            loaded: Vec::new(),
        };
        registry.discover();
        registry
    }

    /// Default plugins directory: ~/.forge-env/plugins/
    fn default_plugins_dir() -> PathBuf {
        dirs()
            .map(|d| d.join("plugins"))
            .unwrap_or_else(|| PathBuf::from(".forge-env/plugins"))
    }

    /// Discover all plugins in the plugins directory
    fn discover(&mut self) {
        if !self.plugins_dir.exists() {
            let _ = std::fs::create_dir_all(&self.plugins_dir);
            return;
        }

        let entries = match std::fs::read_dir(&self.plugins_dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let manifest_path = entry.path().join("plugin.json");
            if !manifest_path.exists() {
                continue;
            }

            match std::fs::read_to_string(&manifest_path) {
                Ok(json) => match serde_json::from_str::<PluginManifest>(&json) {
                    Ok(manifest) => {
                        self.discovered.insert(manifest.id.clone(), manifest);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse plugin manifest {}: {e}", manifest_path.display());
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read plugin manifest {}: {e}", manifest_path.display());
                }
            }
        }
    }

    /// Get all discovered plugins
    pub fn list(&self) -> Vec<&PluginManifest> {
        self.discovered.values().collect()
    }

    /// Get a specific plugin by ID
    pub fn get(&self, id: &str) -> Option<&PluginManifest> {
        self.discovered.get(id)
    }

    /// Get all loaded plugins
    pub fn loaded(&self) -> &[PluginInfo] {
        &self.loaded
    }

    /// Install a plugin from a directory containing plugin.json + library
    pub fn install(&mut self, source_dir: &Path) -> Result<&PluginManifest, String> {
        let manifest_path = source_dir.join("plugin.json");
        let manifest_str = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read {}: {e}", manifest_path.display()))?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_str)
            .map_err(|e| format!("Invalid plugin manifest: {e}"))?;

        let target_dir = self.plugins_dir.join(&manifest.id);
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create plugin directory: {e}"))?;

        // Copy manifest
        std::fs::copy(&manifest_path, target_dir.join("plugin.json"))
            .map_err(|e| format!("Failed to copy manifest: {e}"))?;

        // Copy library
        let lib_source = source_dir.join(&manifest.library_path);
        if lib_source.exists() {
            std::fs::copy(&lib_source, target_dir.join(&manifest.library_path))
                .map_err(|e| format!("Failed to copy library: {e}"))?;
        }

        self.discovered.insert(manifest.id.clone(), manifest.clone());
        Ok(self.discovered.get(&manifest.id).unwrap())
    }

    /// Uninstall a plugin by ID
    pub fn uninstall(&mut self, id: &str) -> Result<(), String> {
        let target_dir = self.plugins_dir.join(id);
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)
                .map_err(|e| format!("Failed to remove plugin directory: {e}"))?;
        }
        self.discovered.remove(id);
        self.loaded.retain(|p| p.manifest.id != id);
        Ok(())
    }

    /// Convert discovered plugins into RuntimeFamilyState for the existing UI
    pub fn detect_plugin_runtimes(&self) -> Vec<RuntimeFamilyState> {
        self.discovered
            .values()
            .map(|manifest| RuntimeFamilyState {
                family: manifest.family.clone(),
                provider: manifest.provider.clone(),
                provider_status: "plugin".to_string(),
                detected_binary: None,
                health: "attention".to_string(),
                package_tools: vec![],
                mirrors: vec![],
                recommended_versions: vec![],
                capabilities: RuntimeCapabilities {
                    can_install: manifest.capabilities.can_install,
                    can_activate: manifest.capabilities.can_activate,
                    can_remove: manifest.capabilities.can_remove,
                },
                notes: vec![format!(
                    "Plugin: {} v{} by {}",
                    manifest.name, manifest.version, manifest.author
                )],
                installed: vec![],
            })
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to get the user's home directory
fn dirs() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_manifest_deserialize() {
        let json = r#"{
            "id": "com.example.bun-provider",
            "name": "Bun Runtime Provider",
            "version": "1.0.0",
            "description": "Manages Bun JavaScript runtime",
            "author": "Example",
            "family": "Bun",
            "provider": "bun",
            "minForgeEnvVersion": "0.1.0",
            "capabilities": {
                "canInstall": true,
                "canActivate": true,
                "canRemove": true,
                "canMirror": false
            },
            "libraryPath": "libbun_provider.dylib"
        }"#;

        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.id, "com.example.bun-provider");
        assert_eq!(manifest.family, "Bun");
        assert!(manifest.capabilities.can_install);
        assert!(!manifest.capabilities.can_mirror);
    }

    #[test]
    fn plugin_manifest_round_trip() {
        let manifest = PluginManifest {
            id: "test".to_string(),
            name: "Test Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test".to_string(),
            family: "TestLang".to_string(),
            provider: "testlang".to_string(),
            min_forge_env_version: "0.1.0".to_string(),
            capabilities: PluginCapabilities {
                can_install: true,
                can_activate: false,
                can_remove: true,
                can_mirror: false,
            },
            library_path: "libtest.dylib".to_string(),
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: PluginManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test");
        assert!(!deserialized.capabilities.can_activate);
    }

    #[test]
    fn plugin_registry_new_creates_dir() {
        let dir = std::env::temp_dir().join(format!("forge-env-plugin-test-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::remove_dir_all(&dir);
        let registry = PluginRegistry::with_dir(dir.clone());
        assert!(dir.exists());
        assert_eq!(registry.list().len(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn plugin_registry_discover_empty() {
        let dir = std::env::temp_dir().join(format!("forge-env-plugin-test-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let registry = PluginRegistry::with_dir(dir.clone());
        assert_eq!(registry.list().len(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn plugin_registry_install_and_uninstall() {
        let dir = std::env::temp_dir().join(format!("forge-env-plugin-test-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::remove_dir_all(&dir);

        let source_dir = dir.join("source");
        std::fs::create_dir_all(&source_dir).unwrap();

        let manifest = PluginManifest {
            id: "test-plugin".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            family: "Test".to_string(),
            provider: "test".to_string(),
            min_forge_env_version: "0.1.0".to_string(),
            capabilities: PluginCapabilities {
                can_install: true,
                can_activate: false,
                can_remove: true,
                can_mirror: false,
            },
            library_path: "libtest.dylib".to_string(),
        };

        std::fs::write(
            source_dir.join("plugin.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let plugins_dir = dir.join("plugins");
        let mut registry = PluginRegistry::with_dir(plugins_dir.clone());
        let installed = registry.install(&source_dir).unwrap();
        assert_eq!(installed.id, "test-plugin");
        assert_eq!(registry.list().len(), 1);

        registry.uninstall("test-plugin").unwrap();
        assert_eq!(registry.list().len(), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn plugin_runtime_state_conversion() {
        let dir = std::env::temp_dir().join(format!("forge-env-plugin-test-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let manifest = PluginManifest {
            id: "bun-plugin".to_string(),
            name: "Bun Provider".to_string(),
            version: "1.0.0".to_string(),
            description: "Bun runtime".to_string(),
            author: "Community".to_string(),
            family: "Bun".to_string(),
            provider: "bun".to_string(),
            min_forge_env_version: "0.1.0".to_string(),
            capabilities: PluginCapabilities {
                can_install: true,
                can_activate: true,
                can_remove: true,
                can_mirror: false,
            },
            library_path: "libbun.dylib".to_string(),
        };

        let mut registry = PluginRegistry::with_dir(dir.clone());
        registry.discovered.insert("bun-plugin".to_string(), manifest);

        let runtimes = registry.detect_plugin_runtimes();
        assert_eq!(runtimes.len(), 1);
        assert_eq!(runtimes[0].family, "Bun");
        assert_eq!(runtimes[0].provider_status, "plugin");
        assert!(runtimes[0].capabilities.can_install);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
