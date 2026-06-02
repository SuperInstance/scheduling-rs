use crate::job::{Job, Schedule, ScheduledJob};
use crate::priority::PriorityRule;
use crate::single_machine::SingleMachineScheduler;

/// Due date-based scheduling objectives.
pub struct DueDateScheduler;

impl DueDateScheduler {
    /// Minimize maximum lateness using EDD rule (optimal for L_max on single machine).
    pub fn minimize_max_lateness(jobs: &[Job]) -> Schedule {
        SingleMachineScheduler::schedule(jobs, PriorityRule::EDD)
    }

    /// Minimize total tardiness using a modified SPT approach.
    /// Note: This is NP-hard in general; we use a heuristic.
    pub fn minimize_total_tardiness(jobs: &[Job]) -> Schedule {
        let n = jobs.len();
        if n == 0 {
            return Schedule::new(1);
        }

        // Use a simple heuristic: sort by slack (due_date - processing_time)
        let mut indexed: Vec<usize> = (0..n).collect();
        indexed.sort_by(|a, b| {
            let slack_a = jobs[*a].due_date.unwrap_or(f64::INFINITY) - jobs[*a].processing_time;
            let slack_b = jobs[*b].due_date.unwrap_or(f64::INFINITY) - jobs[*b].processing_time;
            slack_a.partial_cmp(&slack_b).unwrap()
        });

        let mut schedule = Schedule::new(1);
        let mut t: f64 = 0.0;
        for idx in indexed {
            let job = &jobs[idx];
            let start = t.max(job.release_date.unwrap_or(0.0));
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

    /// Moore's algorithm to minimize the number of tardy jobs.
    /// Returns the schedule with maximum on-time jobs.
    pub fn minimize_num_tardy(jobs: &[Job]) -> Schedule {
        let n = jobs.len();
        if n == 0 {
            return Schedule::new(1);
        }

        // Sort by EDD
        let mut indexed: Vec<usize> = (0..n).collect();
        indexed.sort_by(|a, b| {
            let da = jobs[*a].due_date.unwrap_or(f64::INFINITY);
            let db = jobs[*b].due_date.unwrap_or(f64::INFINITY);
            da.partial_cmp(&db).unwrap()
        });

        let mut accepted: Vec<usize> = Vec::new();
        let mut total_time = 0.0;

        for &idx in &indexed {
            accepted.push(idx);
            total_time += jobs[idx].processing_time;

            // Check if any accepted job is tardy
            let tardy = Self::has_tardy(jobs, &accepted);
            if tardy {
                // Remove the job with the longest processing time
                let longest_idx = accepted
                    .iter()
                    .enumerate()
                    .max_by(|a, b| jobs[*a.1].processing_time.partial_cmp(&jobs[*b.1].processing_time).unwrap())
                    .unwrap()
                    .0;
                total_time -= jobs[accepted[longest_idx]].processing_time;
                accepted.remove(longest_idx);
            }
        }

        // Build schedule for accepted jobs (in EDD order), then append rejected
        let mut rejected: Vec<usize> = indexed.into_iter().filter(|i| !accepted.contains(i)).collect();
        let mut final_order = accepted;
        final_order.append(&mut rejected);

        let mut schedule = Schedule::new(1);
        let mut t: f64 = 0.0;
        for idx in final_order {
            let job = &jobs[idx];
            let start = t;
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

    fn has_tardy(jobs: &[Job], order: &[usize]) -> bool {
        let mut t: f64 = 0.0;
        for &idx in order {
            t += jobs[idx].processing_time;
            if let Some(dd) = jobs[idx].due_date {
                if t > dd + 1e-10 {
                    return true;
                }
            }
        }
        false
    }

    /// Compute the number of tardy jobs in a schedule.
    pub fn num_tardy(jobs: &[Job], schedule: &Schedule) -> usize {
        let job_map: std::collections::HashMap<usize, f64> = jobs
            .iter()
            .filter_map(|j| j.due_date.map(|dd| (j.id, dd)))
            .collect();
        schedule
            .scheduled_jobs
            .iter()
            .filter(|sj| {
                if let Some(&dd) = job_map.get(&sj.job_id) {
                    sj.end_time > dd + 1e-10
                } else {
                    false
                }
            })
            .count()
    }

    /// Compute maximum lateness given a schedule.
    pub fn max_lateness(jobs: &[Job], schedule: &Schedule) -> f64 {
        schedule.max_lateness(jobs)
    }

    /// Compute total tardiness given a schedule.
    pub fn total_tardiness(jobs: &[Job], schedule: &Schedule) -> f64 {
        schedule.total_tardiness(jobs)
    }
}
