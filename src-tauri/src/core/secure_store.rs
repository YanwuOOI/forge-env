use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{models::ProxySettings, shell};

const PROXY_SERVICE_NAME: &str = "Forge Env Proxy";
const PROXY_SETTINGS_FILE: &str = ".forge-env/proxy-settings.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureStoreErrorKind {
    UnsupportedPlatform,
    ToolUnavailable,
    PathUnavailable,
    InvalidInput,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct SecureStoreError {
    pub kind: SecureStoreErrorKind,
    pub message: String,
}

impl SecureStoreError {
    fn new(kind: SecureStoreErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    fn classify(message: impl Into<String>) -> Self {
        let message = message.into();
        let lowered = message.to_lowercase();
        let kind = if lowered.contains("command not found") || lowered.contains("not available") {
            SecureStoreErrorKind::ToolUnavailable
        } else if lowered.contains("unsupported") {
            SecureStoreErrorKind::UnsupportedPlatform
        } else if lowered.contains("no such file")
            || lowered.contains("cannot resolve home directory")
            || lowered.contains("failed to write")
        {
            SecureStoreErrorKind::PathUnavailable
        } else if lowered.contains("blank") || lowered.contains("required") {
            SecureStoreErrorKind::InvalidInput
        } else {
            SecureStoreErrorKind::Unknown
        };
        Self { kind, message }
    }

    pub fn outcome_title(&self) -> &'static str {
        match self.kind {
            SecureStoreErrorKind::UnsupportedPlatform => "Unsupported secure store",
            SecureStoreErrorKind::ToolUnavailable => "Secure store unavailable",
            SecureStoreErrorKind::PathUnavailable => "Proxy settings path unavailable",
            SecureStoreErrorKind::InvalidInput => "Invalid proxy settings",
            SecureStoreErrorKind::Unknown => "Secure storage failed",
        }
    }

    pub fn next_step(&self) -> &'static str {
        match self.kind {
            SecureStoreErrorKind::UnsupportedPlatform => {
                "Use a supported system credential store or extend Forge Env with a platform-specific adapter."
            }
            SecureStoreErrorKind::ToolUnavailable => {
                "Confirm the platform credential tool is available, then retry saving proxy credentials."
            }
            SecureStoreErrorKind::PathUnavailable => {
                "Confirm the Forge Env settings directory is writable, then retry."
            }
            SecureStoreErrorKind::InvalidInput => {
                "Fill the required proxy fields and retry."
            }
            SecureStoreErrorKind::Unknown => {
                "Inspect the underlying credential-store error, then retry once the environment is stable."
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProxySettingsFile {
    enabled: bool,
    scheme: String,
    host: String,
    port: String,
    username: String,
}

pub fn load_proxy_settings() -> Result<ProxySettings, SecureStoreError> {
    let file = proxy_settings_path()?;
    let persisted = if file.exists() {
        let contents = fs::read_to_string(&file).map_err(|error| {
            SecureStoreError::new(
                SecureStoreErrorKind::PathUnavailable,
                format!("Failed to read {}: {error}", file.to_string_lossy()),
            )
        })?;
        serde_json::from_str::<ProxySettingsFile>(&contents).map_err(|error| {
            SecureStoreError::new(
                SecureStoreErrorKind::PathUnavailable,
                format!("Failed to parse {}: {error}", file.to_string_lossy()),
            )
        })?
    } else {
        ProxySettingsFile {
            enabled: false,
            scheme: "http".into(),
            host: String::new(),
            port: String::new(),
            username: String::new(),
        }
    };

    Ok(ProxySettings {
        enabled: persisted.enabled,
        scheme: persisted.scheme,
        host: persisted.host,
        port: persisted.port,
        username: persisted.username.clone(),
        password_saved: proxy_password_exists(&persisted.username)?,
        secure_store: active_secure_store_label().into(),
    })
}

pub fn save_proxy_settings(
    settings: ProxySettings,
    password: Option<String>,
) -> Result<ProxySettings, SecureStoreError> {
    if settings.enabled && (settings.host.trim().is_empty() || settings.port.trim().is_empty()) {
        return Err(SecureStoreError::new(
            SecureStoreErrorKind::InvalidInput,
            "Enabled proxy settings require both host and port.",
        ));
    }

    let file = proxy_settings_path()?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            SecureStoreError::new(
                SecureStoreErrorKind::PathUnavailable,
                format!("Failed to create {}: {error}", parent.to_string_lossy()),
            )
        })?;
    }

    let persisted = ProxySettingsFile {
        enabled: settings.enabled,
        scheme: settings.scheme.trim().to_string(),
        host: settings.host.trim().to_string(),
        port: settings.port.trim().to_string(),
        username: settings.username.trim().to_string(),
    };
    fs::write(
        &file,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&persisted)
                .map_err(|error| SecureStoreError::classify(error.to_string()))?
        ),
    )
    .map_err(|error| {
        SecureStoreError::new(
            SecureStoreErrorKind::PathUnavailable,
            format!("Failed to write {}: {error}", file.to_string_lossy()),
        )
    })?;

    if let Some(password) = password {
        if !password.trim().is_empty() {
            save_proxy_password(&persisted.username, &password)?;
        }
    }

    load_proxy_settings()
}

pub fn clear_proxy_settings() -> Result<ProxySettings, SecureStoreError> {
    let current = load_proxy_settings()?;
    let file = proxy_settings_path()?;
    if file.exists() {
        fs::remove_file(&file).map_err(|error| {
            SecureStoreError::new(
                SecureStoreErrorKind::PathUnavailable,
                format!("Failed to delete {}: {error}", file.to_string_lossy()),
            )
        })?;
    }
    if !current.username.trim().is_empty() {
        let _ = delete_proxy_password(&current.username);
    }
    load_proxy_settings()
}

fn proxy_settings_path() -> Result<PathBuf, SecureStoreError> {
    let home = shell::home_dir().ok_or_else(|| {
        SecureStoreError::new(
            SecureStoreErrorKind::PathUnavailable,
            "Cannot resolve home directory for proxy settings.",
        )
    })?;
    Ok(PathBuf::from(home).join(PROXY_SETTINGS_FILE))
}

fn active_secure_store_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "Keychain"
    } else if cfg!(target_os = "linux") {
        "Secret Service"
    } else if cfg!(target_os = "windows") {
        "Credential Manager"
    } else {
        "Unsupported"
    }
}

fn proxy_password_exists(username: &str) -> Result<bool, SecureStoreError> {
    if username.trim().is_empty() {
        return Ok(false);
    }
    if cfg!(target_os = "macos") {
        return match shell::run_checked(
            "security",
            &[
                "find-generic-password",
                "-s",
                PROXY_SERVICE_NAME,
                "-a",
                username,
            ],
        ) {
            Ok(_) => Ok(true),
            Err(error) if error.contains("could not be found") => Ok(false),
            Err(error) => Err(SecureStoreError::classify(error)),
        };
    }
    if cfg!(target_os = "linux") {
        return match shell::run_checked(
            "secret-tool",
            &["lookup", "service", PROXY_SERVICE_NAME, "account", username],
        ) {
            Ok(result) => Ok(!result.stdout.trim().is_empty()),
            Err(error) if error.contains("No such secret collection") => Ok(false),
            Err(error) => Err(SecureStoreError::classify(error)),
        };
    }
    Err(SecureStoreError::new(
        SecureStoreErrorKind::UnsupportedPlatform,
        "Proxy credential lookup is not implemented for this platform yet.",
    ))
}

fn save_proxy_password(username: &str, password: &str) -> Result<(), SecureStoreError> {
    if username.trim().is_empty() {
        return Err(SecureStoreError::new(
            SecureStoreErrorKind::InvalidInput,
            "Proxy username is required before saving a password.",
        ));
    }
    if cfg!(target_os = "macos") {
        shell::run_checked(
            "security",
            &[
                "add-generic-password",
                "-U",
                "-s",
                PROXY_SERVICE_NAME,
                "-a",
                username,
                "-w",
                password,
            ],
        )
        .map(|_| ())
        .map_err(SecureStoreError::classify)
    } else if cfg!(target_os = "linux") {
        let script = format!(
            "printf '%s' '{}' | secret-tool store --label='{}' service '{}' account '{}'",
            shell_quote(password),
            PROXY_SERVICE_NAME,
            PROXY_SERVICE_NAME,
            shell_quote(username)
        );
        shell::run_shell_script(&script)
            .map(|_| ())
            .map_err(SecureStoreError::classify)
    } else {
        Err(SecureStoreError::new(
            SecureStoreErrorKind::UnsupportedPlatform,
            "Proxy credential save is not implemented for this platform yet.",
        ))
    }
}

fn delete_proxy_password(username: &str) -> Result<(), SecureStoreError> {
    if username.trim().is_empty() {
        return Ok(());
    }
    if cfg!(target_os = "macos") {
        return match shell::run_checked(
            "security",
            &[
                "delete-generic-password",
                "-s",
                PROXY_SERVICE_NAME,
                "-a",
                username,
            ],
        ) {
            Ok(_) => Ok(()),
            Err(error) if error.contains("could not be found") => Ok(()),
            Err(error) => Err(SecureStoreError::classify(error)),
        };
    }
    if cfg!(target_os = "linux") {
        let script = format!(
            "secret-tool clear service '{}' account '{}'",
            PROXY_SERVICE_NAME,
            shell_quote(username)
        );
        return shell::run_shell_script(&script)
            .map(|_| ())
            .map_err(SecureStoreError::classify);
    }
    Err(SecureStoreError::new(
        SecureStoreErrorKind::UnsupportedPlatform,
        "Proxy credential delete is not implemented for this platform yet.",
    ))
}

fn shell_quote(value: &str) -> String {
    value.replace('\'', "'\"'\"'")
}
