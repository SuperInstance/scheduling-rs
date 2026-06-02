use crate::job::{Job, Schedule, ScheduledJob};

/// Branch and bound for optimal single-machine scheduling.
/// Minimizes total weighted completion time.
pub struct BranchAndBound;

impl BranchAndBound {
    /// Find the optimal sequence minimizing total weighted completion time.
    /// Returns job indices in optimal order.
    pub fn optimize_twct(jobs: &[Job]) -> Vec<usize> {
        let n = jobs.len();
        if n <= 1 {
            return (0..n).collect();
        }
        let mut best_cost = f64::INFINITY;
        let mut best_seq: Vec<usize> = (0..n).collect();
        let mut current: Vec<usize> = Vec::with_capacity(n);
        let mut used = vec![false; n];
        let mut current_time = 0.0;
        let mut current_cost = 0.0;

        Self::bb_twct(
            jobs,
            &mut current,
            &mut used,
            &mut current_time,
            &mut current_cost,
            &mut best_cost,
            &mut best_seq,
        );
        best_seq
    }

    fn bb_twct(
        jobs: &[Job],
        current: &mut Vec<usize>,
        used: &mut [bool],
        current_time: &mut f64,
        current_cost: &mut f64,
        best_cost: &mut f64,
        best_seq: &mut Vec<usize>,
    ) {
        let n = jobs.len();
        if current.len() == n {
            if *current_cost < *best_cost {
                *best_cost = *current_cost;
                *best_seq = current.clone();
            }
            return;
        }

        // Lower bound: current_cost + remaining jobs scheduled in WSPT order
        let remaining: Vec<usize> = (0..n).filter(|&i| !used[i]).collect();
        let lb = Self::lower_bound_twct(jobs, &remaining, *current_time, *current_cost);
        if lb >= *best_cost {
            return; // prune
        }

        // Branch: try each remaining job
        let mut candidates: Vec<usize> = remaining;
        candidates.sort_by(|a, b| {
            let ra = jobs[*a].processing_time / jobs[*a].weight;
            let rb = jobs[*b].processing_time / jobs[*b].weight;
            ra.partial_cmp(&rb).unwrap()
        });

        for idx in candidates {
            used[idx] = true;
            let p = jobs[idx].processing_time;
            let w = jobs[idx].weight;
            let old_time = *current_time;
            let old_cost = *current_cost;
            *current_time += p;
            *current_cost += *current_time * w;
            current.push(idx);

            Self::bb_twct(jobs, current, used, current_time, current_cost, best_cost, best_seq);

            current.pop();
            *current_time = old_time;
            *current_cost = old_cost;
            used[idx] = false;
        }
    }

    fn lower_bound_twct(
        jobs: &[Job],
        remaining: &[usize],
        current_time: f64,
        current_cost: f64,
    ) -> f64 {
        // WSPT lower bound
        let mut items: Vec<(f64, f64)> = remaining.iter().map(|&i| (jobs[i].processing_time, jobs[i].weight)).collect();
        items.sort_by(|a, b| (a.0 / a.1).partial_cmp(&(b.0 / b.1)).unwrap());
        let mut t = current_time;
        let mut cost = current_cost;
        for (p, w) in &items {
            t += p;
            cost += t * w;
        }
        cost
    }

    /// Find the optimal sequence minimizing makespan on parallel machines using B&B.
    /// This is a simplified version for small instances.
    pub fn optimize_makespan_parallel(jobs: &[Job], m: usize) -> Schedule {
        assert!(m > 0);
        let n = jobs.len();
        if n == 0 {
            return Schedule::new(m);
        }

        let mut best_makespan = f64::INFINITY;
        let mut best_assignment: Vec<usize> = vec![0; n]; // machine assignment per job

        let mut assignment = vec![0usize; n];
        let mut machine_loads = vec![0.0_f64; m];

        Self::bb_makespan(
            jobs,
            m,
            0,
            &mut assignment,
            &mut machine_loads,
            &mut best_makespan,
            &mut best_assignment,
        );

        // Build schedule from best assignment
        let mut schedule = Schedule::new(m);
        let mut machine_times = vec![0.0_f64; m];
        // Order jobs by their assignment and schedule sequentially on each machine
        for i in 0..n {
            let mach = best_assignment[i];
            let start = machine_times[mach];
            let end = start + jobs[i].processing_time;
            schedule.scheduled_jobs.push(ScheduledJob {
                job_id: jobs[i].id,
                start_time: start,
                end_time: end,
                machine: mach,
            });
            machine_times[mach] = end;
        }
        schedule
    }

    fn bb_makespan(
        jobs: &[Job],
        m: usize,
        job_idx: usize,
        assignment: &mut [usize],
        machine_loads: &mut [f64],
        best_makespan: &mut f64,
        best_assignment: &mut [usize],
    ) {
        if job_idx == jobs.len() {
            let makespan = machine_loads.iter().cloned().fold(0.0_f64, f64::max);
            if makespan < *best_makespan {
                *best_makespan = makespan;
                best_assignment.copy_from_slice(assignment);
            }
            return;
        }

        let current_max = machine_loads.iter().cloned().fold(0.0_f64, f64::max);
        if current_max >= *best_makespan {
            return; // prune
        }

        for mach in 0..m {
            assignment[job_idx] = mach;
            machine_loads[mach] += jobs[job_idx].processing_time;

            Self::bb_makespan(jobs, m, job_idx + 1, assignment, machine_loads, best_makespan, best_assignment);

            machine_loads[mach] -= jobs[job_idx].processing_time;
        }
    }

    /// Create a schedule from an ordered sequence of job indices on a single machine.
    pub fn schedule_from_sequence(jobs: &[Job], sequence: &[usize]) -> Schedule {
        let mut schedule = Schedule::new(1);
        let mut t: f64 = 0.0;
        for &idx in sequence {
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
}
