use std::{fs, path::PathBuf};

use log::{error, warn};
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
        let home = shell::home_dir().ok_or_else(|| {
            error!("Cannot resolve home directory — HOME and USERPROFILE are unset");
            "Cannot resolve home directory.".to_string()
        })?;
        let dir = PathBuf::from(home).join(".forge-env");
        fs::create_dir_all(&dir).map_err(|error| {
            error!("Failed to create data directory {}: {error}", dir.display());
            error.to_string()
        })?;
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
                "SELECT id, label, status, family, version, category, target_name, outcome_title, outcome_detail, next_step, timestamp, progress, progress_label
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
                    progress: row.get(11)?,
                    progress_label: row.get(12)?,
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
                "INSERT OR REPLACE INTO jobs (id, label, status, family, version, category, target_name, outcome_title, outcome_detail, next_step, timestamp, progress, progress_label)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
                    job.timestamp,
                    job.progress,
                    job.progress_label
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
        ensure_job_column(&connection, "progress", "REAL NULL")?;
        ensure_job_column(&connection, "progress_label", "TEXT NULL")?;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_storage() -> Storage {
        let dir = std::env::temp_dir().join(format!("forge-env-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join(format!("test-{}.db", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let storage = Storage { db_path };
        storage.initialize().unwrap();
        storage
    }

    #[test]
    fn persisteda_state_snapshot_default() {
        let snapshot = PersistedStateSnapshot::default();
        assert!(snapshot.jobs.is_empty());
        assert!(snapshot.applied_mirror_preset.is_none());
        assert!(snapshot.selected_host_id.is_none());
        assert!(snapshot.last_env_target_profile.is_none());
    }

    #[test]
    fn schema_creates_tables() {
        let storage = temp_storage();
        let conn = storage.connection().unwrap();

        // Check settings table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='settings'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Check jobs table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='jobs'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn save_and_load_job_round_trip() {
        let storage = temp_storage();
        let job = JobRecord {
            id: "test-1".to_string(),
            label: "Install Python".to_string(),
            status: "completed".to_string(),
            family: Some("Python".to_string()),
            version: Some("3.12".to_string()),
            category: Some("runtime".to_string()),
            target_name: Some("Python".to_string()),
            outcome_title: Some("Installed".to_string()),
            outcome_detail: Some("Done".to_string()),
            next_step: Some("Activate".to_string()),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            progress: None,
            progress_label: None,
        };

        storage.save_job(&job).unwrap();
        let snapshot = storage.load().unwrap();
        assert_eq!(snapshot.jobs.len(), 1);
        assert_eq!(snapshot.jobs[0].id, "test-1");
        assert_eq!(snapshot.jobs[0].family.as_deref(), Some("Python"));
    }

    #[test]
    fn save_job_trims_to_12() {
        let storage = temp_storage();

        for i in 0..15 {
            let job = JobRecord {
                id: format!("job-{i}"),
                label: format!("Job {i}"),
                status: "completed".to_string(),
                family: None,
                version: None,
                category: None,
                target_name: None,
                outcome_title: None,
                outcome_detail: None,
                next_step: None,
                timestamp: format!("2024-01-{:02}", i + 1),
                progress: None,
                progress_label: None,
            };
            storage.save_job(&job).unwrap();
        }

        let snapshot = storage.load().unwrap();
        assert!(snapshot.jobs.len() <= 12, "should keep at most 12 jobs, got {}", snapshot.jobs.len());
    }

    #[test]
    fn set_setting_and_load() {
        let storage = temp_storage();

        storage.set_setting("test_key", Some("test_value")).unwrap();
        let snapshot = storage.load().unwrap();
        // Settings are loaded by key name, test_key won't match the known keys
        // But we can verify it was stored by checking raw
        let conn = storage.connection().unwrap();
        let value: String = conn
            .query_row("SELECT value FROM settings WHERE key = 'test_key'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "test_value");
    }

    #[test]
    fn set_setting_delete() {
        let storage = temp_storage();

        storage.set_setting("to_delete", Some("value")).unwrap();
        storage.set_setting("to_delete", None).unwrap();

        let conn = storage.connection().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM settings WHERE key = 'to_delete'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn ensure_job_column_adds_new_column() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE jobs (id TEXT PRIMARY KEY, label TEXT NOT NULL)")
            .unwrap();

        ensure_job_column(&conn, "category", "TEXT NULL").unwrap();

        // Verify column exists
        let mut stmt = conn.prepare("PRAGMA table_info(jobs)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(columns.contains(&"category".to_string()));
    }

    #[test]
    fn ensure_job_column_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE jobs (id TEXT PRIMARY KEY, label TEXT NOT NULL, category TEXT NULL)")
            .unwrap();

        // Should not error even though column already exists
        ensure_job_column(&conn, "category", "TEXT NULL").unwrap();
    }
}
