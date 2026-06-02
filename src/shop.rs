use crate::job::{Schedule, ScheduledJob};

/// A job for job shop: a sequence of (machine, processing_time) operations.
#[derive(Debug, Clone)]
pub struct JobShopJob {
    pub id: usize,
    pub operations: Vec<(usize, f64)>, // (machine, processing_time) in route order
}

/// A task for open/job shop: job_id, machine, processing_time.
#[derive(Debug, Clone)]
pub struct ShopTask {
    pub job_id: usize,
    pub machine: usize,
    pub processing_time: f64,
}

/// Open shop scheduler: each job has operations on different machines,
/// but the order doesn't matter.
pub struct OpenShopScheduler;

impl OpenShopScheduler {
    /// Simple heuristic for open shop scheduling.
    /// Uses a greedy approach: schedule operations in order of longest remaining.
    pub fn schedule(tasks: &[ShopTask], num_machines: usize) -> Schedule {
        let mut schedule = Schedule::new(num_machines);
        let mut machine_times = vec![0.0_f64; num_machines];
        let mut job_end_times: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();

        let mut remaining: Vec<usize> = (0..tasks.len()).collect();

        while !remaining.is_empty() {
            // Sort by processing time descending (LPT heuristic)
            remaining.sort_by(|a, b| {
                tasks[*b].processing_time.partial_cmp(&tasks[*a].processing_time).unwrap()
            });

            // Try to schedule each remaining task
            let mut scheduled_this_round = false;
            let mut to_schedule: Option<usize> = None;
            let mut best_start = f64::INFINITY;

            for &idx in &remaining {
                let task = &tasks[idx];
                let job_available = job_end_times.get(&task.job_id).copied().unwrap_or(0.0);
                let machine_available = machine_times[task.machine];
                let start = job_available.max(machine_available);

                if start < best_start {
                    best_start = start;
                    to_schedule = Some(idx);
                }
            }

            if let Some(idx) = to_schedule {
                let task = &tasks[idx];
                let job_available = job_end_times.get(&task.job_id).copied().unwrap_or(0.0);
                let machine_available = machine_times[task.machine];
                let start = job_available.max(machine_available);
                let end = start + task.processing_time;

                schedule.scheduled_jobs.push(ScheduledJob {
                    job_id: task.job_id,
                    start_time: start,
                    end_time: end,
                    machine: task.machine,
                });

                machine_times[task.machine] = end;
                job_end_times.insert(task.job_id, end);

                remaining.retain(|&x| x != idx);
                scheduled_this_round = true;
            }

            if !scheduled_this_round {
                break;
            }
        }

        schedule
    }

    /// Compute makespan for a set of scheduled tasks.
    pub fn makespan(schedule: &Schedule) -> f64 {
        schedule.makespan()
    }
}

/// Job shop scheduler: each job has a fixed route through machines.
pub struct JobShopScheduler;

impl JobShopScheduler {
    /// Schedule job shop using a simple greedy/dispatching approach.
    pub fn schedule(jobs: &[JobShopJob], num_machines: usize) -> Schedule {
        let mut schedule = Schedule::new(num_machines);
        let mut machine_times = vec![0.0_f64; num_machines];
        let mut job_progress: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        let mut job_end_times: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();

        // Track which operations are still to be scheduled
        let mut remaining_ops: usize = jobs.iter().map(|j| j.operations.len()).sum();

        while remaining_ops > 0 {
            // Find all schedulable operations (next operation for each job)
            let mut candidates: Vec<(usize, usize)> = Vec::new(); // (job_idx, op_idx)
            for (jidx, job) in jobs.iter().enumerate() {
                let next_op = job_progress.get(&job.id).copied().unwrap_or(0);
                if next_op < job.operations.len() {
                    candidates.push((jidx, next_op));
                }
            }

            if candidates.is_empty() {
                break;
            }

            // Schedule the candidate with earliest possible start (SPT-like tie-breaking)
            candidates.sort_by(|a, b| {
                let ja = jobs[a.0].operations[a.1].1;
                let jb = jobs[b.0].operations[b.1].1;
                ja.partial_cmp(&jb).unwrap()
            });

            let (jidx, op_idx) = candidates[0];
            let job = &jobs[jidx];
            let (machine, proc_time) = job.operations[op_idx];

            let job_available = job_end_times.get(&job.id).copied().unwrap_or(0.0);
            let machine_available = machine_times[machine];
            let start = job_available.max(machine_available);
            let end = start + proc_time;

            schedule.scheduled_jobs.push(ScheduledJob {
                job_id: job.id,
                start_time: start,
                end_time: end,
                machine,
            });

            machine_times[machine] = end;
            job_end_times.insert(job.id, end);
            job_progress.insert(job.id, op_idx + 1);
            remaining_ops -= 1;
        }

        schedule
    }
}
