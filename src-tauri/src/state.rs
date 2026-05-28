use std::{
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::core::{models::JobRecord, storage::Storage};

pub struct SharedState {
    pub inner: Mutex<MemoryState>,
}

pub struct MemoryState {
    storage: Option<Storage>,
    pub jobs: Vec<JobRecord>,
    pub applied_mirror_preset: Option<String>,
    pub selected_host_id: Option<String>,
    pub last_env_target_profile: Option<String>,
}

impl SharedState {
    pub fn new() -> Self {
        let storage = Storage::new().ok();
        Self {
            inner: Mutex::new(MemoryState::seed(storage)),
        }
    }
}

impl MemoryState {
    fn seed(storage: Option<Storage>) -> Self {
        let persisted = storage
            .as_ref()
            .and_then(|storage| storage.load().ok())
            .unwrap_or_default();
        let mut jobs = persisted.jobs.clone();
        if jobs.is_empty() {
            jobs.push(JobRecord {
                id: "job-bootstrap".into(),
                label: "Bootstrap runtime inventory".into(),
                status: "completed".into(),
                family: None,
                version: None,
                category: None,
                target_name: None,
                outcome_title: None,
                outcome_detail: None,
                next_step: None,
                timestamp: timestamp(),
            });
        }

        Self {
            storage,
            jobs,
            applied_mirror_preset: persisted.applied_mirror_preset,
            selected_host_id: persisted.selected_host_id,
            last_env_target_profile: persisted.last_env_target_profile,
        }
    }

    pub fn jobs(&self) -> Vec<JobRecord> {
        self.jobs.clone()
    }

    pub fn applied_mirror_preset(&self) -> Option<String> {
        self.applied_mirror_preset.clone()
    }

    pub fn last_env_target_profile(&self) -> Option<String> {
        self.last_env_target_profile.clone()
    }

    pub fn selected_host_id(&self) -> Option<String> {
        self.selected_host_id.clone()
    }

    pub fn structured_job(
        &mut self,
        label: String,
        family: Option<&str>,
        version: Option<&str>,
        category: Option<&str>,
        target_name: Option<&str>,
        outcome_title: Option<&str>,
        outcome_detail: Option<&str>,
        next_step: Option<&str>,
    ) -> JobRecord {
        self.structured_job_with_status(
            "completed",
            label,
            family,
            version,
            category,
            target_name,
            outcome_title,
            outcome_detail,
            next_step,
        )
    }

    pub fn failed_job(
        &mut self,
        label: String,
        family: Option<&str>,
        version: Option<&str>,
        category: Option<&str>,
        target_name: Option<&str>,
        outcome_title: Option<&str>,
        outcome_detail: Option<&str>,
        next_step: Option<&str>,
    ) -> JobRecord {
        self.structured_job_with_status(
            "failed",
            label,
            family,
            version,
            category,
            target_name,
            outcome_title,
            outcome_detail,
            next_step,
        )
    }

    fn structured_job_with_status(
        &mut self,
        status: &str,
        label: String,
        family: Option<&str>,
        version: Option<&str>,
        category: Option<&str>,
        target_name: Option<&str>,
        outcome_title: Option<&str>,
        outcome_detail: Option<&str>,
        next_step: Option<&str>,
    ) -> JobRecord {
        self.push_job(
            status,
            label,
            family,
            version,
            category,
            target_name,
            outcome_title,
            outcome_detail,
            next_step,
        )
    }

    pub fn set_mirror_preset(&mut self, preset: &str) {
        self.applied_mirror_preset = Some(preset.to_string());
        self.persist_setting(
            "applied_mirror_preset",
            self.applied_mirror_preset.as_deref(),
        );
    }

    pub fn set_last_env_target_profile(&mut self, profile: &str) {
        self.last_env_target_profile = Some(profile.to_string());
        self.persist_setting(
            "last_env_target_profile",
            self.last_env_target_profile.as_deref(),
        );
    }

    pub fn set_selected_host_id(&mut self, host_id: &str) {
        self.selected_host_id = Some(host_id.to_string());
        self.persist_setting("selected_host_id", self.selected_host_id.as_deref());
    }

    fn push_job(
        &mut self,
        status: &str,
        label: String,
        family: Option<&str>,
        version: Option<&str>,
        category: Option<&str>,
        target_name: Option<&str>,
        outcome_title: Option<&str>,
        outcome_detail: Option<&str>,
        next_step: Option<&str>,
    ) -> JobRecord {
        let job = JobRecord {
            id: format!(
                "job-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            ),
            label,
            status: status.into(),
            family: family.map(ToOwned::to_owned),
            version: version.map(ToOwned::to_owned),
            category: category.map(ToOwned::to_owned),
            target_name: target_name.map(ToOwned::to_owned),
            outcome_title: outcome_title.map(ToOwned::to_owned),
            outcome_detail: outcome_detail.map(ToOwned::to_owned),
            next_step: next_step.map(ToOwned::to_owned),
            timestamp: timestamp(),
        };

        self.jobs.insert(0, job.clone());
        self.jobs.truncate(12);
        self.persist_job(&job);
        job
    }

    fn persist_job(&self, job: &JobRecord) {
        if let Some(storage) = &self.storage {
            let _ = storage.save_job(job);
        }
    }

    fn persist_setting(&self, key: &str, value: Option<&str>) {
        if let Some(storage) = &self.storage {
            let _ = storage.set_setting(key, value);
        }
    }
}

fn timestamp() -> String {
    format!(
        "{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    )
}
