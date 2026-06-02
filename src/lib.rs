//! # scheduling-rs
//!
//! Scheduling theory — optimal resource allocation over time.
//!
//! Covers single-machine and parallel-machine scheduling, priority rules
//! (SPT, EDD, WSPT), Johnson's algorithm for 2-machine flow shops,
//! branch-and-bound optimal scheduling, preemptive and non-preemptive
//! scheduling, due-date-based objectives, precedence constraints,
//! resource-constrained scheduling, open/job shop basics, and fleet
//! scheduling.

pub mod job;
pub mod priority;
pub mod single_machine;
pub mod parallel_machine;
pub mod flow_shop;
pub mod branch_and_bound;
pub mod preemptive;
pub mod due_date;
pub mod precedence;
pub mod resource_constrained;
pub mod shop;
pub mod fleet;

pub use job::{Job, Schedule, ScheduledJob};
pub use priority::PriorityRule;
pub use single_machine::SingleMachineScheduler;
pub use parallel_machine::ParallelMachineScheduler;
pub use flow_shop::JohnsonAlgorithm;
pub use branch_and_bound::BranchAndBound;
pub use preemptive::PreemptiveScheduler;
pub use due_date::DueDateScheduler;
pub use precedence::PrecedenceScheduler;
pub use resource_constrained::ResourceConstrainedScheduler;
pub use shop::{OpenShopScheduler, JobShopScheduler};
pub use fleet::FleetScheduler;
