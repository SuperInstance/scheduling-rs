use crate::job::Job;
use serde::{Deserialize, Serialize};

/// Priority rules for sequencing jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriorityRule {
    /// Shortest Processing Time first
    SPT,
    /// Earliest Due Date first
    EDD,
    /// Weighted Shortest Processing Time first (ratio p_j / w_j, ascending)
    WSPT,
    /// Longest Processing Time first
    LPT,
    /// First Come First Served (by job id)
    FCFS,
}

/// Apply a priority rule and return job indices in sequence order.
pub fn apply_rule(jobs: &[Job], rule: PriorityRule) -> Vec<usize> {
    let mut indexed: Vec<(usize, &Job)> = jobs.iter().enumerate().collect();
    match rule {
        PriorityRule::SPT => {
            indexed.sort_by(|a, b| a.1.processing_time.partial_cmp(&b.1.processing_time).unwrap())
        }
        PriorityRule::EDD => {
            indexed.sort_by(|a, b| {
                let da = a.1.due_date.unwrap_or(f64::INFINITY);
                let db = b.1.due_date.unwrap_or(f64::INFINITY);
                da.partial_cmp(&db).unwrap()
            })
        }
        PriorityRule::WSPT => {
            indexed.sort_by(|a, b| {
                let ra = a.1.processing_time / a.1.weight;
                let rb = b.1.processing_time / b.1.weight;
                ra.partial_cmp(&rb).unwrap()
            })
        }
        PriorityRule::LPT => {
            indexed.sort_by(|a, b| b.1.processing_time.partial_cmp(&a.1.processing_time).unwrap())
        }
        PriorityRule::FCFS => indexed.sort_by_key(|a| a.1.id),
    }
    indexed.iter().map(|(i, _)| *i).collect()
}
