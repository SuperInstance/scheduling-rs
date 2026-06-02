use crate::job::{Job, Schedule, ScheduledJob};
use crate::priority::{self, PriorityRule};

/// Parallel machine scheduler.
pub struct ParallelMachineScheduler;

impl ParallelMachineScheduler {
    /// Schedule jobs on `m` identical parallel machines using list scheduling.
    /// Uses the given priority rule to order jobs, then greedily assigns each
    /// job to the machine that becomes available earliest.
    pub fn schedule(jobs: &[Job], m: usize, rule: PriorityRule) -> Schedule {
        assert!(m > 0);
        let order = priority::apply_rule(jobs, rule);
        let mut machine_times = vec![0.0_f64; m];
        let mut schedule = Schedule::new(m);

        for idx in order {
            let job = &jobs[idx];
            // Pick machine with smallest load
            let (machine, _) = machine_times
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap();
            let start = machine_times[machine].max(job.release_date.unwrap_or(0.0));
            let end = start + job.processing_time;
            schedule.scheduled_jobs.push(ScheduledJob {
                job_id: job.id,
                start_time: start,
                end_time: end,
                machine,
            });
            machine_times[machine] = end;
        }
        schedule
    }

    /// Compute lower bound on makespan: max(max_job_time, total_processing_time / m).
    pub fn makespan_lower_bound(jobs: &[Job], m: usize) -> f64 {
        let max_job = jobs.iter().map(|j| j.processing_time).fold(0.0_f64, f64::max);
        let total: f64 = jobs.iter().map(|j| j.processing_time).sum();
        max_job.max(total / m as f64)
    }

    /// LPT heuristic: sort by longest processing time first, then list schedule.
    pub fn schedule_lpt(jobs: &[Job], m: usize) -> Schedule {
        Self::schedule(jobs, m, PriorityRule::LPT)
    }

    /// SPT heuristic for parallel machines.
    pub fn schedule_spt(jobs: &[Job], m: usize) -> Schedule {
        Self::schedule(jobs, m, PriorityRule::SPT)
    }
}
