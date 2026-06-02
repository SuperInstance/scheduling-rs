use serde::{Deserialize, Serialize};

/// A job to be scheduled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: usize,
    pub processing_time: f64,
    pub weight: f64,
    pub due_date: Option<f64>,
    pub release_date: Option<f64>,
    pub deadline: Option<f64>,
}

impl Job {
    pub fn new(id: usize, processing_time: f64) -> Self {
        Self {
            id,
            processing_time,
            weight: 1.0,
            due_date: None,
            release_date: None,
            deadline: None,
        }
    }

    pub fn with_weight(mut self, w: f64) -> Self {
        self.weight = w;
        self
    }

    pub fn with_due_date(mut self, d: f64) -> Self {
        self.due_date = Some(d);
        self
    }

    pub fn with_release_date(mut self, r: f64) -> Self {
        self.release_date = Some(r);
        self
    }

    pub fn with_deadline(mut self, d: f64) -> Self {
        self.deadline = Some(d);
        self
    }
}

/// A job that has been placed in a schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub job_id: usize,
    pub start_time: f64,
    pub end_time: f64,
    pub machine: usize,
}

/// A complete schedule across one or more machines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub scheduled_jobs: Vec<ScheduledJob>,
    pub num_machines: usize,
}

impl Schedule {
    pub fn new(num_machines: usize) -> Self {
        Self {
            scheduled_jobs: Vec::new(),
            num_machines,
        }
    }

    pub fn makespan(&self) -> f64 {
        self.scheduled_jobs
            .iter()
            .map(|j| j.end_time)
            .fold(0.0_f64, f64::max)
    }

    pub fn total_completion_time(&self) -> f64 {
        self.scheduled_jobs.iter().map(|j| j.end_time).sum()
    }

    pub fn total_weighted_completion_time(&self, jobs: &[Job]) -> f64 {
        let job_map: std::collections::HashMap<usize, f64> =
            jobs.iter().map(|j| (j.id, j.weight)).collect();
        self.scheduled_jobs
            .iter()
            .map(|sj| sj.end_time * job_map.get(&sj.job_id).copied().unwrap_or(1.0))
            .sum()
    }

    pub fn max_lateness(&self, jobs: &[Job]) -> f64 {
        let job_map: std::collections::HashMap<usize, f64> = jobs
            .iter()
            .filter_map(|j| j.due_date.map(|dd| (j.id, dd)))
            .collect();
        self.scheduled_jobs
            .iter()
            .map(|sj| {
                let due = job_map.get(&sj.job_id).copied().unwrap_or(f64::INFINITY);
                sj.end_time - due
            })
            .fold(0.0_f64, f64::max)
    }

    pub fn total_tardiness(&self, jobs: &[Job]) -> f64 {
        let job_map: std::collections::HashMap<usize, f64> = jobs
            .iter()
            .filter_map(|j| j.due_date.map(|dd| (j.id, dd)))
            .collect();
        self.scheduled_jobs
            .iter()
            .map(|sj| {
                let due = job_map.get(&sj.job_id).copied().unwrap_or(f64::INFINITY);
                (sj.end_time - due).max(0.0)
            })
            .sum()
    }

    pub fn job_order(&self) -> Vec<usize> {
        let mut jobs: Vec<_> = self.scheduled_jobs.clone();
        jobs.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
        jobs.iter().map(|j| j.job_id).collect()
    }
}
