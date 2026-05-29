use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeError {
    /// Provider binary not found (e.g., pyenv, volta, rustup)
    ProviderUnavailable { provider: String, host_id: String },
    /// Version not found or unsupported by the provider
    VersionUnsupported { family: String, version: String },
    /// Host is unreachable or not available
    HostUnavailable { host_id: String },
    /// Permission denied during system operation
    PermissionDenied { detail: String },
    /// Active version conflict — cannot remove the currently active version
    ActiveConflict { family: String, version: String },
    /// Command execution failed
    CommandFailed { command: String, stderr: String },
    /// Configuration file write/read error
    ConfigError { path: String, reason: String },
    /// SQLite storage error
    StorageError { detail: String },
    /// JSON serialization/deserialization error
    SerializationError { detail: String },
    /// Generic internal error with context
    Internal { context: String, detail: String },
}

impl fmt::Display for ForgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderUnavailable { provider, host_id } => {
                write!(f, "{provider} is not available on host {host_id}")
            }
            Self::VersionUnsupported { family, version } => {
                write!(f, "{family} {version} is not supported")
            }
            Self::HostUnavailable { host_id } => {
                write!(f, "Host {host_id} is unavailable")
            }
            Self::PermissionDenied { detail } => {
                write!(f, "Permission denied: {detail}")
            }
            Self::ActiveConflict { family, version } => {
                write!(f, "Cannot remove {family} {version} — it is the active version")
            }
            Self::CommandFailed { command, stderr } => {
                write!(f, "Command '{command}' failed: {stderr}")
            }
            Self::ConfigError { path, reason } => {
                write!(f, "Config error at {path}: {reason}")
            }
            Self::StorageError { detail } => {
                write!(f, "Storage error: {detail}")
            }
            Self::SerializationError { detail } => {
                write!(f, "Serialization error: {detail}")
            }
            Self::Internal { context, detail } => {
                write!(f, "{context}: {detail}")
            }
        }
    }
}

impl ForgeError {
    /// User-facing outcome title for job records
    pub fn outcome_title(&self) -> &'static str {
        match self {
            Self::ProviderUnavailable { .. } => "Provider not found",
            Self::VersionUnsupported { .. } => "Version not supported",
            Self::HostUnavailable { .. } => "Host unavailable",
            Self::PermissionDenied { .. } => "Permission denied",
            Self::ActiveConflict { .. } => "Active version conflict",
            Self::CommandFailed { .. } => "Command failed",
            Self::ConfigError { .. } => "Configuration error",
            Self::StorageError { .. } => "Storage error",
            Self::SerializationError { .. } => "Data error",
            Self::Internal { .. } => "Internal error",
        }
    }

    /// Suggested next step for the user
    pub fn next_step(&self) -> &'static str {
        match self {
            Self::ProviderUnavailable { .. } => {
                "Bootstrap the canonical provider first, then retry."
            }
            Self::VersionUnsupported { .. } => {
                "Check available versions and retry with a supported version."
            }
            Self::HostUnavailable { .. } => {
                "Ensure the host is online and accessible."
            }
            Self::PermissionDenied { .. } => {
                "Check system permissions and retry."
            }
            Self::ActiveConflict { .. } => {
                "Switch to another version first, then remove this one."
            }
            Self::CommandFailed { .. } => {
                "Check the command output and retry."
            }
            Self::ConfigError { .. } => {
                "Verify the configuration file path and permissions."
            }
            Self::StorageError { .. } => {
                "Retry. If the problem persists, check disk space."
            }
            Self::SerializationError { .. } => {
                "Verify the input data format."
            }
            Self::Internal { .. } => {
                "Retry. If the problem persists, check the logs."
            }
        }
    }
}

impl From<ForgeError> for String {
    fn from(err: ForgeError) -> String {
        format!("{}: {} {}", err.outcome_title(), err, err.next_step())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_provider_unavailable() {
        let err = ForgeError::ProviderUnavailable {
            provider: "pyenv".to_string(),
            host_id: "native".to_string(),
        };
        assert!(err.to_string().contains("pyenv"));
        assert!(err.to_string().contains("native"));
    }

    #[test]
    fn outcome_title_matches_all_variants() {
        let errors = vec![
            ForgeError::ProviderUnavailable { provider: "x".into(), host_id: "y".into() },
            ForgeError::VersionUnsupported { family: "x".into(), version: "y".into() },
            ForgeError::HostUnavailable { host_id: "x".into() },
            ForgeError::PermissionDenied { detail: "x".into() },
            ForgeError::ActiveConflict { family: "x".into(), version: "y".into() },
            ForgeError::CommandFailed { command: "x".into(), stderr: "y".into() },
            ForgeError::ConfigError { path: "x".into(), reason: "y".into() },
            ForgeError::StorageError { detail: "x".into() },
            ForgeError::SerializationError { detail: "x".into() },
            ForgeError::Internal { context: "x".into(), detail: "y".into() },
        ];
        for err in &errors {
            assert!(!err.outcome_title().is_empty());
            assert!(!err.next_step().is_empty());
        }
    }

    #[test]
    fn forge_error_into_string() {
        let err = ForgeError::CommandFailed {
            command: "cargo build".into(),
            stderr: "compilation failed".into(),
        };
        let msg: String = err.into();
        assert!(msg.contains("Command failed"));
        assert!(msg.contains("cargo build"));
    }
}
