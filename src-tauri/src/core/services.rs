use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    hosts,
    models::{ServiceArtifact, ServiceConfigState, ServiceState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceErrorKind {
    PermissionDenied,
    PortConflict,
    PathUnavailable,
    ToolUnavailable,
    HostUnavailable,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ServiceError {
    pub kind: ServiceErrorKind,
    pub message: String,
}

impl ServiceError {
    fn new(kind: ServiceErrorKind, message: impl Into<String>) -> Self {
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
            ServiceErrorKind::PermissionDenied
        } else if lowered.contains("address already in use")
            || lowered.contains("port already in use")
            || (lowered.contains("bind") && lowered.contains("in use"))
        {
            ServiceErrorKind::PortConflict
        } else if lowered.contains("read-only")
            || lowered.contains("writable")
            || lowered.contains("could not determine")
            || lowered.contains("no writable")
        {
            ServiceErrorKind::PathUnavailable
        } else if lowered.contains("command not found")
            || lowered.contains("not found")
            || lowered.contains("unsupported")
            || lowered.contains("not implemented")
        {
            ServiceErrorKind::ToolUnavailable
        } else if lowered.contains("unknown host")
            || lowered.contains("wsl command is not available")
        {
            ServiceErrorKind::HostUnavailable
        } else {
            ServiceErrorKind::Unknown
        };
        Self { kind, message }
    }

    pub fn outcome_title(&self) -> &'static str {
        match self.kind {
            ServiceErrorKind::PermissionDenied => "Permission denied",
            ServiceErrorKind::PortConflict => "Port conflict",
            ServiceErrorKind::PathUnavailable => "Path unavailable",
            ServiceErrorKind::ToolUnavailable => "Service tooling unavailable",
            ServiceErrorKind::HostUnavailable => "Host unavailable",
            ServiceErrorKind::Unsupported => "Unsupported operation",
            ServiceErrorKind::Unknown => "Service action failed",
        }
    }

    pub fn next_step(&self, target_name: &str) -> String {
        match self.kind {
            ServiceErrorKind::PermissionDenied => format!(
                "Grant the required host permission or rerun {} through a user that can access the service manager and config path.",
                target_name
            ),
            ServiceErrorKind::PortConflict => format!(
                "Free the conflicting port or choose a different {} port before retrying.",
                target_name
            ),
            ServiceErrorKind::PathUnavailable => format!(
                "Confirm that the target config path for {} exists and is writable on the selected host.",
                target_name
            ),
            ServiceErrorKind::ToolUnavailable => format!(
                "Install or expose the required host tooling for {} before retrying.",
                target_name
            ),
            ServiceErrorKind::HostUnavailable => {
                "Re-select a reachable host or restore the missing WSL/native execution path before retrying."
                    .into()
            }
            ServiceErrorKind::Unsupported => format!(
                "{} is not supported through Forge Env on the selected host yet.",
                target_name
            ),
            ServiceErrorKind::Unknown => format!(
                "Inspect {} logs and host state, then retry once the underlying service issue is understood.",
                target_name
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
}

struct ServiceSpec {
    name: &'static str,
    kind: &'static str,
    binaries: &'static [&'static str],
    brew_names: &'static [&'static str],
    system_names: &'static [&'static str],
    direct_start: Option<fn(&str) -> Result<String, ServiceError>>,
    direct_stop: Option<fn(&str) -> Result<String, ServiceError>>,
    direct_restart: Option<fn(&str) -> Result<String, ServiceError>>,
}

const REDIS: ServiceSpec = ServiceSpec {
    name: "Redis",
    kind: "cache",
    binaries: &["redis-server", "redis-cli"],
    brew_names: &["redis", "redis-stack-server"],
    system_names: &["redis", "redis-server"],
    direct_start: Some(start_redis_direct),
    direct_stop: Some(stop_redis_direct),
    direct_restart: Some(restart_redis_direct),
};

const POSTGRESQL: ServiceSpec = ServiceSpec {
    name: "PostgreSQL",
    kind: "database",
    binaries: &["postgres", "psql", "pg_ctl", "pg_isready"],
    brew_names: &[
        "postgresql",
        "postgresql@17",
        "postgresql@16",
        "postgresql@15",
        "postgresql@14",
    ],
    system_names: &[
        "postgresql",
        "postgresql@17-main",
        "postgresql@16-main",
        "postgresql@15-main",
        "postgresql@14-main",
    ],
    direct_start: Some(start_postgresql_direct),
    direct_stop: Some(stop_postgresql_direct),
    direct_restart: Some(restart_postgresql_direct),
};

const MYSQL: ServiceSpec = ServiceSpec {
    name: "MySQL",
    kind: "database",
    binaries: &["mysqld", "mysql", "mysqladmin"],
    brew_names: &["mysql", "mysql@8.0", "mariadb"],
    system_names: &["mysql", "mysqld", "mariadb"],
    direct_start: None,
    direct_stop: None,
    direct_restart: None,
};

const MONGODB: ServiceSpec = ServiceSpec {
    name: "MongoDB",
    kind: "database",
    binaries: &["mongod", "mongosh", "mongo"],
    brew_names: &[
        "mongodb-community",
        "mongodb-community@8.0",
        "mongodb-community@7.0",
    ],
    system_names: &["mongod", "mongodb"],
    direct_start: None,
    direct_stop: None,
    direct_restart: None,
};

const RABBITMQ: ServiceSpec = ServiceSpec {
    name: "RabbitMQ",
    kind: "queue",
    binaries: &["rabbitmq-server", "rabbitmqctl", "rabbitmq-diagnostics"],
    brew_names: &["rabbitmq"],
    system_names: &["rabbitmq-server", "rabbitmq"],
    direct_start: None,
    direct_stop: None,
    direct_restart: None,
};

const NGINX: ServiceSpec = ServiceSpec {
    name: "Nginx",
    kind: "proxy",
    binaries: &["nginx"],
    brew_names: &["nginx"],
    system_names: &["nginx"],
    direct_start: None,
    direct_stop: None,
    direct_restart: None,
};

const SERVICE_SPECS: &[ServiceSpec] = &[REDIS, POSTGRESQL, MYSQL, MONGODB, RABBITMQ, NGINX];

const CONFIG_BEGIN: &str = "# >>> forge-env service >>>";
const CONFIG_END: &str = "# <<< forge-env service <<<";
const ARTIFACT_DIR: &str = ".forge-env/service-artifacts";

pub fn detect_services_for_host(host_id: &str) -> Vec<ServiceState> {
    SERVICE_SPECS
        .iter()
        .map(|spec| detect_service_for_host(host_id, spec))
        .collect()
}

pub fn detect_service_configs_for_host(host_id: &str) -> Vec<ServiceConfigState> {
    SERVICE_SPECS
        .iter()
        .map(|spec| detect_service_config_for_host(host_id, spec))
        .collect()
}

pub fn detect_service_artifacts_for_host(host_id: &str) -> Vec<ServiceArtifact> {
    supported_snapshot_services()
        .into_iter()
        .flat_map(|service_name| {
            ["backup", "export"]
                .into_iter()
                .flat_map(move |kind| list_service_artifacts_for_kind(host_id, service_name, kind))
        })
        .collect()
}

pub fn create_service_backup_for_host(host_id: &str, name: &str) -> Result<String, ServiceError> {
    ensure_snapshot_capable_service(name)?;
    let service = require_service_with_data_dir(host_id, name)?;
    ensure_snapshot_safe_state(&service)?;
    let archive_path = managed_service_artifact_path(host_id, name, "backup")?;
    write_service_archive(
        host_id,
        name,
        &service.data_dir.clone().unwrap_or_default(),
        &archive_path,
    )?;
    Ok(format!(
        "Created a managed {} data-dir snapshot at {}.",
        name,
        archive_path.to_string_lossy()
    ))
}

pub fn export_service_data_for_host(host_id: &str, name: &str) -> Result<String, ServiceError> {
    ensure_snapshot_capable_service(name)?;
    let service = require_service_with_data_dir(host_id, name)?;
    ensure_snapshot_safe_state(&service)?;
    let archive_path = managed_service_artifact_path(host_id, name, "export")?;
    write_service_archive(
        host_id,
        name,
        &service.data_dir.clone().unwrap_or_default(),
        &archive_path,
    )?;
    Ok(format!(
        "Exported the current {} data directory to {}.",
        name,
        archive_path.to_string_lossy()
    ))
}

pub fn create_service_logical_backup_for_host(
    host_id: &str,
    name: &str,
) -> Result<String, ServiceError> {
    let archive_path = managed_service_artifact_path(host_id, name, "logical-backup")?;
    match name {
        "PostgreSQL" => write_postgresql_logical_backup(host_id, &archive_path)?,
        "MySQL" => write_mysql_logical_backup(host_id, &archive_path)?,
        "Redis" => {
            return Err(ServiceError::new(
                ServiceErrorKind::Unsupported,
                "Forge Env does not expose a Redis logical backup stream yet. Use snapshot backup or export instead.",
            ));
        }
        _ => {
            return Err(ServiceError::new(
                ServiceErrorKind::Unsupported,
                format!("{name} does not expose logical backup through Forge Env yet."),
            ));
        }
    }
    Ok(format!(
        "Created a managed {} logical backup at {}.",
        name,
        archive_path.to_string_lossy()
    ))
}

pub fn restore_service_backup_for_host(
    host_id: &str,
    name: &str,
    archive_path: Option<String>,
) -> Result<String, ServiceError> {
    ensure_snapshot_capable_service(name)?;
    let service = require_service_with_data_dir(host_id, name)?;
    ensure_snapshot_safe_state(&service)?;
    let data_dir = service.data_dir.clone().unwrap_or_default();
    let archive = archive_path
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| latest_managed_service_backup(host_id, name))
        .ok_or_else(|| {
            ServiceError::new(
                ServiceErrorKind::PathUnavailable,
                format!("No managed {} backup archive was found for restore.", name),
            )
        })?;
    if !hosts::path_exists_on_host(host_id, &archive) {
        return Err(ServiceError::new(
            ServiceErrorKind::PathUnavailable,
            format!(
                "Restore archive does not exist on the selected host: {}",
                archive.to_string_lossy()
            ),
        ));
    }

    let stamp = unix_timestamp().to_string();
    let restore_backup = PathBuf::from(format!("{data_dir}.pre-restore-{stamp}"));
    restore_service_archive(
        host_id,
        &archive,
        &PathBuf::from(&data_dir),
        &restore_backup,
    )?;
    Ok(format!(
        "Restored {} from {} into {}. The previous data directory, if present, was moved to {}.",
        name,
        archive.to_string_lossy(),
        data_dir,
        restore_backup.to_string_lossy()
    ))
}

pub fn validate_service_artifact_for_host(
    host_id: &str,
    path: &str,
) -> Result<String, ServiceError> {
    let artifact = PathBuf::from(path.trim());
    if !hosts::path_exists_on_host(host_id, &artifact) {
        return Err(ServiceError::new(
            ServiceErrorKind::PathUnavailable,
            format!(
                "Artifact does not exist on the selected host: {}",
                artifact.to_string_lossy()
            ),
        ));
    }
    let artifact_path = shell_quote(artifact.to_string_lossy().as_ref());
    let script = if path.ends_with(".tar.gz") || path.ends_with(".tgz") {
        format!("set -e; tar -tzf '{artifact_path}' >/dev/null")
    } else if path.ends_with(".sql.gz") {
        format!("set -e; gzip -t '{artifact_path}'")
    } else {
        return Err(ServiceError::new(
            ServiceErrorKind::Unsupported,
            format!(
                "Forge Env only validates managed .tar.gz or .sql.gz artifacts right now: {path}"
            ),
        ));
    };
    hosts::run_shell_script_on_host(host_id, &script).map_err(ServiceError::classify)?;
    Ok(format!(
        "Validated artifact integrity for {} on {}.",
        artifact.to_string_lossy(),
        host_id
    ))
}

pub fn delete_service_artifact_for_host(host_id: &str, path: &str) -> Result<String, ServiceError> {
    let artifact = PathBuf::from(path.trim());
    ensure_managed_artifact_path(host_id, &artifact)?;
    if !hosts::path_exists_on_host(host_id, &artifact) {
        return Err(ServiceError::new(
            ServiceErrorKind::PathUnavailable,
            format!(
                "Artifact does not exist on the selected host: {}",
                artifact.to_string_lossy()
            ),
        ));
    }
    let quoted = shell_quote(artifact.to_string_lossy().as_ref());
    hosts::run_shell_script_on_host(host_id, &format!("rm -f '{quoted}'"))
        .map_err(ServiceError::classify)?;
    Ok(format!(
        "Deleted managed artifact {} on {}.",
        artifact.to_string_lossy(),
        host_id
    ))
}

pub fn control_service_for_host(
    host_id: &str,
    name: &str,
    action: ServiceAction,
) -> Result<String, ServiceError> {
    let spec = SERVICE_SPECS
        .iter()
        .find(|spec| spec.name == name)
        .ok_or_else(|| {
            ServiceError::new(
                ServiceErrorKind::Unsupported,
                format!("Unsupported service: {name}"),
            )
        })?;

    let mut last_error: Option<ServiceError> = None;

    if has_command(host_id, "brew") {
        for candidate in spec.brew_names {
            match hosts::run_checked_on_host(
                host_id,
                "brew",
                &["services", action_name(action), candidate],
            ) {
                Ok(result) => {
                    return Ok(summarize_action(
                        spec.name,
                        host_id,
                        action,
                        "Homebrew services",
                        &result.stdout,
                        &result.stderr,
                    ))
                }
                Err(error) => last_error = Some(ServiceError::classify(error)),
            }
        }
    }

    if has_command(host_id, "systemctl") {
        for candidate in spec.system_names {
            match hosts::run_checked_on_host(
                host_id,
                "systemctl",
                &[action_name(action), candidate],
            ) {
                Ok(result) => {
                    return Ok(summarize_action(
                        spec.name,
                        host_id,
                        action,
                        "systemctl",
                        &result.stdout,
                        &result.stderr,
                    ))
                }
                Err(error) => last_error = Some(ServiceError::classify(error)),
            }
        }
    }

    if has_command(host_id, "service") {
        for candidate in spec.system_names {
            match hosts::run_checked_on_host(host_id, "service", &[candidate, action_name(action)])
            {
                Ok(result) => {
                    return Ok(summarize_action(
                        spec.name,
                        host_id,
                        action,
                        "service",
                        &result.stdout,
                        &result.stderr,
                    ))
                }
                Err(error) => last_error = Some(ServiceError::classify(error)),
            }
        }
    }

    let direct_result = match action {
        ServiceAction::Start => spec
            .direct_start
            .ok_or_else(|| {
                last_error.clone().unwrap_or_else(|| {
                    ServiceError::new(
                        ServiceErrorKind::Unsupported,
                        unsupported_direct_action(spec.name, "start"),
                    )
                })
            })
            .and_then(|handler| handler(host_id)),
        ServiceAction::Stop => spec
            .direct_stop
            .ok_or_else(|| {
                last_error.clone().unwrap_or_else(|| {
                    ServiceError::new(
                        ServiceErrorKind::Unsupported,
                        unsupported_direct_action(spec.name, "stop"),
                    )
                })
            })
            .and_then(|handler| handler(host_id)),
        ServiceAction::Restart => spec
            .direct_restart
            .ok_or_else(|| {
                last_error.clone().unwrap_or_else(|| {
                    ServiceError::new(
                        ServiceErrorKind::Unsupported,
                        unsupported_direct_action(spec.name, "restart"),
                    )
                })
            })
            .and_then(|handler| handler(host_id)),
    };

    match direct_result {
        Ok(result) => Ok(result),
        Err(error) => Err(last_error.unwrap_or(error)),
    }
}

pub fn apply_service_config_for_host(
    host_id: &str,
    name: &str,
    port: Option<u16>,
    data_dir: Option<String>,
) -> Result<String, ServiceError> {
    let spec = SERVICE_SPECS
        .iter()
        .find(|spec| spec.name == name)
        .ok_or_else(|| {
            ServiceError::new(
                ServiceErrorKind::Unsupported,
                format!("Unsupported service: {name}"),
            )
        })?;

    match spec.name {
        "Redis" => apply_redis_config(host_id, port, data_dir),
        "PostgreSQL" => apply_postgresql_config(host_id, port, data_dir),
        "MySQL" => apply_mysql_config(host_id, port, data_dir),
        "MongoDB" => Err(ServiceError::new(
            ServiceErrorKind::Unsupported,
            "MongoDB config editing is not automated yet because mongod.conf is YAML-based and needs structured writes.",
        )),
        "RabbitMQ" => Err(ServiceError::new(
            ServiceErrorKind::Unsupported,
            "RabbitMQ config editing is not automated yet; use the detected rabbitmq.conf path directly for now.",
        )),
        "Nginx" => Err(ServiceError::new(
            ServiceErrorKind::Unsupported,
            "Nginx config editing is not automated yet. Preview the detected config path and edit it manually for now.",
        )),
        _ => Err(ServiceError::new(
            ServiceErrorKind::Unsupported,
            format!("Unsupported service config target: {}", spec.name),
        )),
    }
}

pub fn apply_service_config_and_reconcile_for_host(
    host_id: &str,
    name: &str,
    port: Option<u16>,
    data_dir: Option<String>,
) -> Result<String, ServiceError> {
    let config_outcome = apply_service_config_for_host(host_id, name, port, data_dir)?;
    let service = detect_services_for_host(host_id)
        .into_iter()
        .find(|service| service.name == name)
        .ok_or_else(|| {
            ServiceError::new(
                ServiceErrorKind::Unsupported,
                format!("Unsupported service: {name}"),
            )
        })?;

    if !service.installed {
        return Ok(format!(
            "{} {} is not installed on {}, so Forge Env only wrote the config override.",
            config_outcome, name, host_id
        ));
    }

    let action = if service.running {
        ServiceAction::Restart
    } else {
        ServiceAction::Start
    };
    let service_outcome = control_service_for_host(host_id, name, action)?;
    Ok(format!("{} {}", config_outcome, service_outcome))
}

fn detect_service_for_host(host_id: &str, spec: &ServiceSpec) -> ServiceState {
    let installed = spec
        .binaries
        .iter()
        .any(|command| has_command(host_id, command));
    let running = match spec.name {
        "Redis" => redis_running(host_id),
        "PostgreSQL" => postgresql_running(host_id),
        "MySQL" => mysql_running(host_id),
        "MongoDB" => mongodb_running(host_id),
        "RabbitMQ" => rabbitmq_running(host_id),
        "Nginx" => nginx_running(host_id),
        _ => false,
    };
    let version = match spec.name {
        "Redis" => first_line(
            hosts::run_capture_on_host(host_id, "redis-server", &["--version"])
                .or_else(|| hosts::run_capture_on_host(host_id, "redis-cli", &["--version"])),
        ),
        "PostgreSQL" => first_line(
            hosts::run_capture_on_host(host_id, "postgres", &["--version"])
                .or_else(|| hosts::run_capture_on_host(host_id, "psql", &["--version"])),
        ),
        "MySQL" => first_line(
            hosts::run_capture_on_host(host_id, "mysqld", &["--version"])
                .or_else(|| hosts::run_capture_on_host(host_id, "mysql", &["--version"])),
        ),
        "MongoDB" => first_line(
            hosts::run_capture_on_host(host_id, "mongod", &["--version"])
                .or_else(|| hosts::run_capture_on_host(host_id, "mongosh", &["--version"]))
                .or_else(|| hosts::run_capture_on_host(host_id, "mongo", &["--version"])),
        ),
        "RabbitMQ" => first_line(
            hosts::run_capture_on_host(host_id, "rabbitmqctl", &["version"]).or_else(|| {
                hosts::run_capture_on_host(host_id, "rabbitmq-diagnostics", &["server_version"])
            }),
        ),
        "Nginx" => first_line(
            hosts::run_capture_on_host(host_id, "nginx", &["-v"])
                .or_else(|| hosts::run_capture_on_host(host_id, "nginx", &["-V"])),
        ),
        _ => None,
    };

    let manager = if has_command(host_id, "brew") {
        "Homebrew services"
    } else if has_command(host_id, "systemctl") {
        "systemctl"
    } else if has_command(host_id, "service") {
        "service"
    } else if spec.direct_start.is_some() {
        "direct local process"
    } else {
        "manual"
    }
    .to_string();

    let port = match spec.name {
        "Redis" => redis_port(host_id).or(Some(6379)),
        "PostgreSQL" => postgresql_port(host_id).or(Some(5432)),
        "MySQL" => mysql_port(host_id).or(Some(3306)),
        "MongoDB" => mongodb_port(host_id).or(Some(27017)),
        "RabbitMQ" => Some(5672),
        "Nginx" => nginx_port(host_id).or(Some(80)),
        _ => None,
    };

    let data_dir = match spec.name {
        "Redis" => redis_data_dir(host_id),
        "PostgreSQL" => postgresql_data_dir(host_id).or_else(|| postgres_managed_data_dir(host_id)),
        "MySQL" => mysql_data_dir(host_id),
        "MongoDB" => mongodb_data_dir(host_id),
        "RabbitMQ" => rabbitmq_data_dir(host_id),
        "Nginx" => None,
        _ => None,
    };

    let mut notes = Vec::new();
    if !installed {
        notes.push(format!(
            "{} binaries were not detected on this host.",
            spec.name
        ));
    } else if !running {
        notes.push(format!(
            "{} is installed but not currently accepting local connections.",
            spec.name
        ));
    } else {
        notes.push(format!(
            "{} responds on the expected local control path.",
            spec.name
        ));
    }
    if spec.name == "MySQL" && manager == "manual" {
        notes.push(
            "MySQL direct bootstrap is not implemented yet; use Homebrew services or system service management."
                .into(),
        );
    }
    if spec.name == "MongoDB" {
        notes.push(
            "MongoDB is controllable through host service managers, but structured mongod.conf editing is still pending."
                .into(),
        );
    }
    if spec.name == "RabbitMQ" {
        notes.push(
            "RabbitMQ management currently focuses on service control; rabbitmq.conf remains preview-only."
                .into(),
        );
    }
    if spec.name == "Nginx" {
        notes.push(
            "Nginx config preview is available, but Forge Env does not rewrite nginx.conf yet."
                .into(),
        );
    }

    ServiceState {
        name: spec.name.into(),
        kind: spec.kind.into(),
        installed,
        running,
        health: if !installed {
            "missing".into()
        } else if running {
            "good".into()
        } else {
            "attention".into()
        },
        version,
        manager,
        port,
        data_dir,
        notes,
    }
}

fn detect_service_config_for_host(host_id: &str, spec: &ServiceSpec) -> ServiceConfigState {
    let (config_path, port, data_dir, can_edit_port, can_edit_data_dir, notes) = match spec.name {
        "Redis" => {
            let config_path = resolve_redis_config_path(host_id);
            let port = redis_port(host_id).or_else(|| {
                config_path
                    .as_ref()
                    .and_then(|path| config_line_value(host_id, path, "port"))
                    .and_then(|value| value.parse::<u16>().ok())
            });
            let data_dir = redis_data_dir(host_id).or_else(|| {
                config_path
                    .as_ref()
                    .and_then(|path| config_line_value(host_id, path, "dir"))
            });
            let mut notes = vec![
                "Redis port and data dir can be managed through a Forge Env override block.".into(),
            ];
            if config_path.is_none() {
                notes.push(
                    "No existing redis.conf was detected; Forge Env will create a managed config under ~/.forge-env/services/redis if you apply one."
                        .into(),
                );
            }
            (config_path, port, data_dir, true, true, notes)
        }
        "PostgreSQL" => {
            let config_path = resolve_postgresql_config_path(host_id);
            let port = postgresql_port(host_id).or_else(|| {
                config_path
                    .as_ref()
                    .and_then(|path| config_assignment_value(host_id, path, "port"))
                    .and_then(|value| value.parse::<u16>().ok())
            });
            let data_dir =
                postgresql_data_dir(host_id).or_else(|| postgres_managed_data_dir(host_id));
            let mut notes = vec![
                "Forge Env can append a managed port override to postgresql.conf.".into(),
                "Data directory migration is not automated yet; treat it as read-only here.".into(),
            ];
            if config_path.is_none() {
                notes.push(
                    "No postgresql.conf was found. Start PostgreSQL once or use the managed direct cluster path first."
                        .into(),
                );
            }
            (config_path, port, data_dir, true, false, notes)
        }
        "MySQL" => {
            let config_path = resolve_mysql_config_path(host_id);
            let port = mysql_port(host_id).or_else(|| {
                config_path
                    .as_ref()
                    .and_then(|path| mysql_port_from_config(host_id, path))
            });
            let mut notes = vec![
                "Forge Env can append a managed port override under a [mysqld] section.".into(),
                "Data directory changes are not automated yet because they usually require migration and permissions work."
                    .into(),
            ];
            if config_path.is_none() {
                notes.push(
                    "No writable MySQL config file was detected. Use your system package defaults or create one before applying overrides."
                        .into(),
                );
            }
            (config_path, port, None, true, false, notes)
        }
        "MongoDB" => {
            let config_path = resolve_mongodb_config_path(host_id);
            let port = mongodb_port(host_id).or_else(|| {
                config_path
                    .as_ref()
                    .and_then(|path| yaml_scalar_value(host_id, path, "port"))
                    .and_then(|value| value.parse::<u16>().ok())
            });
            let data_dir = mongodb_data_dir(host_id).or_else(|| {
                config_path
                    .as_ref()
                    .and_then(|path| yaml_scalar_value(host_id, path, "dbPath"))
            });
            let mut notes = vec![
                "Forge Env currently exposes MongoDB config as read-only because mongod.conf needs structured YAML updates."
                    .into(),
            ];
            if config_path.is_none() {
                notes.push(
                    "No mongod.conf was detected on this host; preview values fall back to live process defaults."
                        .into(),
                );
            }
            (config_path, port, data_dir, false, false, notes)
        }
        "RabbitMQ" => {
            let config_path = resolve_rabbitmq_config_path(host_id);
            let data_dir = rabbitmq_data_dir(host_id);
            let mut notes = vec![
                "RabbitMQ config is preview-only for now; Forge Env does not rewrite rabbitmq.conf yet."
                    .into(),
            ];
            if config_path.is_none() {
                notes.push(
                    "No rabbitmq.conf was found among common locations for this host.".into(),
                );
            }
            (config_path, Some(5672), data_dir, false, false, notes)
        }
        "Nginx" => {
            let config_path = resolve_nginx_config_path(host_id);
            let port = nginx_port(host_id).or_else(|| {
                config_path
                    .as_ref()
                    .and_then(|path| nginx_port_from_config(host_id, path))
            });
            let mut notes = vec![
                "Nginx config is preview-only for now; server block editing and reload flows are follow-up work."
                    .into(),
            ];
            if config_path.is_none() {
                notes.push("No nginx.conf was detected in common host locations.".into());
            }
            (config_path, port, None, false, false, notes)
        }
        _ => (None, None, None, false, false, Vec::new()),
    };

    ServiceConfigState {
        service_name: spec.name.into(),
        config_path,
        port,
        data_dir,
        can_edit_port,
        can_edit_data_dir,
        notes,
    }
}

fn supported_snapshot_services() -> Vec<&'static str> {
    vec!["Redis", "PostgreSQL", "MySQL"]
}

fn ensure_snapshot_capable_service(name: &str) -> Result<(), ServiceError> {
    if supported_snapshot_services().contains(&name) {
        Ok(())
    } else {
        Err(ServiceError::new(
            ServiceErrorKind::Unsupported,
            format!(
                "{} snapshot backup and restore are only available for Redis, PostgreSQL, and MySQL right now.",
                name
            ),
        ))
    }
}

fn require_service_with_data_dir(host_id: &str, name: &str) -> Result<ServiceState, ServiceError> {
    let service = detect_services_for_host(host_id)
        .into_iter()
        .find(|service| service.name == name)
        .ok_or_else(|| {
            ServiceError::new(
                ServiceErrorKind::Unsupported,
                format!("Unsupported service: {name}"),
            )
        })?;
    if !service.installed {
        return Err(ServiceError::new(
            ServiceErrorKind::ToolUnavailable,
            format!("{name} is not installed on {host_id}."),
        ));
    }
    if service
        .data_dir
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        return Err(ServiceError::new(
            ServiceErrorKind::PathUnavailable,
            format!(
                "Forge Env could not determine a data directory for {} on {}.",
                name, host_id
            ),
        ));
    }

    Ok(service)
}

fn ensure_snapshot_safe_state(service: &ServiceState) -> Result<(), ServiceError> {
    if service.running {
        return Err(ServiceError::new(
            ServiceErrorKind::Unsupported,
            format!(
                "{} must be stopped before Forge Env creates or restores a data-dir snapshot archive.",
                service.name
            ),
        ));
    }

    Ok(())
}

fn managed_service_artifact_path(
    host_id: &str,
    service_name: &str,
    kind: &str,
) -> Result<PathBuf, ServiceError> {
    let service_dir = managed_service_artifact_dir(host_id, service_name, kind)?;
    let file_name = format!(
        "{}-{}-{}.tar.gz",
        slug_service_name(service_name),
        kind,
        unix_timestamp()
    );
    Ok(service_dir.join(file_name))
}

fn managed_service_artifact_dir(
    host_id: &str,
    service_name: &str,
    kind: &str,
) -> Result<PathBuf, ServiceError> {
    let home = hosts::home_dir_for_host(host_id).ok_or_else(|| {
        ServiceError::new(
            ServiceErrorKind::HostUnavailable,
            format!("Forge Env could not resolve a home directory for {host_id}."),
        )
    })?;
    let dir = PathBuf::from(home)
        .join(ARTIFACT_DIR)
        .join(slug_service_name(service_name))
        .join(kind);
    hosts::create_dir_all_on_host(host_id, &dir).map_err(ServiceError::classify)?;
    Ok(dir)
}

fn list_service_artifacts_for_kind(
    host_id: &str,
    service_name: &str,
    kind: &str,
) -> Vec<ServiceArtifact> {
    let directory = match managed_service_artifact_dir(host_id, service_name, kind) {
        Ok(directory) => directory,
        Err(_) => return Vec::new(),
    };
    let quoted = shell_quote(directory.to_string_lossy().as_ref());
    let script = format!(
        "if [ -d '{quoted}' ]; then find '{quoted}' -maxdepth 1 -type f \\( -name '*.tar.gz' -o -name '*.tgz' \\) | while IFS= read -r file; do size=$(wc -c < \"$file\" | tr -d ' '); modified=$(stat -f %m \"$file\" 2>/dev/null || stat -c %Y \"$file\" 2>/dev/null || echo 0); printf '%s|%s|%s\\n' \"$file\" \"$modified\" \"$size\"; done; fi"
    );
    let output = hosts::run_shell_script_on_host(host_id, &script)
        .ok()
        .map(|result| result.stdout)
        .unwrap_or_default();
    let mut artifacts = output
        .lines()
        .filter_map(|line| parse_service_artifact_line(service_name, kind, line))
        .collect::<Vec<_>>();
    artifacts.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    artifacts
}

fn latest_managed_service_backup(host_id: &str, service_name: &str) -> Option<PathBuf> {
    list_service_artifacts_for_kind(host_id, service_name, "backup")
        .into_iter()
        .max_by(|left, right| left.created_at.cmp(&right.created_at))
        .map(|artifact| PathBuf::from(artifact.path))
}

fn ensure_managed_artifact_path(host_id: &str, path: &Path) -> Result<(), ServiceError> {
    let home = hosts::home_dir_for_host(host_id).ok_or_else(|| {
        ServiceError::new(
            ServiceErrorKind::HostUnavailable,
            format!("Forge Env could not resolve a home directory for {host_id}."),
        )
    })?;
    let managed_root = PathBuf::from(home).join(ARTIFACT_DIR);
    let candidate = path.to_string_lossy();
    let root = managed_root.to_string_lossy();
    if candidate.starts_with(root.as_ref()) {
        Ok(())
    } else {
        Err(ServiceError::new(
            ServiceErrorKind::Unsupported,
            format!(
                "Forge Env only deletes artifacts inside its managed root: {}",
                managed_root.to_string_lossy()
            ),
        ))
    }
}

fn parse_service_artifact_line(
    service_name: &str,
    kind: &str,
    line: &str,
) -> Option<ServiceArtifact> {
    let mut parts = line.split('|');
    let path = parts.next()?.trim();
    if path.is_empty() {
        return None;
    }
    let created_at = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let size_bytes = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<u64>().ok());
    Some(ServiceArtifact {
        service_name: service_name.into(),
        kind: kind.into(),
        path: path.into(),
        size_bytes,
        created_at: created_at.map(ToOwned::to_owned),
        managed: true,
    })
}

fn write_service_archive(
    host_id: &str,
    service_name: &str,
    data_dir: &str,
    archive_path: &Path,
) -> Result<(), ServiceError> {
    let archive = shell_quote(archive_path.to_string_lossy().as_ref());
    let source = shell_quote(data_dir);
    let script = format!(
        "set -e; [ -d '{source}' ] || {{ echo '{service_name} data directory is missing: {source}' >&2; exit 1; }}; mkdir -p \"$(dirname '{archive}')\"; tar -czf '{archive}' -C '{source}' ."
    );
    hosts::run_shell_script_on_host(host_id, &script)
        .map(|_| ())
        .map_err(ServiceError::classify)
}

fn write_postgresql_logical_backup(host_id: &str, archive_path: &Path) -> Result<(), ServiceError> {
    if !has_command(host_id, "pg_dumpall") {
        return Err(ServiceError::new(
            ServiceErrorKind::ToolUnavailable,
            "pg_dumpall is not available on the selected host.",
        ));
    }
    let archive = shell_quote(archive_path.to_string_lossy().as_ref());
    let script = format!(
        "set -e; mkdir -p \"$(dirname '{archive}')\"; pg_dumpall --clean --if-exists | gzip -c > '{archive}'"
    );
    hosts::run_shell_script_on_host(host_id, &script)
        .map(|_| ())
        .map_err(ServiceError::classify)
}

fn write_mysql_logical_backup(host_id: &str, archive_path: &Path) -> Result<(), ServiceError> {
    if !has_command(host_id, "mysqldump") {
        return Err(ServiceError::new(
            ServiceErrorKind::ToolUnavailable,
            "mysqldump is not available on the selected host.",
        ));
    }
    let archive = shell_quote(archive_path.to_string_lossy().as_ref());
    let script = format!(
        "set -e; mkdir -p \"$(dirname '{archive}')\"; mysqldump --all-databases --routines --events --triggers | gzip -c > '{archive}'"
    );
    hosts::run_shell_script_on_host(host_id, &script)
        .map(|_| ())
        .map_err(ServiceError::classify)
}

fn restore_service_archive(
    host_id: &str,
    archive_path: &Path,
    data_dir: &Path,
    restore_backup: &Path,
) -> Result<(), ServiceError> {
    let archive = shell_quote(archive_path.to_string_lossy().as_ref());
    let target = shell_quote(data_dir.to_string_lossy().as_ref());
    let previous = shell_quote(restore_backup.to_string_lossy().as_ref());
    let script = format!(
        "set -e; [ -f '{archive}' ] || {{ echo 'Archive not found: {archive}' >&2; exit 1; }}; parent=\"$(dirname '{target}')\"; mkdir -p \"$parent\"; if [ -d '{target}' ]; then mv '{target}' '{previous}'; fi; mkdir -p '{target}'; tar -xzf '{archive}' -C '{target}'"
    );
    hosts::run_shell_script_on_host(host_id, &script)
        .map(|_| ())
        .map_err(ServiceError::classify)
}

fn slug_service_name(service_name: &str) -> String {
    service_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn shell_quote(value: &str) -> String {
    value.replace('\'', "'\"'\"'")
}

fn redis_running(host_id: &str) -> bool {
    hosts::run_capture_on_host(host_id, "redis-cli", &["ping"])
        .map(|output| output.contains("PONG"))
        .unwrap_or_else(|| script_true(host_id, "pgrep -x redis-server >/dev/null 2>&1"))
}

fn postgresql_running(host_id: &str) -> bool {
    hosts::run_capture_on_host(host_id, "pg_isready", &[])
        .map(|output| output.contains("accepting connections"))
        .unwrap_or_else(|| {
            script_true(
                host_id,
                "pgrep -x postgres >/dev/null 2>&1 || pgrep -x postmaster >/dev/null 2>&1",
            )
        })
}

fn mysql_running(host_id: &str) -> bool {
    script_true(
        host_id,
        "pgrep -x mysqld >/dev/null 2>&1 || pgrep -x mariadbd >/dev/null 2>&1",
    )
}

fn mongodb_running(host_id: &str) -> bool {
    script_true(host_id, "pgrep -x mongod >/dev/null 2>&1")
}

fn rabbitmq_running(host_id: &str) -> bool {
    hosts::run_capture_on_host(host_id, "rabbitmq-diagnostics", &["check_running"])
        .map(|output| output.contains("Ping succeeded") || output.contains("RabbitMQ on node"))
        .unwrap_or_else(|| script_true(host_id, "pgrep -f rabbitmq-server >/dev/null 2>&1"))
}

fn nginx_running(host_id: &str) -> bool {
    script_true(host_id, "pgrep -x nginx >/dev/null 2>&1")
}

fn mysql_port(host_id: &str) -> Option<u16> {
    hosts::run_capture_on_host(host_id, "mysqladmin", &["variables"]).and_then(|output| {
        output
            .lines()
            .find(|line| line.contains("| port"))
            .and_then(|line| line.split('|').next_back())
            .map(str::trim)
            .and_then(|value| value.parse::<u16>().ok())
    })
}

fn mongodb_port(host_id: &str) -> Option<u16> {
    hosts::run_capture_on_host(
        host_id,
        "mongosh",
        &[
            "--quiet",
            "--eval",
            "db.adminCommand({getCmdLineOpts:1}).parsed.net.port",
        ],
    )
    .or_else(|| {
        hosts::run_capture_on_host(
            host_id,
            "mongo",
            &[
                "--quiet",
                "--eval",
                "db.adminCommand({getCmdLineOpts:1}).parsed.net.port",
            ],
        )
    })
    .and_then(|value| {
        value
            .lines()
            .last()
            .map(str::trim)
            .and_then(|port| port.parse::<u16>().ok())
    })
}

fn redis_port(host_id: &str) -> Option<u16> {
    config_get_last_value(host_id, "redis-cli", &["config", "get", "port"])
        .and_then(|value| value.parse::<u16>().ok())
}

fn redis_data_dir(host_id: &str) -> Option<String> {
    config_get_last_value(host_id, "redis-cli", &["config", "get", "dir"])
}

fn postgresql_port(host_id: &str) -> Option<u16> {
    hosts::run_capture_on_host(host_id, "psql", &["-Atqc", "show port"])
        .and_then(|value| value.trim().parse::<u16>().ok())
}

fn postgresql_data_dir(host_id: &str) -> Option<String> {
    hosts::run_capture_on_host(host_id, "psql", &["-Atqc", "show data_directory"])
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn mysql_data_dir(host_id: &str) -> Option<String> {
    hosts::run_capture_on_host(host_id, "mysqladmin", &["variables"])
        .and_then(|output| {
            output
                .lines()
                .find(|line| line.contains("| datadir"))
                .and_then(|line| line.split('|').next_back())
                .map(str::trim)
                .map(ToOwned::to_owned)
        })
        .filter(|value| !value.is_empty())
}

fn mongodb_data_dir(host_id: &str) -> Option<String> {
    hosts::run_capture_on_host(
        host_id,
        "mongosh",
        &[
            "--quiet",
            "--eval",
            "db.adminCommand({getCmdLineOpts:1}).parsed.storage.dbPath",
        ],
    )
    .or_else(|| {
        hosts::run_capture_on_host(
            host_id,
            "mongo",
            &[
                "--quiet",
                "--eval",
                "db.adminCommand({getCmdLineOpts:1}).parsed.storage.dbPath",
            ],
        )
    })
    .and_then(|value| {
        value
            .lines()
            .last()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
    })
    .or_else(|| {
        [
            "/opt/homebrew/var/mongodb",
            "/usr/local/var/mongodb",
            "/var/lib/mongodb",
            "/data/db",
        ]
        .iter()
        .find(|candidate| hosts::path_exists_on_host(host_id, Path::new(candidate)))
        .map(|candidate| (*candidate).to_string())
    })
}

fn rabbitmq_data_dir(host_id: &str) -> Option<String> {
    hosts::run_capture_on_host(host_id, "rabbitmqctl", &["status"])
        .and_then(|output| {
            output.lines().find_map(|line| {
                line.contains("data_dir")
                    .then(|| line.split('"').nth(1).map(ToOwned::to_owned))
                    .flatten()
            })
        })
        .or_else(|| {
            [
                "/opt/homebrew/var/lib/rabbitmq",
                "/usr/local/var/lib/rabbitmq",
                "/var/lib/rabbitmq",
            ]
            .iter()
            .find(|candidate| hosts::path_exists_on_host(host_id, Path::new(candidate)))
            .map(|candidate| (*candidate).to_string())
        })
}

fn start_redis_direct(host_id: &str) -> Result<String, ServiceError> {
    if let Some(config_path) = resolve_redis_config_path(host_id) {
        hosts::run_checked_on_host(
            host_id,
            "redis-server",
            &[&config_path, "--daemonize", "yes"],
        )
        .map_err(ServiceError::classify)?;
        return Ok(format!(
            "Started Redis directly on {host_id} using config {}.",
            config_path
        ));
    }
    hosts::run_checked_on_host(host_id, "redis-server", &["--daemonize", "yes"])
        .map_err(ServiceError::classify)?;
    Ok(format!(
        "Started Redis directly on {host_id} via redis-server --daemonize yes."
    ))
}

fn stop_redis_direct(host_id: &str) -> Result<String, ServiceError> {
    hosts::run_checked_on_host(host_id, "redis-cli", &["shutdown"])
        .map_err(ServiceError::classify)?;
    Ok(format!(
        "Stopped Redis directly on {host_id} via redis-cli shutdown."
    ))
}

fn restart_redis_direct(host_id: &str) -> Result<String, ServiceError> {
    let _ = stop_redis_direct(host_id);
    start_redis_direct(host_id)?;
    Ok(format!("Restarted Redis directly on {host_id}."))
}

fn start_postgresql_direct(host_id: &str) -> Result<String, ServiceError> {
    let home = hosts::home_dir_for_host(host_id).ok_or_else(|| {
        ServiceError::new(
            ServiceErrorKind::HostUnavailable,
            format!("Cannot resolve home directory for host {host_id}."),
        )
    })?;
    let service_root = PathBuf::from(home)
        .join(".forge-env")
        .join("services")
        .join("postgresql");
    let data_dir = service_root.join("data");
    let log_dir = service_root.join("logs");
    let log_path = log_dir.join("server.log");
    hosts::create_dir_all_on_host(host_id, &data_dir).map_err(ServiceError::classify)?;
    hosts::create_dir_all_on_host(host_id, &log_dir).map_err(ServiceError::classify)?;

    let init_script = format!(
        "if [ ! -f '{pg_version}' ]; then initdb -D '{data_dir}'; fi && pg_ctl -D '{data_dir}' -l '{log_path}' start",
        pg_version = data_dir.join("PG_VERSION").to_string_lossy(),
        data_dir = data_dir.to_string_lossy(),
        log_path = log_path.to_string_lossy(),
    );
    hosts::run_shell_script_on_host(host_id, &init_script).map_err(ServiceError::classify)?;
    Ok(format!(
        "Started PostgreSQL directly on {host_id} using managed data dir {}.",
        data_dir.to_string_lossy()
    ))
}

fn stop_postgresql_direct(host_id: &str) -> Result<String, ServiceError> {
    let data_dir = postgres_managed_data_dir(host_id).ok_or_else(|| {
        ServiceError::new(
            ServiceErrorKind::PathUnavailable,
            format!("No managed PostgreSQL data dir found for host {host_id}."),
        )
    })?;
    hosts::run_shell_script_on_host(host_id, &format!("pg_ctl -D '{}' stop -m fast", data_dir))
        .map_err(ServiceError::classify)?;
    Ok(format!("Stopped PostgreSQL directly on {host_id}."))
}

fn restart_postgresql_direct(host_id: &str) -> Result<String, ServiceError> {
    let data_dir = postgres_managed_data_dir(host_id).ok_or_else(|| {
        ServiceError::new(
            ServiceErrorKind::PathUnavailable,
            format!("No managed PostgreSQL data dir found for host {host_id}."),
        )
    })?;
    hosts::run_shell_script_on_host(
        host_id,
        &format!("pg_ctl -D '{}' restart -m fast", data_dir),
    )
    .map_err(ServiceError::classify)?;
    Ok(format!("Restarted PostgreSQL directly on {host_id}."))
}

fn postgres_managed_data_dir(host_id: &str) -> Option<String> {
    let home = hosts::home_dir_for_host(host_id)?;
    let candidate = PathBuf::from(home)
        .join(".forge-env")
        .join("services")
        .join("postgresql")
        .join("data");
    hosts::path_exists_on_host(host_id, &candidate).then(|| candidate.to_string_lossy().to_string())
}

fn resolve_redis_config_path(host_id: &str) -> Option<String> {
    let mut candidates = vec![
        PathBuf::from("/opt/homebrew/etc/redis.conf"),
        PathBuf::from("/usr/local/etc/redis.conf"),
        PathBuf::from("/etc/redis/redis.conf"),
        PathBuf::from("/etc/redis.conf"),
    ];
    if let Some(home) = hosts::home_dir_for_host(host_id) {
        candidates.insert(
            0,
            PathBuf::from(home)
                .join(".forge-env")
                .join("services")
                .join("redis")
                .join("redis.conf"),
        );
    }
    first_existing_or_managed(host_id, &candidates)
}

fn resolve_postgresql_config_path(host_id: &str) -> Option<String> {
    if let Some(path) = hosts::run_capture_on_host(host_id, "psql", &["-Atqc", "show config_file"])
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Some(path);
    }

    let mut candidates = vec![
        PathBuf::from("/opt/homebrew/var/postgresql@17/postgresql.conf"),
        PathBuf::from("/opt/homebrew/var/postgresql@16/postgresql.conf"),
        PathBuf::from("/opt/homebrew/var/postgresql@15/postgresql.conf"),
        PathBuf::from("/usr/local/var/postgresql@17/postgresql.conf"),
        PathBuf::from("/usr/local/var/postgresql@16/postgresql.conf"),
        PathBuf::from("/usr/local/var/postgresql@15/postgresql.conf"),
        PathBuf::from("/usr/local/var/postgres/postgresql.conf"),
        PathBuf::from("/etc/postgresql/17/main/postgresql.conf"),
        PathBuf::from("/etc/postgresql/16/main/postgresql.conf"),
        PathBuf::from("/etc/postgresql/15/main/postgresql.conf"),
        PathBuf::from("/etc/postgresql/14/main/postgresql.conf"),
    ];

    if let Some(data_dir) = postgres_managed_data_dir(host_id) {
        candidates.insert(0, PathBuf::from(data_dir).join("postgresql.conf"));
    }

    first_existing(host_id, &candidates)
}

fn resolve_mysql_config_path(host_id: &str) -> Option<String> {
    let candidates = vec![
        PathBuf::from("/opt/homebrew/etc/my.cnf"),
        PathBuf::from("/usr/local/etc/my.cnf"),
        PathBuf::from("/etc/my.cnf"),
        PathBuf::from("/etc/mysql/my.cnf"),
        PathBuf::from("/etc/mysql/mysql.conf.d/mysqld.cnf"),
    ];
    first_existing(host_id, &candidates)
}

fn resolve_mongodb_config_path(host_id: &str) -> Option<String> {
    let candidates = vec![
        PathBuf::from("/opt/homebrew/etc/mongod.conf"),
        PathBuf::from("/usr/local/etc/mongod.conf"),
        PathBuf::from("/etc/mongod.conf"),
        PathBuf::from("/etc/mongodb.conf"),
        PathBuf::from("/etc/mongodb/mongod.conf"),
    ];
    first_existing(host_id, &candidates)
}

fn resolve_rabbitmq_config_path(host_id: &str) -> Option<String> {
    let candidates = vec![
        PathBuf::from("/opt/homebrew/etc/rabbitmq/rabbitmq.conf"),
        PathBuf::from("/usr/local/etc/rabbitmq/rabbitmq.conf"),
        PathBuf::from("/etc/rabbitmq/rabbitmq.conf"),
    ];
    first_existing(host_id, &candidates)
}

fn resolve_nginx_config_path(host_id: &str) -> Option<String> {
    hosts::run_capture_on_host(host_id, "nginx", &["-V"])
        .and_then(|output| {
            output
                .split_whitespace()
                .find_map(|token| token.strip_prefix("--conf-path=").map(ToOwned::to_owned))
        })
        .or_else(|| {
            let candidates = vec![
                PathBuf::from("/opt/homebrew/etc/nginx/nginx.conf"),
                PathBuf::from("/usr/local/etc/nginx/nginx.conf"),
                PathBuf::from("/etc/nginx/nginx.conf"),
            ];
            first_existing(host_id, &candidates)
        })
}

fn apply_redis_config(
    host_id: &str,
    port: Option<u16>,
    data_dir: Option<String>,
) -> Result<String, ServiceError> {
    let config_path = resolve_redis_config_path(host_id).ok_or_else(|| {
        ServiceError::new(
            ServiceErrorKind::PathUnavailable,
            "Could not determine a Redis config path for this host.",
        )
    })?;
    let path = PathBuf::from(&config_path);
    if let Some(parent) = path.parent() {
        hosts::create_dir_all_on_host(host_id, parent).map_err(ServiceError::classify)?;
    }
    let existing = hosts::read_file_on_host(host_id, &path).map_err(ServiceError::classify)?;
    let managed = format!(
        "{CONFIG_BEGIN}\n{}{}{}\n{CONFIG_END}\n",
        port.map(|value| format!("port {value}\n"))
            .unwrap_or_default(),
        data_dir
            .as_ref()
            .map(|value| format!("dir {value}\n"))
            .unwrap_or_default(),
        ""
    );
    let updated = replace_managed_block(&existing, &managed);
    hosts::write_file_on_host(host_id, &path, &updated).map_err(ServiceError::classify)?;
    Ok(format!(
        "Updated Redis config on {} at {}. Restart Redis to apply the new settings.",
        host_id, config_path
    ))
}

fn apply_postgresql_config(
    host_id: &str,
    port: Option<u16>,
    data_dir: Option<String>,
) -> Result<String, ServiceError> {
    if data_dir.is_some() {
        return Err(ServiceError::new(
            ServiceErrorKind::Unsupported,
            "PostgreSQL data directory migration is not automated yet. Use the detected data dir as read-only for now.",
        ));
    }
    let config_path = resolve_postgresql_config_path(host_id).ok_or_else(|| {
        ServiceError::new(
            ServiceErrorKind::PathUnavailable,
            "Could not determine a PostgreSQL config path for this host.",
        )
    })?;
    let path = PathBuf::from(&config_path);
    let existing = hosts::read_file_on_host(host_id, &path).map_err(ServiceError::classify)?;
    let managed = format!(
        "{CONFIG_BEGIN}\n{}{CONFIG_END}\n",
        port.map(|value| format!("port = {value}\n"))
            .unwrap_or_default(),
    );
    let updated = replace_managed_block(&existing, &managed);
    hosts::write_file_on_host(host_id, &path, &updated).map_err(ServiceError::classify)?;
    Ok(format!(
        "Updated PostgreSQL config on {} at {}. Restart PostgreSQL to apply the new settings.",
        host_id, config_path
    ))
}

fn apply_mysql_config(
    host_id: &str,
    port: Option<u16>,
    data_dir: Option<String>,
) -> Result<String, ServiceError> {
    if data_dir.is_some() {
        return Err(ServiceError::new(
            ServiceErrorKind::Unsupported,
            "MySQL data directory changes are not automated yet because they usually require migration and permission fixes.",
        ));
    }
    let config_path = resolve_mysql_config_path(host_id).ok_or_else(|| {
        ServiceError::new(
            ServiceErrorKind::PathUnavailable,
            "Could not determine a MySQL config path for this host.",
        )
    })?;
    let path = PathBuf::from(&config_path);
    let existing = hosts::read_file_on_host(host_id, &path).map_err(ServiceError::classify)?;
    let managed = format!(
        "{CONFIG_BEGIN}\n[mysqld]\n{}{CONFIG_END}\n",
        port.map(|value| format!("port={value}\n"))
            .unwrap_or_default(),
    );
    let updated = replace_managed_block(&existing, &managed);
    hosts::write_file_on_host(host_id, &path, &updated).map_err(ServiceError::classify)?;
    Ok(format!(
        "Updated MySQL config on {} at {}. Restart MySQL to apply the new settings.",
        host_id, config_path
    ))
}

fn config_get_last_value(host_id: &str, command: &str, args: &[&str]) -> Option<String> {
    hosts::run_capture_on_host(host_id, command, args).and_then(|output| {
        output
            .lines()
            .last()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn config_line_value(host_id: &str, path: &str, key: &str) -> Option<String> {
    hosts::read_file_on_host(host_id, Path::new(path))
        .ok()
        .and_then(|contents| {
            contents.lines().find_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.starts_with(';') {
                    return None;
                }
                let mut parts = trimmed.split_whitespace();
                let name = parts.next()?;
                (name == key)
                    .then(|| parts.collect::<Vec<_>>().join(" ").trim().to_string())
                    .filter(|value| !value.is_empty())
            })
        })
}

fn config_assignment_value(host_id: &str, path: &str, key: &str) -> Option<String> {
    hosts::read_file_on_host(host_id, Path::new(path))
        .ok()
        .and_then(|contents| {
            contents.lines().find_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.starts_with(';') {
                    return None;
                }
                let (name, value) = trimmed.split_once('=')?;
                (name.trim() == key).then(|| value.trim().trim_matches('\'').to_string())
            })
        })
}

fn mysql_port_from_config(host_id: &str, path: &str) -> Option<u16> {
    config_assignment_value(host_id, path, "port").and_then(|value| value.parse::<u16>().ok())
}

fn nginx_port(host_id: &str) -> Option<u16> {
    resolve_nginx_config_path(host_id)
        .as_deref()
        .and_then(|path| nginx_port_from_config(host_id, path))
}

fn nginx_port_from_config(host_id: &str, path: &str) -> Option<u16> {
    hosts::read_file_on_host(host_id, Path::new(path))
        .ok()
        .and_then(|contents| {
            contents.lines().find_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('#') {
                    return None;
                }
                let rest = trimmed.strip_prefix("listen ")?;
                rest.split_whitespace()
                    .next()
                    .map(|value| value.trim_end_matches(';'))
                    .and_then(|value| value.parse::<u16>().ok())
            })
        })
}

fn yaml_scalar_value(host_id: &str, path: &str, key: &str) -> Option<String> {
    hosts::read_file_on_host(host_id, Path::new(path))
        .ok()
        .and_then(|contents| {
            contents.lines().find_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('#') {
                    return None;
                }
                let (name, value) = trimmed.split_once(':')?;
                (name.trim() == key).then(|| {
                    value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string()
                })
            })
        })
}

fn first_line(output: Option<String>) -> Option<String> {
    output.and_then(|value| {
        value
            .lines()
            .next()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn has_command(host_id: &str, command: &str) -> bool {
    script_true(host_id, &format!("command -v {command} >/dev/null 2>&1"))
}

fn first_existing(host_id: &str, candidates: &[PathBuf]) -> Option<String> {
    candidates
        .iter()
        .find(|candidate| hosts::path_exists_on_host(host_id, candidate))
        .map(|candidate| candidate.to_string_lossy().to_string())
}

fn first_existing_or_managed(host_id: &str, candidates: &[PathBuf]) -> Option<String> {
    first_existing(host_id, candidates).or_else(|| {
        candidates
            .first()
            .map(|candidate| candidate.to_string_lossy().to_string())
    })
}

fn replace_managed_block(existing: &str, managed_block: &str) -> String {
    if let (Some(begin), Some(end)) = (existing.find(CONFIG_BEGIN), existing.find(CONFIG_END)) {
        let after_end = end + CONFIG_END.len();
        let mut next = String::new();
        next.push_str(existing[..begin].trim_end());
        if !next.is_empty() {
            next.push_str("\n\n");
        }
        next.push_str(managed_block.trim_end());
        let tail = existing[after_end..].trim();
        if !tail.is_empty() {
            next.push_str("\n\n");
            next.push_str(tail);
        }
        next.push('\n');
        return next;
    }

    if existing.trim().is_empty() {
        return format!("{}\n", managed_block.trim_end());
    }

    format!("{}\n\n{}\n", existing.trim_end(), managed_block.trim_end())
}

fn script_true(host_id: &str, script: &str) -> bool {
    hosts::run_shell_script_on_host(host_id, script).is_ok()
}

fn summarize_action(
    service_name: &str,
    host_id: &str,
    action: ServiceAction,
    manager: &str,
    stdout: &str,
    stderr: &str,
) -> String {
    let detail = if !stdout.is_empty() { stdout } else { stderr };
    if detail.is_empty() {
        format!(
            "{} {} via {} on {}.",
            action_label(action),
            service_name,
            manager,
            host_id
        )
    } else {
        format!(
            "{} {} via {} on {}. {}",
            action_label(action),
            service_name,
            manager,
            host_id,
            detail.lines().next().unwrap_or(detail)
        )
    }
}

fn action_name(action: ServiceAction) -> &'static str {
    match action {
        ServiceAction::Start => "start",
        ServiceAction::Stop => "stop",
        ServiceAction::Restart => "restart",
    }
}

fn action_label(action: ServiceAction) -> &'static str {
    match action {
        ServiceAction::Start => "Started",
        ServiceAction::Stop => "Stopped",
        ServiceAction::Restart => "Restarted",
    }
}

fn unsupported_direct_action(name: &str, action: &str) -> String {
    format!(
        "{} does not have a direct {} fallback yet on this host. Use Homebrew services or the system service manager if available.",
        name, action
    )
}

#[cfg(test)]
mod tests {
    use super::{
        action_label, action_name, first_line, parse_service_artifact_line,
        replace_managed_block, shell_quote, slug_service_name, supported_snapshot_services,
        ServiceAction,
    };

    #[test]
    fn parses_service_artifact_line() {
        let artifact = parse_service_artifact_line(
            "Redis",
            "backup",
            "/tmp/redis-backup.tar.gz|1716288000|18432",
        )
        .expect("artifact");
        assert_eq!(artifact.service_name, "Redis");
        assert_eq!(artifact.kind, "backup");
        assert_eq!(artifact.path, "/tmp/redis-backup.tar.gz");
        assert_eq!(artifact.created_at.as_deref(), Some("1716288000"));
        assert_eq!(artifact.size_bytes, Some(18432));
        assert!(artifact.managed);
    }

    #[test]
    fn slugs_service_name() {
        assert_eq!(slug_service_name("PostgreSQL"), "postgresql");
        assert_eq!(slug_service_name("C/C++"), "c-c");
    }

    #[test]
    fn slug_service_name_edge_cases() {
        assert_eq!(slug_service_name(""), "");
        assert_eq!(slug_service_name("Redis"), "redis");
        assert_eq!(slug_service_name("C/C++"), "c-c");
        assert_eq!(slug_service_name("RabbitMQ"), "rabbitmq");
        assert_eq!(slug_service_name("  MongoDB  "), "mongodb");
        assert_eq!(slug_service_name("my-service"), "my-service");
        assert_eq!(
            slug_service_name("Service With Spaces"),
            "service-with-spaces"
        );
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("hello"), "hello");
        assert_eq!(shell_quote("it's"), "it'\"'\"'s");
        assert_eq!(shell_quote("a''b"), "a'\"'\"''\"'\"'b");
    }

    #[test]
    fn first_line_extracts_first_non_empty() {
        assert_eq!(first_line(None), None);
        assert_eq!(first_line(Some("".to_string())), None);
        assert_eq!(
            first_line(Some("  \nsecond".to_string())),
            None,
            "first line is whitespace-only so it is filtered"
        );
        assert_eq!(
            first_line(Some("first\nsecond".to_string())),
            Some("first".to_string())
        );
        assert_eq!(
            first_line(Some("  trimmed  ".to_string())),
            Some("trimmed".to_string())
        );
    }

    #[test]
    fn action_name_and_label() {
        assert_eq!(action_name(ServiceAction::Start), "start");
        assert_eq!(action_name(ServiceAction::Stop), "stop");
        assert_eq!(action_name(ServiceAction::Restart), "restart");
        assert_eq!(action_label(ServiceAction::Start), "Started");
        assert_eq!(action_label(ServiceAction::Stop), "Stopped");
        assert_eq!(action_label(ServiceAction::Restart), "Restarted");
    }

    #[test]
    fn ensure_snapshot_capable_service_valid() {
        let supported = supported_snapshot_services();
        assert!(supported.contains(&"Redis"));
        assert!(supported.contains(&"PostgreSQL"));
        assert!(supported.contains(&"MySQL"));
        assert_eq!(supported.len(), 3);
    }

    #[test]
    fn parse_service_artifact_line_edge_cases() {
        // Empty line returns None (empty path)
        assert!(parse_service_artifact_line("Redis", "backup", "").is_none());
        // Missing fields — only path, no timestamp or size
        let artifact = parse_service_artifact_line("Redis", "backup", "incomplete")
            .expect("still returns an artifact with just a path");
        assert_eq!(artifact.path, "incomplete");
        assert!(artifact.created_at.is_none());
        assert!(artifact.size_bytes.is_none());
        // Empty path after pipe
        assert!(parse_service_artifact_line("Redis", "backup", "|12345|1024").is_none());
        // Zero size
        let artifact = parse_service_artifact_line("Redis", "backup", "/tmp/test.tar.gz|12345|0")
            .expect("ok");
        assert_eq!(artifact.size_bytes, Some(0));
    }

    #[test]
    fn replace_managed_block_inserts_new_block() {
        let content = "existing config line\n";
        let block = "# >>> forge-env service >>>\ndata = true\n# <<< forge-env service <<<";
        let result = replace_managed_block(content, block);
        assert!(result.contains("# >>> forge-env service >>>"));
        assert!(result.contains("existing config line"));
    }

    #[test]
    fn replace_managed_block_replaces_existing() {
        let content =
            "line1\n# >>> forge-env service >>>\nold data\n# <<< forge-env service <<<\nline2";
        let block = "# >>> forge-env service >>>\nnew data\n# <<< forge-env service <<<";
        let result = replace_managed_block(content, block);
        assert!(result.contains("new data"));
        assert!(!result.contains("old data"));
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    #[test]
    fn replace_managed_block_empty_content() {
        let content = "";
        let block = "# >>> forge-env service >>>\ndata\n# <<< forge-env service <<<";
        let result = replace_managed_block(content, block);
        assert!(result.contains("# >>> forge-env service >>>"));
        assert!(result.contains("data"));
    }
}
