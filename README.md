# scheduling-rs

**Scheduling algorithms for real-world fleet and job management.**

## What This Does

- **Job model** — jobs with processing times, weights, due dates, release dates, and deadlines; schedules with computed objectives (makespan, total completion time, weighted completion time, max lateness, total tardiness)
- **Priority rules** — SPT, EDD, WSPT, LPT, FCFS dispatching heuristics
- **Single-machine scheduling** — schedule jobs on one machine using any priority rule
- **Parallel-machine scheduling** — identical parallel machines with list scheduling and makespan lower bounds
- **Flow shop** — Johnson's algorithm for the optimal 2-machine flow shop (F2 ‖ C_max)
- **Branch and bound** — exact optimization for total weighted completion time (single machine) and makespan (parallel machines)
- **Preemptive scheduling** — Shortest Remaining Processing Time (SRPT) and preemptive EDD
- **Due-date objectives** — minimize L_max, minimize ΣT_j, minimize number of tardy jobs (Moore's algorithm)
- **Precedence constraints** — topological sorting, SPT-with-precedence scheduling
- **Resource-constrained scheduling** — serial schedule generation scheme with resource capacity tracking
- **Shop scheduling** — open shop (greedy/LPT dispatching) and job shop (greedy route-based dispatching)
- **Fleet scheduling** — multi-worker task assignment with specializations, dependencies, capacity scaling, utilization and load-balance metrics

## Install

```toml
[dependencies]
scheduling-rs = "0.1.0"
```

Requires Rust 2021 edition.

## Quick Start

### Single Machine with Priority Rules

```rust
use scheduling_rs::*;

let jobs = vec![
    Job::new(0, 5.0).with_due_date(10.0),
    Job::new(1, 3.0).with_due_date(7.0).with_weight(2.0),
    Job::new(2, 8.0).with_due_date(20.0),
    Job::new(3, 2.0).with_due_date(5.0),
];

let schedule = SingleMachineScheduler::schedule(&jobs, PriorityRule::SPT);
println!("Makespan: {}", schedule.makespan());
```

### Johnson's Algorithm (2-Machine Flow Shop)

```rust
use scheduling_rs::flow_shop::{JohnsonAlgorithm, FlowShopJob};

let jobs = vec![
    FlowShopJob { id: 0, time_a: 3.0, time_b: 6.0 },
    FlowShopJob { id: 1, time_a: 8.0, time_b: 2.0 },
];
let schedule = JohnsonAlgorithm::schedule(&jobs);
println!("Optimal makespan: {}", schedule.makespan());
```

### Fleet Scheduling

```rust
use scheduling_rs::fleet::{FleetScheduler, Worker, FleetTask};

let workers = vec![
    Worker { id: 0, capacity: 1.0, specializations: vec!["compute".into()] },
    Worker { id: 1, capacity: 2.0, specializations: vec!["compute".into(), "io".into()] },
];
let tasks = vec![
    FleetTask { id: 0, processing_time: 5.0, task_type: "compute".into(), priority: 1.0, deadline: Some(10.0), dependencies: vec![] },
    FleetTask { id: 1, processing_time: 3.0, task_type: "io".into(), priority: 2.0, deadline: None, dependencies: vec![0] },
];
let schedule = FleetScheduler::schedule(&tasks, &workers);
```

## License

MIT OR Apache-2.0
