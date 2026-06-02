use crate::job::{Job, Schedule, ScheduledJob};

/// A resource with limited capacity.
#[derive(Debug, Clone)]
pub struct Resource {
    pub id: usize,
    pub capacity: f64,
}

/// A job that requires certain resources.
#[derive(Debug, Clone)]
pub struct ResourceJob {
    pub job: Job,
    pub resource_requirements: Vec<(usize, f64)>, // (resource_id, amount)
}

/// Resource-constrained scheduler using a serial schedule generation scheme (SGS).
pub struct ResourceConstrainedScheduler;

impl ResourceConstrainedScheduler {
    /// Schedule jobs subject to resource constraints.
    /// Uses a serial SGS: at each decision point, schedule the highest priority
    /// eligible job whose resource requirements can be met.
    pub fn schedule(
        jobs: &[ResourceJob],
        resources: &[Resource],
        precedence: &[(usize, usize)], // (before, after) job indices
    ) -> Schedule {
        let n = jobs.len();
        if n == 0 {
            return Schedule::new(1);
        }

        let mut resource_caps: Vec<f64> = resources.iter().map(|r| r.capacity).collect();
        let mut resource_available: Vec<f64> = resource_caps.clone();
        let mut scheduled_indices: Vec<usize> = Vec::new();
        let mut end_times: Vec<f64> = vec![0.0; n];
        let mut started = vec![false; n];

        // Build precedence successors
        let mut successors: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut in_degree = vec![0usize; n];
        for &(before, after) in precedence {
            successors[before].push(after);
            in_degree[after] += 1;
        }

        let mut schedule = Schedule::new(1);

        while scheduled_indices.len() < n {
            // Find eligible jobs: not started, all predecessors done
            let mut eligible: Vec<usize> = (0..n)
                .filter(|&i| !started[i])
                .filter(|&i| {
                    let pred_count = (0..n).filter(|&j| successors[j].contains(&i) && !started[j]).count();
                    pred_count == 0
                })
                .collect();

            if eligible.is_empty() {
                // This shouldn't happen if the problem is feasible
                break;
            }

            // Sort eligible by processing time (SPT-like)
            eligible.sort_by(|a, b| {
                jobs[*a].job.processing_time.partial_cmp(&jobs[*b].job.processing_time).unwrap()
            });

            // Try to schedule the first eligible job whose resources are available
            let mut found = false;
            for &idx in &eligible {
                let can_schedule = jobs[idx].resource_requirements.iter().all(|&(rid, amt)| {
                    if rid < resource_available.len() {
                        resource_available[rid] >= amt
                    } else {
                        true
                    }
                });

                if can_schedule {
                    // Allocate resources
                    for &(rid, amt) in &jobs[idx].resource_requirements {
                        if rid < resource_available.len() {
                            resource_available[rid] -= amt;
                        }
                    }

                    // Determine start time
                    let predecessor_end = (0..n)
                        .filter(|&j| successors[j].contains(&idx) && started[j])
                        .map(|j| end_times[j])
                        .fold(0.0_f64, f64::max);

                    let release = jobs[idx].job.release_date.unwrap_or(0.0);
                    let start = predecessor_end.max(release);
                    let end = start + jobs[idx].job.processing_time;

                    started[idx] = true;
                    end_times[idx] = end;
                    scheduled_indices.push(idx);

                    schedule.scheduled_jobs.push(ScheduledJob {
                        job_id: jobs[idx].job.id,
                        start_time: start,
                        end_time: end,
                        machine: 0,
                    });

                    // Release resources after job completes
                    for &(rid, amt) in &jobs[idx].resource_requirements {
                        if rid < resource_available.len() {
                            resource_available[rid] += amt;
                        }
                    }

                    found = true;
                    break;
                }
            }

            if !found {
                // No eligible job can be scheduled now; advance time to next completion
                // This is a simplified approach - in practice we'd need event-based simulation
                break;
            }
        }

        schedule
    }
}
