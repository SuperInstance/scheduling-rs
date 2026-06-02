use crate::job::{Schedule, ScheduledJob};

/// A job for 2-machine flow shop: each job has processing times on machine A and machine B.
#[derive(Debug, Clone)]
pub struct FlowShopJob {
    pub id: usize,
    pub time_a: f64,
    pub time_b: f64,
}

/// Johnson's algorithm for the 2-machine flow shop problem (F2 || C_max).
pub struct JohnsonAlgorithm;

impl JohnsonAlgorithm {
    /// Run Johnson's algorithm and return the optimal sequence (job ids).
    pub fn sequence(jobs: &[FlowShopJob]) -> Vec<usize> {
        let n = jobs.len();
        if n == 0 {
            return vec![];
        }
        let mut front = Vec::new();
        let mut back = Vec::new();
        let mut remaining: Vec<usize> = (0..n).collect();

        while !remaining.is_empty() {
            // Find job with minimum processing time across both machines
            let (min_idx, min_pos) = remaining
                .iter()
                .map(|&i| {
                    let ta = jobs[i].time_a;
                    let tb = jobs[i].time_b;
                    if ta <= tb {
                        (i, 'a')
                    } else {
                        (i, 'b')
                    }
                })
                .min_by(|a, b| {
                    let va = if a.1 == 'a' {
                        jobs[a.0].time_a
                    } else {
                        jobs[a.0].time_b
                    };
                    let vb = if b.1 == 'a' {
                        jobs[b.0].time_a
                    } else {
                        jobs[b.0].time_b
                    };
                    va.partial_cmp(&vb).unwrap()
                })
                .unwrap();

            if min_pos == 'a' {
                front.push(jobs[min_idx].id);
            } else {
                back.push(jobs[min_idx].id);
            }
            remaining.retain(|&x| x != min_idx);
        }

        front.extend(back.into_iter().rev());
        front
    }

    /// Compute the makespan for a given sequence on a 2-machine flow shop.
    pub fn makespan(jobs: &[FlowShopJob], sequence: &[usize]) -> f64 {
        let job_map: std::collections::HashMap<usize, &FlowShopJob> =
            jobs.iter().map(|j| (j.id, j)).collect();
        let mut time_a: f64 = 0.0;
        let mut time_b: f64 = 0.0;
        for &jid in sequence {
            let job = job_map[&jid];
            time_a += job.time_a;
            time_b = time_b.max(time_a) + job.time_b;
        }
        time_b
    }

    /// Run Johnson's algorithm and return a full schedule.
    pub fn schedule(jobs: &[FlowShopJob]) -> Schedule {
        let seq = Self::sequence(jobs);
        let job_map: std::collections::HashMap<usize, &FlowShopJob> =
            jobs.iter().map(|j| (j.id, j)).collect();

        let mut schedule = Schedule::new(2);
        let mut time_a: f64 = 0.0;
        let mut time_b: f64 = 0.0;

        for &jid in &seq {
            let job = job_map[&jid];
            let start_a = time_a;
            let end_a = start_a + job.time_a;
            schedule.scheduled_jobs.push(ScheduledJob {
                job_id: jid,
                start_time: start_a,
                end_time: end_a,
                machine: 0,
            });
            let start_b = time_b.max(end_a);
            let end_b = start_b + job.time_b;
            schedule.scheduled_jobs.push(ScheduledJob {
                job_id: jid,
                start_time: start_b,
                end_time: end_b,
                machine: 1,
            });
            time_a = end_a;
            time_b = end_b;
        }
        schedule
    }
}
