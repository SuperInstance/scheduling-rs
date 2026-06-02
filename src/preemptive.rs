use crate::job::{Job, Schedule, ScheduledJob};

/// A segment of a preemptive schedule for one job.
#[derive(Debug, Clone)]
pub struct PreemptiveSegment {
    pub job_id: usize,
    pub start: f64,
    pub end: f64,
    pub machine: usize,
}

/// Preemptive scheduling algorithms.
pub struct PreemptiveScheduler;

impl PreemptiveScheduler {
    /// Shortest Remaining Processing Time (SRPT) preemptive scheduling on a single machine.
    /// At each time point, schedule the job with the shortest remaining time.
    /// Returns segments for visualization.
    pub fn srpt(jobs: &[Job]) -> Vec<PreemptiveSegment> {
        let n = jobs.len();
        if n == 0 {
            return vec![];
        }

        let mut remaining: Vec<f64> = jobs.iter().map(|j| j.processing_time).collect();
        let mut release: Vec<f64> = jobs
            .iter()
            .map(|j| j.release_date.unwrap_or(0.0))
            .collect();
        let mut segments: Vec<PreemptiveSegment> = Vec::new();
        let mut t: f64 = 0.0;

        loop {
            // Find next event time
            let total_remaining: f64 = remaining.iter().sum();
            if total_remaining < 1e-12 {
                break;
            }

            // Find available jobs with remaining work
            let mut available: Vec<usize> = (0..n)
                .filter(|&i| remaining[i] > 1e-12 && release[i] <= t + 1e-12)
                .collect();

            if available.is_empty() {
                // Jump to next release
                let next_release = (0..n)
                    .filter(|&i| remaining[i] > 1e-12)
                    .filter_map(|i| {
                        if release[i] > t + 1e-12 {
                            Some(release[i])
                        } else {
                            None
                        }
                    })
                    .fold(f64::INFINITY, f64::min);
                t = next_release;
                continue;
            }

            // Pick shortest remaining
            available.sort_by(|a, b| remaining[*a].partial_cmp(&remaining[*b]).unwrap());
            let job_idx = available[0];

            // Find next event: either a new job releases or this job finishes
            let finish_time = t + remaining[job_idx];
            let next_release_time = (0..n)
                .filter(|&i| remaining[i] > 1e-12 && release[i] > t + 1e-12)
                .map(|i| release[i])
                .fold(f64::INFINITY, f64::min);

            let end = finish_time.min(next_release_time);
            let duration = end - t;

            segments.push(PreemptiveSegment {
                job_id: jobs[job_idx].id,
                start: t,
                end,
                machine: 0,
            });

            remaining[job_idx] -= duration;
            t = end;
        }

        segments
    }

    /// Convert preemptive segments to a Schedule.
    pub fn segments_to_schedule(segments: &[PreemptiveSegment]) -> Schedule {
        let mut schedule = Schedule::new(1);
        for seg in segments {
            schedule.scheduled_jobs.push(ScheduledJob {
                job_id: seg.job_id,
                start_time: seg.start,
                end_time: seg.end,
                machine: seg.machine,
            });
        }
        schedule
    }

    /// Compute total completion time from preemptive segments.
    pub fn total_completion_time(jobs: &[Job], segments: &[PreemptiveSegment]) -> f64 {
        let mut completion: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
        for seg in segments {
            let e = completion.entry(seg.job_id).or_insert(0.0_f64);
            *e = e.max(seg.end);
        }
        let mut total = 0.0;
        for job in jobs {
            if let Some(&c) = completion.get(&job.id) {
                total += c;
            }
        }
        total
    }

    /// Preemptive EDD: Earliest Due Date with preemption (minimize max lateness).
    /// Uses the Moore-Hodgson-like approach with preemption.
    pub fn preemptive_edd(jobs: &[Job]) -> Vec<PreemptiveSegment> {
        // For preemptive single machine, EDD is optimal for minimizing max lateness
        // Sort by due date and schedule (preemption doesn't help for max lateness on single machine
        // without release dates, but we allow it for release date cases)
        let n = jobs.len();
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|a, b| {
            let da = jobs[*a].due_date.unwrap_or(f64::INFINITY);
            let db = jobs[*b].due_date.unwrap_or(f64::INFINITY);
            da.partial_cmp(&db).unwrap()
        });

        let mut segments = Vec::new();
        let mut t: f64 = 0.0;
        for &idx in &order {
            let job = &jobs[idx];
            let start = t.max(job.release_date.unwrap_or(0.0));
            let end = start + job.processing_time;
            segments.push(PreemptiveSegment {
                job_id: job.id,
                start,
                end,
                machine: 0,
            });
            t = end;
        }
        segments
    }
}
