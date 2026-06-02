use crate::job::{Job, Schedule, ScheduledJob};
use crate::priority::{self, PriorityRule};

/// Single-machine scheduler.
pub struct SingleMachineScheduler;

impl SingleMachineScheduler {
    /// Schedule jobs on a single machine using the given priority rule.
    pub fn schedule(jobs: &[Job], rule: PriorityRule) -> Schedule {
        let order = priority::apply_rule(jobs, rule);
        let mut schedule = Schedule::new(1);
        let mut t: f64 = 0.0;
        for idx in order {
            let job = &jobs[idx];
            let release = job.release_date.unwrap_or(0.0);
            let start = t.max(release);
            let end = start + job.processing_time;
            schedule.scheduled_jobs.push(ScheduledJob {
                job_id: job.id,
                start_time: start,
                end_time: end,
                machine: 0,
            });
            t = end;
        }
        schedule
    }

    /// Compute total completion time for a given job order (indices).
    pub fn total_completion_time(jobs: &[Job], order: &[usize]) -> f64 {
        let mut t = 0.0;
        let mut total = 0.0;
        for &idx in order {
            t += jobs[idx].processing_time;
            total += t;
        }
        total
    }

    /// Compute total weighted completion time for a given order.
    pub fn total_weighted_completion_time(jobs: &[Job], order: &[usize]) -> f64 {
        let mut t = 0.0;
        let mut total = 0.0;
        for &idx in order {
            t += jobs[idx].processing_time;
            total += t * jobs[idx].weight;
        }
        total
    }

    /// Compute maximum lateness for a given order.
    pub fn max_lateness(jobs: &[Job], order: &[usize]) -> f64 {
        let mut t = 0.0;
        let mut max_l = 0.0_f64;
        for &idx in order {
            let job = &jobs[idx];
            t += job.processing_time;
            if let Some(dd) = job.due_date {
                max_l = max_l.max(t - dd);
            }
        }
        max_l
    }
}
