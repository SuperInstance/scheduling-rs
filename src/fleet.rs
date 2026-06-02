use crate::job::{Schedule, ScheduledJob};

/// A worker in the fleet.
#[derive(Debug, Clone)]
pub struct Worker {
    pub id: usize,
    pub capacity: f64,    // processing speed multiplier
    pub specializations: Vec<String>, // task types this worker can handle
}

/// A task to be assigned to a worker.
#[derive(Debug, Clone)]
pub struct FleetTask {
    pub id: usize,
    pub processing_time: f64,
    pub task_type: String,
    pub priority: f64,
    pub deadline: Option<f64>,
    pub dependencies: Vec<usize>, // task IDs that must complete first
}

/// Worker fleet scheduler: optimal task distribution across worker pools.
pub struct FleetScheduler;

impl FleetScheduler {
    /// Schedule tasks across workers using a greedy assignment.
    /// Respects worker specializations and minimizes makespan.
    pub fn schedule(tasks: &[FleetTask], workers: &[Worker]) -> Schedule {
        let num_workers = workers.len();
        if num_workers == 0 || tasks.is_empty() {
            return Schedule::new(num_workers);
        }

        // Build assignment cost: task -> compatible workers
        let mut task_workers: Vec<Vec<usize>> = Vec::new();
        for task in tasks {
            let compatible: Vec<usize> = workers
                .iter()
                .enumerate()
                .filter(|(_, worker)| {
                    worker.specializations.is_empty()
                        || worker.specializations.contains(&task.task_type)
                })
                .map(|(i, _)| i)
                .collect();
            task_workers.push(compatible);
        }

        // Sort tasks by priority (descending) then by deadline (ascending)
        let mut task_order: Vec<usize> = (0..tasks.len()).collect();
        task_order.sort_by(|a, b| {
            let pa = tasks[*a].priority;
            let pb = tasks[*b].priority;
            match pb.partial_cmp(&pa) {
                Some(std::cmp::Ordering::Equal) => {
                    let da = tasks[*a].deadline.unwrap_or(f64::INFINITY);
                    let db = tasks[*b].deadline.unwrap_or(f64::INFINITY);
                    da.partial_cmp(&db).unwrap()
                }
                other => other.unwrap(),
            }
        });

        let mut worker_loads = vec![0.0_f64; num_workers];
        let mut task_completion: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
        let mut schedule = Schedule::new(num_workers);
        let mut assigned = vec![false; tasks.len()];

        // Multiple passes to handle dependencies
        let max_passes = tasks.len() * 2;
        for _ in 0..max_passes {
            let all_assigned = assigned.iter().all(|&a| a);
            if all_assigned {
                break;
            }

            for &tidx in &task_order {
                if assigned[tidx] {
                    continue;
                }

                let task = &tasks[tidx];

                // Check dependencies
                let deps_met = task.dependencies.iter().all(|&dep_id| {
                    task_completion.contains_key(&dep_id)
                });
                if !deps_met {
                    continue;
                }

                // Find earliest available compatible worker
                let compatible = &task_workers[tidx];
                if compatible.is_empty() {
                    continue;
                }

                let best_worker = *compatible
                    .iter()
                    .min_by(|a, b| worker_loads[**a].partial_cmp(&worker_loads[**b]).unwrap())
                    .unwrap();

                let dep_end = task.dependencies.iter()
                    .filter_map(|d| task_completion.get(d))
                    .fold(0.0_f64, |a, &b| a.max(b));

                let worker_available = worker_loads[best_worker];
                let start = dep_end.max(worker_available);
                let actual_time = task.processing_time / workers[best_worker].capacity;
                let end = start + actual_time;

                schedule.scheduled_jobs.push(ScheduledJob {
                    job_id: task.id,
                    start_time: start,
                    end_time: end,
                    machine: best_worker,
                });

                worker_loads[best_worker] = end;
                task_completion.insert(task.id, end);
                assigned[tidx] = true;
            }
        }

        schedule
    }

    /// Compute worker utilization (fraction of makespan each worker is busy).
    pub fn worker_utilization(schedule: &Schedule) -> Vec<f64> {
        let makespan = schedule.makespan();
        if makespan <= 0.0 {
            return vec![0.0; schedule.num_machines];
        }

        let mut busy_time = vec![0.0_f64; schedule.num_machines];
        for sj in &schedule.scheduled_jobs {
            busy_time[sj.machine] += sj.end_time - sj.start_time;
        }

        busy_time.iter().map(|&b| b / makespan).collect()
    }

    /// Compute load balance: standard deviation of worker loads.
    pub fn load_balance(schedule: &Schedule) -> f64 {
        let mut busy_time = vec![0.0_f64; schedule.num_machines];
        for sj in &schedule.scheduled_jobs {
            busy_time[sj.machine] += sj.end_time - sj.start_time;
        }

        if busy_time.is_empty() {
            return 0.0;
        }

        let mean: f64 = busy_time.iter().sum::<f64>() / busy_time.len() as f64;
        let variance: f64 = busy_time.iter().map(|&b| (b - mean).powi(2)).sum::<f64>() / busy_time.len() as f64;
        variance.sqrt()
    }
}
