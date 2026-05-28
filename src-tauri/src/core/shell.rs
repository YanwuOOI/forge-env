use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
}

pub fn command_path(command: &str) -> Option<String> {
    let path = env::var_os("PATH")?;
    let path_exts = windows_extensions();

    env::split_paths(&path)
        .flat_map(|dir| candidate_paths(&dir, command, &path_exts))
        .find(|candidate| candidate.is_file())
        .map(|candidate| candidate.to_string_lossy().to_string())
}

pub fn command_exists(command: &str) -> bool {
    command_path(command).is_some()
}

pub fn run_capture(command: &str, args: &[&str]) -> Option<String> {
    let binary = command_path(command)?;
    let output = Command::new(binary).args(args).output().ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return (!stderr.is_empty()).then_some(stderr);
        }

        return Some(stdout);
    }

    None
}

pub fn run_checked(command: &str, args: &[&str]) -> Result<CommandResult, String> {
    let binary =
        command_path(command).ok_or_else(|| format!("Command not found in PATH: {command}"))?;
    let output = Command::new(binary)
        .args(args)
        .output()
        .map_err(|error| format!("Failed to start {command}: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        return Ok(CommandResult { stdout, stderr });
    }

    let detail = if stderr.is_empty() {
        stdout.clone()
    } else {
        stderr.clone()
    };
    Err(format!(
        "{command} {} failed with status {}{}",
        args.join(" "),
        output.status,
        if detail.is_empty() {
            String::new()
        } else {
            format!(": {detail}")
        }
    ))
}

pub fn run_shell_script(script: &str) -> Result<CommandResult, String> {
    let shell = env::var("SHELL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "/bin/bash".to_string());
    let output = Command::new(&shell)
        .args(["-lc", script])
        .output()
        .map_err(|error| format!("Failed to start shell script via {shell}: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        return Ok(CommandResult { stdout, stderr });
    }

    let detail = if stderr.is_empty() {
        stdout.clone()
    } else {
        stderr.clone()
    };
    Err(format!(
        "Shell script failed with status {}{}",
        output.status,
        if detail.is_empty() {
            String::new()
        } else {
            format!(": {detail}")
        }
    ))
}

pub fn home_dir() -> Option<String> {
    env::var("HOME")
        .ok()
        .or_else(|| env::var("USERPROFILE").ok())
}

pub fn path_exists(path: &Path) -> bool {
    path.exists()
}

fn candidate_paths(dir: &Path, command: &str, path_exts: &[String]) -> Vec<PathBuf> {
    if cfg!(windows) {
        let mut candidates = Vec::with_capacity(path_exts.len().saturating_add(1));
        candidates.push(dir.join(command));
        candidates.extend(
            path_exts
                .iter()
                .map(|ext| dir.join(format!("{command}{ext}"))),
        );
        candidates
    } else {
        vec![dir.join(command)]
    }
}

fn windows_extensions() -> Vec<String> {
    if !cfg!(windows) {
        return Vec::new();
    }

    env::var("PATHEXT")
        .map(|value| {
            value
                .split(';')
                .map(str::trim)
                .filter(|ext| !ext.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_else(|_| vec![".exe".into(), ".cmd".into(), ".bat".into()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_dir_returns_home_env() {
        // HOME is typically set in test environments
        let result = home_dir();
        assert!(result.is_some(), "HOME or USERPROFILE should be set in test env");
    }

    #[test]
    fn candidate_paths_non_windows_returns_single_path() {
        let dir = PathBuf::from("/usr/bin");
        let paths = candidate_paths(&dir, "ls", &[]);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], PathBuf::from("/usr/bin/ls"));
    }

    #[test]
    fn path_exists_for_existing_path() {
        assert!(path_exists(Path::new("/")));
    }

    #[test]
    fn path_exists_for_nonexistent_path() {
        assert!(!path_exists(Path::new("/nonexistent/path/that/does/not/exist")));
    }

    #[test]
    fn command_result_construction() {
        let result = CommandResult {
            stdout: "hello".to_string(),
            stderr: "world".to_string(),
        };
        assert_eq!(result.stdout, "hello");
        assert_eq!(result.stderr, "world");
    }

    #[test]
    fn run_capture_echo() {
        let result = run_capture("echo", &["hello world"]);
        assert_eq!(result.as_deref(), Some("hello world"));
    }

    #[test]
    fn run_capture_nonexistent_command() {
        let result = run_capture("nonexistent_command_12345", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn run_checked_echo() {
        let result = run_checked("echo", &["test"]);
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.stdout, "test");
    }

    #[test]
    fn run_checked_false_returns_err() {
        let result = run_checked("false", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn command_exists_ls() {
        assert!(command_exists("ls"));
    }

    #[test]
    fn command_exists_nonexistent() {
        assert!(!command_exists("nonexistent_command_12345"));
    }
}
