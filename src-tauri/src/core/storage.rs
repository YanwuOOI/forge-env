use std::{fs, path::PathBuf};

use rusqlite::{params, Connection};

use super::{models::JobRecord, shell};

#[derive(Debug, Clone, Default)]
pub struct PersistedStateSnapshot {
    pub jobs: Vec<JobRecord>,
    pub applied_mirror_preset: Option<String>,
    pub selected_host_id: Option<String>,
    pub last_env_target_profile: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Storage {
    db_path: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self, String> {
        let home = shell::home_dir().ok_or_else(|| "Cannot resolve home directory.".to_string())?;
        let dir = PathBuf::from(home).join(".forge-env");
        fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        let db_path = dir.join("forge-env.db");
        let storage = Self { db_path };
        storage.initialize()?;
        Ok(storage)
    }

    pub fn load(&self) -> Result<PersistedStateSnapshot, String> {
        let connection = self.connection()?;
        let mut snapshot = PersistedStateSnapshot::default();

        let mut settings = connection
            .prepare("SELECT key, value FROM settings")
            .map_err(|error| error.to_string())?;
        let setting_rows = settings
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| error.to_string())?;

        for row in setting_rows {
            let (key, value) = row.map_err(|error| error.to_string())?;
            match key.as_str() {
                "applied_mirror_preset" => snapshot.applied_mirror_preset = Some(value),
                "selected_host_id" => snapshot.selected_host_id = Some(value),
                "last_env_target_profile" => snapshot.last_env_target_profile = Some(value),
                _ => {}
            }
        }

        let mut jobs = connection
            .prepare(
                "SELECT id, label, status, family, version, category, target_name, outcome_title, outcome_detail, next_step, timestamp
                 FROM jobs
                 ORDER BY rowid DESC
                 LIMIT 12",
            )
            .map_err(|error| error.to_string())?;
        let job_rows = jobs
            .query_map([], |row| {
                Ok(JobRecord {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    status: row.get(2)?,
                    family: row.get(3)?,
                    version: row.get(4)?,
                    category: row.get(5)?,
                    target_name: row.get(6)?,
                    outcome_title: row.get(7)?,
                    outcome_detail: row.get(8)?,
                    next_step: row.get(9)?,
                    timestamp: row.get(10)?,
                })
            })
            .map_err(|error| error.to_string())?;

        for row in job_rows {
            snapshot.jobs.push(row.map_err(|error| error.to_string())?);
        }

        Ok(snapshot)
    }

    pub fn save_job(&self, job: &JobRecord) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT OR REPLACE INTO jobs (id, label, status, family, version, category, target_name, outcome_title, outcome_detail, next_step, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    job.id,
                    job.label,
                    job.status,
                    job.family,
                    job.version,
                    job.category,
                    job.target_name,
                    job.outcome_title,
                    job.outcome_detail,
                    job.next_step,
                    job.timestamp
                ],
            )
            .map_err(|error| error.to_string())?;
        connection
            .execute(
                "DELETE FROM jobs
                 WHERE id NOT IN (
                   SELECT id FROM jobs ORDER BY rowid DESC LIMIT 12
                 )",
                [],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn set_setting(&self, key: &str, value: Option<&str>) -> Result<(), String> {
        let connection = self.connection()?;
        match value {
            Some(value) => connection
                .execute(
                    "INSERT INTO settings (key, value) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    params![key, value],
                )
                .map_err(|error| error.to_string())?,
            None => connection
                .execute("DELETE FROM settings WHERE key = ?1", params![key])
                .map_err(|error| error.to_string())?,
        };
        Ok(())
    }

    fn initialize(&self) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS settings (
                  key TEXT PRIMARY KEY,
                  value TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS jobs (
                  id TEXT PRIMARY KEY,
                  label TEXT NOT NULL,
                  status TEXT NOT NULL,
                  family TEXT NULL,
                  version TEXT NULL,
                  category TEXT NULL,
                  target_name TEXT NULL,
                  outcome_title TEXT NULL,
                  outcome_detail TEXT NULL,
                  next_step TEXT NULL,
                  timestamp TEXT NOT NULL
                );
                ",
            )
            .map_err(|error| error.to_string())?;

        ensure_job_column(&connection, "category", "TEXT NULL")?;
        ensure_job_column(&connection, "target_name", "TEXT NULL")?;
        ensure_job_column(&connection, "outcome_title", "TEXT NULL")?;
        ensure_job_column(&connection, "outcome_detail", "TEXT NULL")?;
        ensure_job_column(&connection, "next_step", "TEXT NULL")?;

        Ok(())
    }

    fn connection(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path).map_err(|error| error.to_string())
    }
}

fn ensure_job_column(connection: &Connection, name: &str, declaration: &str) -> Result<(), String> {
    let pragma = format!("PRAGMA table_info(jobs)");
    let mut stmt = connection
        .prepare(&pragma)
        .map_err(|error| error.to_string())?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| error.to_string())?;
    let mut exists = false;
    for column in columns {
        if column.map_err(|error| error.to_string())? == name {
            exists = true;
            break;
        }
    }
    if !exists {
        connection
            .execute(
                &format!("ALTER TABLE jobs ADD COLUMN {name} {declaration}"),
                [],
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}
