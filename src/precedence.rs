use crate::job::{Job, Schedule, ScheduledJob};

/// Scheduling with precedence constraints.
pub struct PrecedenceScheduler;

/// A precedence constraint: job `before` must complete before job `after` can start.
#[derive(Debug, Clone)]
pub struct PrecedenceConstraint {
    pub before: usize,
    pub after: usize,
}

impl PrecedenceScheduler {
    /// Topological sort respecting precedence constraints.
    /// Returns job indices in a valid order.
    pub fn topological_sort(n: usize, constraints: &[PrecedenceConstraint]) -> Vec<usize> {
        let mut in_degree = vec![0usize; n];
        let mut successors: Vec<Vec<usize>> = vec![Vec::new(); n];

        for c in constraints {
            successors[c.before].push(c.after);
            in_degree[c.after] += 1;
        }

        let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        for i in 0..n {
            if in_degree[i] == 0 {
                queue.push_back(i);
            }
        }

        let mut result = Vec::with_capacity(n);
        while let Some(node) = queue.pop_front() {
            result.push(node);
            for &succ in &successors[node] {
                in_degree[succ] -= 1;
                if in_degree[succ] == 0 {
                    queue.push_back(succ);
                }
            }
        }

        assert_eq!(result.len(), n, "Cycle detected in precedence constraints");
        result
    }

    /// Schedule jobs on a single machine respecting precedence constraints.
    /// Uses topological ordering.
    pub fn schedule(jobs: &[Job], constraints: &[PrecedenceConstraint]) -> Schedule {
        let order = Self::topological_sort(jobs.len(), constraints);
        let mut schedule = Schedule::new(1);
        let mut t: f64 = 0.0;
        for idx in order {
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

    /// Schedule with precedence constraints using a priority rule within the topological order.
    /// At each step, among all ready jobs (all predecessors completed), pick the one
    /// with the highest priority according to SPT.
    pub fn schedule_spt_with_precedence(jobs: &[Job], constraints: &[PrecedenceConstraint]) -> Schedule {
        let n = jobs.len();
        let mut in_degree = vec![0usize; n];
        let mut successors: Vec<Vec<usize>> = vec![Vec::new(); n];

        for c in constraints {
            successors[c.before].push(c.after);
            in_degree[c.after] += 1;
        }

        // Use a binary heap for SPT ordering among ready jobs
        let mut ready: std::collections::BinaryHeap<std::cmp::Reverse<(std::cmp::Reverse<OrderedFloat>, usize)>> =
            std::collections::BinaryHeap::new();

        for i in 0..n {
            if in_degree[i] == 0 {
                ready.push(std::cmp::Reverse((
                    std::cmp::Reverse(OrderedFloat(jobs[i].processing_time)),
                    i,
                )));
            }
        }

        let mut schedule = Schedule::new(1);
        let mut t: f64 = 0.0;

        while let Some(std::cmp::Reverse((_, idx))) = ready.pop() {
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

            for &succ in &successors[idx] {
                in_degree[succ] -= 1;
                if in_degree[succ] == 0 {
                    ready.push(std::cmp::Reverse((
                        std::cmp::Reverse(OrderedFloat(jobs[succ].processing_time)),
                        succ,
                    )));
                }
            }
        }

        schedule
    }

    /// Check if a given sequence respects all precedence constraints.
    pub fn is_valid_sequence(sequence: &[usize], constraints: &[PrecedenceConstraint]) -> bool {
        let pos: std::collections::HashMap<usize, usize> = sequence
            .iter()
            .enumerate()
            .map(|(i, &j)| (j, i))
            .collect();
        for c in constraints {
            let p_before = pos.get(&c.before).copied().unwrap_or(usize::MAX);
            let p_after = pos.get(&c.after).copied().unwrap_or(usize::MAX);
            if p_before >= p_after {
                return false;
            }
        }
        true
    }
}

/// Wrapper for f64 to enable Ord-based sorting.
#[derive(Debug, Clone, Copy)]
struct OrderedFloat(f64);

impl Eq for OrderedFloat {}

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
