#[cfg(test)]
mod tests {
    use scheduling_rs::*;
    use scheduling_rs::flow_shop::{FlowShopJob, JohnsonAlgorithm};
    use scheduling_rs::precedence::{PrecedenceConstraint, PrecedenceScheduler};
    use scheduling_rs::resource_constrained::{Resource, ResourceConstrainedScheduler, ResourceJob};
    use scheduling_rs::shop::{OpenShopScheduler, JobShopScheduler, ShopTask, JobShopJob};
    use scheduling_rs::fleet::{Worker, FleetTask, FleetScheduler};
    use scheduling_rs::preemptive::PreemptiveScheduler;

    // ── SPT priority rule ──

    #[test]
    fn spt_orders_shortest_first() {
        let jobs = vec![
            Job::new(0, 5.0),
            Job::new(1, 2.0),
            Job::new(2, 8.0),
            Job::new(3, 1.0),
        ];
        let order = priority::apply_rule(&jobs, PriorityRule::SPT);
        assert_eq!(order, vec![3, 1, 0, 2]);
    }

    #[test]
    fn spt_minimizes_total_completion_time() {
        let jobs = vec![
            Job::new(0, 5.0),
            Job::new(1, 2.0),
            Job::new(2, 8.0),
        ];
        let spt_order = priority::apply_rule(&jobs, PriorityRule::SPT);
        let spt_tct = SingleMachineScheduler::total_completion_time(&jobs, &spt_order);
        // Any non-SPT order should be >=
        let order_012 = vec![0, 1, 2];
        let tct_012 = SingleMachineScheduler::total_completion_time(&jobs, &order_012);
        assert!(spt_tct <= tct_012 + 1e-10);
    }

    #[test]
    fn spt_optimality_verification() {
        // SPT is optimal for minimizing total completion time
        let jobs = vec![
            Job::new(0, 3.0),
            Job::new(1, 7.0),
            Job::new(2, 1.0),
            Job::new(3, 5.0),
        ];
        let spt_order = priority::apply_rule(&jobs, PriorityRule::SPT);
        let spt_tct = SingleMachineScheduler::total_completion_time(&jobs, &spt_order);
        // Check all permutations
        let perms = vec![
            vec![0,1,2,3], vec![0,1,3,2], vec![0,2,1,3], vec![0,2,3,1],
            vec![0,3,1,2], vec![0,3,2,1], vec![1,0,2,3], vec![1,0,3,2],
            vec![1,2,0,3], vec![1,2,3,0], vec![1,3,0,2], vec![1,3,2,0],
            vec![2,0,1,3], vec![2,0,3,1], vec![2,1,0,3], vec![2,1,3,0],
            vec![2,3,0,1], vec![2,3,1,0], vec![3,0,1,2], vec![3,0,2,1],
            vec![3,1,0,2], vec![3,1,2,0], vec![3,2,0,1], vec![3,2,1,0],
        ];
        for perm in &perms {
            let tct = SingleMachineScheduler::total_completion_time(&jobs, perm);
            assert!(spt_tct <= tct + 1e-10, "SPT not optimal vs {:?}: {} > {}", perm, spt_tct, tct);
        }
    }

    // ── EDD priority rule ──

    #[test]
    fn edd_orders_earliest_due_first() {
        let jobs = vec![
            Job::new(0, 5.0).with_due_date(10.0),
            Job::new(1, 3.0).with_due_date(5.0),
            Job::new(2, 4.0).with_due_date(8.0),
        ];
        let order = priority::apply_rule(&jobs, PriorityRule::EDD);
        assert_eq!(order, vec![1, 2, 0]);
    }

    #[test]
    fn edd_minimizes_max_lateness() {
        let jobs = vec![
            Job::new(0, 5.0).with_due_date(10.0),
            Job::new(1, 3.0).with_due_date(4.0),
            Job::new(2, 4.0).with_due_date(12.0),
        ];
        let edd_order = priority::apply_rule(&jobs, PriorityRule::EDD);
        let edd_ml = SingleMachineScheduler::max_lateness(&jobs, &edd_order);
        // EDD is optimal for max lateness
        let ml_012 = SingleMachineScheduler::max_lateness(&jobs, &vec![0, 1, 2]);
        assert!(edd_ml <= ml_012 + 1e-10);
    }

    // ── WSPT priority rule ──

    #[test]
    fn wspt_orders_by_ratio() {
        let jobs = vec![
            Job::new(0, 6.0).with_weight(2.0), // ratio 3
            Job::new(1, 4.0).with_weight(4.0), // ratio 1
            Job::new(2, 3.0).with_weight(1.0), // ratio 3
        ];
        let order = priority::apply_rule(&jobs, PriorityRule::WSPT);
        assert_eq!(order[0], 1); // smallest ratio first
    }

    #[test]
    fn wspt_minimizes_weighted_completion() {
        let jobs = vec![
            Job::new(0, 6.0).with_weight(2.0),
            Job::new(1, 4.0).with_weight(4.0),
            Job::new(2, 3.0).with_weight(3.0),
        ];
        let wspt_order = priority::apply_rule(&jobs, PriorityRule::WSPT);
        let wspt_twct = SingleMachineScheduler::total_weighted_completion_time(&jobs, &wspt_order);
        let twct_012 = SingleMachineScheduler::total_weighted_completion_time(&jobs, &vec![0, 1, 2]);
        assert!(wspt_twct <= twct_012 + 1e-10);
    }

    // ── Single machine schedule ──

    #[test]
    fn single_machine_spt_schedule() {
        let jobs = vec![
            Job::new(0, 5.0),
            Job::new(1, 2.0),
            Job::new(2, 3.0),
        ];
        let schedule = SingleMachineScheduler::schedule(&jobs, PriorityRule::SPT);
        assert_eq!(schedule.scheduled_jobs.len(), 3);
        // SPT order: 1, 2, 0
        assert_eq!(schedule.scheduled_jobs[0].job_id, 1);
        assert_eq!(schedule.scheduled_jobs[1].job_id, 2);
        assert_eq!(schedule.scheduled_jobs[2].job_id, 0);
    }

    #[test]
    fn single_machine_makespan() {
        let jobs = vec![
            Job::new(0, 5.0),
            Job::new(1, 3.0),
        ];
        let schedule = SingleMachineScheduler::schedule(&jobs, PriorityRule::FCFS);
        // Makespan = sum of processing times on single machine
        assert!((schedule.makespan() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn single_machine_with_release_date() {
        let jobs = vec![
            Job::new(0, 3.0),
            Job::new(1, 2.0).with_release_date(5.0),
        ];
        let schedule = SingleMachineScheduler::schedule(&jobs, PriorityRule::SPT);
        // Job 1 (shorter) but has release date 5.0
        // SPT order: 1, 0. Job 1 starts at 5.0, ends 7.0. Job 0 starts at 7.0, ends 10.0.
        assert!((schedule.scheduled_jobs[0].start_time - 5.0).abs() < 1e-10);
    }

    // ── Parallel machines ──

    #[test]
    fn parallel_makespan_basic() {
        let jobs = vec![
            Job::new(0, 4.0),
            Job::new(1, 3.0),
            Job::new(2, 2.0),
            Job::new(3, 1.0),
        ];
        let schedule = ParallelMachineScheduler::schedule_lpt(&jobs, 2);
        assert_eq!(schedule.scheduled_jobs.len(), 4);
        assert!(schedule.makespan() > 0.0);
    }

    #[test]
    fn parallel_makespan_lower_bound() {
        let jobs = vec![
            Job::new(0, 4.0),
            Job::new(1, 3.0),
            Job::new(2, 2.0),
        ];
        let lb = ParallelMachineScheduler::makespan_lower_bound(&jobs, 2);
        // total = 9, m=2, so lb = max(4, 4.5) = 4.5
        assert!((lb - 4.5).abs() < 1e-10);
    }

    #[test]
    fn lpt_heuristic_quality() {
        let jobs = vec![
            Job::new(0, 7.0),
            Job::new(1, 6.0),
            Job::new(2, 5.0),
            Job::new(3, 4.0),
            Job::new(4, 3.0),
            Job::new(5, 2.0),
            Job::new(6, 1.0),
        ];
        let schedule = ParallelMachineScheduler::schedule_lpt(&jobs, 3);
        let lb = ParallelMachineScheduler::makespan_lower_bound(&jobs, 3);
        // LPT should be close to optimal (within 4/3 - 1/(3m) factor)
        assert!(schedule.makespan() <= lb * (4.0 / 3.0) + 1e-10);
    }

    #[test]
    fn parallel_schedule_uses_all_machines() {
        let jobs = vec![
            Job::new(0, 2.0),
            Job::new(1, 2.0),
            Job::new(2, 2.0),
            Job::new(3, 2.0),
        ];
        let schedule = ParallelMachineScheduler::schedule_spt(&jobs, 2);
        let machines_used: std::collections::HashSet<usize> =
            schedule.scheduled_jobs.iter().map(|j| j.machine).collect();
        assert_eq!(machines_used.len(), 2);
    }

    // ── Johnson's algorithm ──

    #[test]
    fn johnson_basic() {
        let jobs = vec![
            FlowShopJob { id: 0, time_a: 5.0, time_b: 2.0 },
            FlowShopJob { id: 1, time_a: 1.0, time_b: 6.0 },
            FlowShopJob { id: 2, time_a: 9.0, time_b: 7.0 },
            FlowShopJob { id: 3, time_a: 3.0, time_b: 8.0 },
        ];
        let seq = JohnsonAlgorithm::sequence(&jobs);
        assert_eq!(seq.len(), 4);
        // Verify all jobs are present
        let mut sorted = seq.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1, 2, 3]);
    }

    #[test]
    fn johnson_optimal_makespan() {
        let jobs = vec![
            FlowShopJob { id: 0, time_a: 3.0, time_b: 6.0 },
            FlowShopJob { id: 1, time_a: 2.0, time_b: 8.0 },
            FlowShopJob { id: 2, time_a: 6.0, time_b: 2.0 },
        ];
        let seq = JohnsonAlgorithm::sequence(&jobs);
        let mk = JohnsonAlgorithm::makespan(&jobs, &seq);
        // Johnson's should give optimal makespan
        // Check against all permutations
        use std::collections::BTreeSet;
        let all_perms = vec![
            vec![0,1,2], vec![0,2,1], vec![1,0,2],
            vec![1,2,0], vec![2,0,1], vec![2,1,0],
        ];
        for perm in &all_perms {
            let pmk = JohnsonAlgorithm::makespan(&jobs, perm);
            assert!(mk <= pmk + 1e-10, "Johnson not optimal: {} > {}", mk, pmk);
        }
    }

    #[test]
    fn johnson_makespan_computation() {
        let jobs = vec![
            FlowShopJob { id: 0, time_a: 3.0, time_b: 2.0 },
            FlowShopJob { id: 1, time_a: 2.0, time_b: 4.0 },
        ];
        let seq = JohnsonAlgorithm::sequence(&jobs);
        let mk = JohnsonAlgorithm::makespan(&jobs, &seq);
        // Machine A: 3+2=5 or 2+3=5
        // For order [1,0]: A: 0->2, 1->5; B: 1 starts at max(2,2)=2, ends 6; 0 starts max(6,5)=6, ends 8
        // For order [0,1]: A: 0->3, 1->5; B: 0 starts at 3, ends 5; 1 starts max(5,5)=5, ends 9
        assert!(mk > 0.0);
    }

    #[test]
    fn johnson_empty() {
        let jobs: Vec<FlowShopJob> = vec![];
        let seq = JohnsonAlgorithm::sequence(&jobs);
        assert!(seq.is_empty());
    }

    #[test]
    fn johnson_single_job() {
        let jobs = vec![FlowShopJob { id: 0, time_a: 5.0, time_b: 3.0 }];
        let seq = JohnsonAlgorithm::sequence(&jobs);
        assert_eq!(seq, vec![0]);
        let mk = JohnsonAlgorithm::makespan(&jobs, &seq);
        assert!((mk - 8.0).abs() < 1e-10);
    }

    // ── Schedule struct ──

    #[test]
    fn schedule_makespan() {
        let schedule = Schedule {
            scheduled_jobs: vec![
                ScheduledJob { job_id: 0, start_time: 0.0, end_time: 5.0, machine: 0 },
                ScheduledJob { job_id: 1, start_time: 0.0, end_time: 7.0, machine: 1 },
            ],
            num_machines: 2,
        };
        assert!((schedule.makespan() - 7.0).abs() < 1e-10);
    }

    #[test]
    fn schedule_total_completion_time() {
        let schedule = Schedule {
            scheduled_jobs: vec![
                ScheduledJob { job_id: 0, start_time: 0.0, end_time: 3.0, machine: 0 },
                ScheduledJob { job_id: 1, start_time: 0.0, end_time: 5.0, machine: 1 },
            ],
            num_machines: 2,
        };
        assert!((schedule.total_completion_time() - 8.0).abs() < 1e-10);
    }

    // ── Branch and bound ──

    #[test]
    fn bb_optimal_twct() {
        let jobs = vec![
            Job::new(0, 6.0).with_weight(2.0),
            Job::new(1, 4.0).with_weight(4.0),
            Job::new(2, 3.0).with_weight(3.0),
        ];
        let optimal = BranchAndBound::optimize_twct(&jobs);
        let optimal_cost = SingleMachineScheduler::total_weighted_completion_time(&jobs, &optimal);
        // Check against all permutations
        let perms = vec![
            vec![0,1,2], vec![0,2,1], vec![1,0,2],
            vec![1,2,0], vec![2,0,1], vec![2,1,0],
        ];
        for perm in &perms {
            let cost = SingleMachineScheduler::total_weighted_completion_time(&jobs, perm);
            assert!(optimal_cost <= cost + 1e-10);
        }
    }

    #[test]
    fn bb_optimal_makespan_parallel() {
        let jobs = vec![
            Job::new(0, 3.0),
            Job::new(1, 2.0),
            Job::new(2, 2.0),
        ];
        let schedule = BranchAndBound::optimize_makespan_parallel(&jobs, 2);
        let mk = schedule.makespan();
        let lb = ParallelMachineScheduler::makespan_lower_bound(&jobs, 2);
        assert!(mk >= lb - 1e-10);
        // Optimal should be 3 (3|2+2 = 3|4 -> but actually best is 2+2|3 = 4... or 3+2|2 = 5)
        // total=7, m=2, lb=max(3, 3.5)=3.5. Best assignment: [0,1] on m0, [2] on m1 -> 5,2 -> mk=5
        // Or [0] on m0, [1,2] on m1 -> 3,4 -> mk=4
        // Or [0,2] on m0, [1] on m1 -> 5,2 -> mk=5
        // Or [1,0] on m0, [2] on m1 -> 5,2 -> mk=5
        // Best is mk=3.5? No, mk must be integer-ish... mk=4 is achievable
        assert!(mk <= 4.0 + 1e-10);
    }

    #[test]
    fn bb_empty_jobs() {
        let jobs: Vec<Job> = vec![];
        let optimal = BranchAndBound::optimize_twct(&jobs);
        assert!(optimal.is_empty());
    }

    #[test]
    fn bb_single_job() {
        let jobs = vec![Job::new(0, 5.0).with_weight(3.0)];
        let optimal = BranchAndBound::optimize_twct(&jobs);
        assert_eq!(optimal, vec![0]);
        let cost = SingleMachineScheduler::total_weighted_completion_time(&jobs, &optimal);
        assert!((cost - 15.0).abs() < 1e-10);
    }

    // ── Precedence constraints ──

    #[test]
    fn topological_sort_basic() {
        let constraints = vec![
            PrecedenceConstraint { before: 0, after: 1 },
            PrecedenceConstraint { before: 1, after: 2 },
        ];
        let order = PrecedenceScheduler::topological_sort(3, &constraints);
        assert_eq!(order.len(), 3);
        // 0 must come before 1, 1 before 2
        let pos: std::collections::HashMap<usize, usize> =
            order.iter().enumerate().map(|(i, &j)| (j, i)).collect();
        assert!(pos[&0] < pos[&1]);
        assert!(pos[&1] < pos[&2]);
    }

    #[test]
    fn precedence_schedule_respects_order() {
        let jobs = vec![
            Job::new(0, 3.0),
            Job::new(1, 2.0),
            Job::new(2, 4.0),
        ];
        let constraints = vec![
            PrecedenceConstraint { before: 2, after: 0 },
            PrecedenceConstraint { before: 2, after: 1 },
        ];
        let schedule = PrecedenceScheduler::schedule(&jobs, &constraints);
        assert!(PrecedenceScheduler::is_valid_sequence(&schedule.job_order(), &[
            PrecedenceConstraint { before: 2, after: 0 },
            PrecedenceConstraint { before: 2, after: 1 },
        ]));
    }

    #[test]
    fn precedence_spt_combination() {
        let jobs = vec![
            Job::new(0, 5.0),
            Job::new(1, 2.0),
            Job::new(2, 3.0),
        ];
        let constraints = vec![
            PrecedenceConstraint { before: 1, after: 2 },
        ];
        let schedule = PrecedenceScheduler::schedule_spt_with_precedence(&jobs, &constraints);
        let order = schedule.job_order();
        assert!(PrecedenceScheduler::is_valid_sequence(&order, &constraints));
    }

    #[test]
    fn is_valid_sequence_check() {
        let constraints = vec![
            PrecedenceConstraint { before: 0, after: 1 },
        ];
        assert!(PrecedenceScheduler::is_valid_sequence(&vec![0, 1], &constraints));
        assert!(!PrecedenceScheduler::is_valid_sequence(&vec![1, 0], &constraints));
    }

    // ── Due date scheduling ──

    #[test]
    fn due_date_minimize_max_lateness() {
        let jobs = vec![
            Job::new(0, 5.0).with_due_date(10.0),
            Job::new(1, 3.0).with_due_date(4.0),
            Job::new(2, 4.0).with_due_date(15.0),
        ];
        let schedule = DueDateScheduler::minimize_max_lateness(&jobs);
        let ml = DueDateScheduler::max_lateness(&jobs, &schedule);
        // EDD order: 1, 0, 2
        // Completion times: 3, 8, 12
        // Lateness: max(3-4, 8-10, 12-15) = max(-1, -2, -3) = -1
        assert!(ml < 0.0 + 1e-10); // No tardiness
    }

    #[test]
    fn due_date_total_tardiness() {
        let jobs = vec![
            Job::new(0, 5.0).with_due_date(3.0),
            Job::new(1, 2.0).with_due_date(8.0),
        ];
        let schedule = DueDateScheduler::minimize_total_tardiness(&jobs);
        let tt = DueDateScheduler::total_tardiness(&jobs, &schedule);
        assert!(tt >= 0.0);
    }

    #[test]
    fn moore_algorithm_minimizes_tardy_count() {
        let jobs = vec![
            Job::new(0, 5.0).with_due_date(10.0),
            Job::new(1, 6.0).with_due_date(6.0),
            Job::new(2, 4.0).with_due_date(15.0),
        ];
        let schedule = DueDateScheduler::minimize_num_tardy(&jobs);
        let num_tardy = DueDateScheduler::num_tardy(&jobs, &schedule);
        // With EDD: 1(6,dd=6), 0(11,dd=10), 2(15,dd=15) -> 1 tardy
        // Moore should remove job 0 or 1, achieving 0 or 1 tardy
        assert!(num_tardy <= 1);
    }

    #[test]
    fn due_date_no_tardiness_possible() {
        let jobs = vec![
            Job::new(0, 2.0).with_due_date(10.0),
            Job::new(1, 3.0).with_due_date(10.0),
        ];
        let schedule = DueDateScheduler::minimize_max_lateness(&jobs);
        let ml = DueDateScheduler::max_lateness(&jobs, &schedule);
        assert!(ml <= 0.0 + 1e-10);
    }

    // ── Preemptive scheduling ──

    #[test]
    fn srpt_basic() {
        let jobs = vec![
            Job::new(0, 5.0),
            Job::new(1, 3.0),
            Job::new(2, 2.0),
        ];
        let segments = PreemptiveScheduler::srpt(&jobs);
        // All jobs available at 0, so SPT order without preemption: 2,1,0
        let total_processing: f64 = segments.iter().map(|s| s.end - s.start).sum();
        assert!((total_processing - 10.0).abs() < 1e-10);
    }

    #[test]
    fn srpt_with_release_dates() {
        let jobs = vec![
            Job::new(0, 5.0),
            Job::new(1, 3.0).with_release_date(2.0),
        ];
        let segments = PreemptiveScheduler::srpt(&jobs);
        // Job 0 starts at 0, at t=2 job 1 arrives with shorter remaining (3 < 3)
        // So job 0 gets preempted at t=2, job 1 runs 2->5, job 0 resumes 5->7
        assert!(segments.len() >= 2);
    }

    #[test]
    fn srpt_total_completion() {
        let jobs = vec![
            Job::new(0, 4.0),
            Job::new(1, 2.0),
            Job::new(2, 1.0),
        ];
        let segments = PreemptiveScheduler::srpt(&jobs);
        let tct = PreemptiveScheduler::total_completion_time(&jobs, &segments);
        // SRPT minimizes total completion time for preemptive case
        assert!(tct > 0.0);
    }

    #[test]
    fn preemptive_edd_basic() {
        let jobs = vec![
            Job::new(0, 5.0).with_due_date(10.0),
            Job::new(1, 3.0).with_due_date(5.0),
        ];
        let segments = PreemptiveScheduler::preemptive_edd(&jobs);
        assert_eq!(segments[0].job_id, 1); // Earlier due date first
    }

    // ── Resource-constrained scheduling ──

    #[test]
    fn resource_constrained_basic() {
        let jobs = vec![
            ResourceJob {
                job: Job::new(0, 3.0),
                resource_requirements: vec![(0, 2.0)],
            },
            ResourceJob {
                job: Job::new(1, 4.0),
                resource_requirements: vec![(0, 3.0)],
            },
            ResourceJob {
                job: Job::new(2, 2.0),
                resource_requirements: vec![(0, 1.0)],
            },
        ];
        let resources = vec![Resource { id: 0, capacity: 3.0 }];
        let schedule = ResourceConstrainedScheduler::schedule(&jobs, &resources, &[]);
        assert_eq!(schedule.scheduled_jobs.len(), 3);
    }

    #[test]
    fn resource_constrained_with_precedence() {
        let jobs = vec![
            ResourceJob {
                job: Job::new(0, 3.0),
                resource_requirements: vec![(0, 1.0)],
            },
            ResourceJob {
                job: Job::new(1, 2.0),
                resource_requirements: vec![(0, 1.0)],
            },
        ];
        let resources = vec![Resource { id: 0, capacity: 5.0 }];
        let schedule = ResourceConstrainedScheduler::schedule(&jobs, &resources, &[(0, 1)]);
        assert_eq!(schedule.scheduled_jobs.len(), 2);
        // Job 1 must start after job 0
        let j0 = schedule.scheduled_jobs.iter().find(|j| j.job_id == 0).unwrap();
        let j1 = schedule.scheduled_jobs.iter().find(|j| j.job_id == 1).unwrap();
        assert!(j1.start_time >= j0.end_time - 1e-10);
    }

    // ── Open shop scheduling ──

    #[test]
    fn open_shop_basic() {
        let tasks = vec![
            ShopTask { job_id: 0, machine: 0, processing_time: 3.0 },
            ShopTask { job_id: 0, machine: 1, processing_time: 2.0 },
            ShopTask { job_id: 1, machine: 0, processing_time: 2.0 },
            ShopTask { job_id: 1, machine: 1, processing_time: 4.0 },
        ];
        let schedule = OpenShopScheduler::schedule(&tasks, 2);
        assert_eq!(schedule.scheduled_jobs.len(), 4);
        let mk = OpenShopScheduler::makespan(&schedule);
        assert!(mk > 0.0);
    }

    #[test]
    fn open_shop_no_overlap_same_job() {
        let tasks = vec![
            ShopTask { job_id: 0, machine: 0, processing_time: 3.0 },
            ShopTask { job_id: 0, machine: 1, processing_time: 2.0 },
        ];
        let schedule = OpenShopScheduler::schedule(&tasks, 2);
        // Same job can't be on two machines at once
        let j0_tasks: Vec<_> = schedule.scheduled_jobs.iter().filter(|j| j.job_id == 0).collect();
        if j0_tasks.len() == 2 {
            assert!(j0_tasks[0].end_time <= j0_tasks[1].start_time + 1e-10
                || j0_tasks[1].end_time <= j0_tasks[0].start_time + 1e-10);
        }
    }

    // ── Job shop scheduling ──

    #[test]
    fn job_shop_basic() {
        let jobs = vec![
            JobShopJob {
                id: 0,
                operations: vec![(0, 3.0), (1, 2.0)],
            },
            JobShopJob {
                id: 1,
                operations: vec![(1, 4.0), (0, 1.0)],
            },
        ];
        let schedule = JobShopScheduler::schedule(&jobs, 2);
        assert_eq!(schedule.scheduled_jobs.len(), 4);
    }

    #[test]
    fn job_shop_respects_route() {
        let jobs = vec![
            JobShopJob {
                id: 0,
                operations: vec![(0, 3.0), (1, 2.0)],
            },
        ];
        let schedule = JobShopScheduler::schedule(&jobs, 2);
        let ops: Vec<_> = schedule.scheduled_jobs.iter()
            .filter(|j| j.job_id == 0)
            .collect();
        assert_eq!(ops.len(), 2);
        // Machine 0 op must finish before machine 1 op starts
        let m0 = ops.iter().find(|j| j.machine == 0).unwrap();
        let m1 = ops.iter().find(|j| j.machine == 1).unwrap();
        assert!(m1.start_time >= m0.end_time - 1e-10);
    }

    // ── Worker fleet scheduling ──

    #[test]
    fn fleet_basic() {
        let tasks = vec![
            FleetTask {
                id: 0,
                processing_time: 4.0,
                task_type: "coding".to_string(),
                priority: 1.0,
                deadline: None,
                dependencies: vec![],
            },
            FleetTask {
                id: 1,
                processing_time: 3.0,
                task_type: "coding".to_string(),
                priority: 2.0,
                deadline: None,
                dependencies: vec![],
            },
        ];
        let agents = vec![
            Worker {
                id: 0,
                capacity: 1.0,
                specializations: vec!["coding".to_string()],
            },
            Worker {
                id: 1,
                capacity: 1.0,
                specializations: vec!["coding".to_string()],
            },
        ];
        let schedule = FleetScheduler::schedule(&tasks, &agents);
        assert_eq!(schedule.scheduled_jobs.len(), 2);
    }

    #[test]
    fn fleet_respects_specialization() {
        let tasks = vec![
            FleetTask {
                id: 0,
                processing_time: 4.0,
                task_type: "coding".to_string(),
                priority: 1.0,
                deadline: None,
                dependencies: vec![],
            },
            FleetTask {
                id: 1,
                processing_time: 3.0,
                task_type: "review".to_string(),
                priority: 1.0,
                deadline: None,
                dependencies: vec![],
            },
        ];
        let agents = vec![
            Worker {
                id: 0,
                capacity: 1.0,
                specializations: vec!["coding".to_string()],
            },
            Worker {
                id: 1,
                capacity: 1.0,
                specializations: vec!["review".to_string()],
            },
        ];
        let schedule = FleetScheduler::schedule(&tasks, &agents);
        let coding_task = schedule.scheduled_jobs.iter().find(|j| j.job_id == 0).unwrap();
        let review_task = schedule.scheduled_jobs.iter().find(|j| j.job_id == 1).unwrap();
        assert_eq!(coding_task.machine, 0);
        assert_eq!(review_task.machine, 1);
    }

    #[test]
    fn fleet_capacity_speedup() {
        let tasks = vec![
            FleetTask {
                id: 0,
                processing_time: 4.0,
                task_type: "general".to_string(),
                priority: 1.0,
                deadline: None,
                dependencies: vec![],
            },
        ];
        let agents = vec![
            Worker {
                id: 0,
                capacity: 2.0, // 2x speed
                specializations: vec![],
            },
        ];
        let schedule = FleetScheduler::schedule(&tasks, &agents);
        let task = &schedule.scheduled_jobs[0];
        // 4.0 / 2.0 = 2.0 actual time
        assert!((task.end_time - task.start_time - 2.0).abs() < 1e-10);
    }

    #[test]
    fn fleet_dependencies() {
        let tasks = vec![
            FleetTask {
                id: 0,
                processing_time: 3.0,
                task_type: "general".to_string(),
                priority: 1.0,
                deadline: None,
                dependencies: vec![],
            },
            FleetTask {
                id: 1,
                processing_time: 2.0,
                task_type: "general".to_string(),
                priority: 2.0,
                deadline: None,
                dependencies: vec![0], // depends on task 0
            },
        ];
        let agents = vec![
            Worker {
                id: 0,
                capacity: 1.0,
                specializations: vec![],
            },
        ];
        let schedule = FleetScheduler::schedule(&tasks, &agents);
        let t0 = schedule.scheduled_jobs.iter().find(|j| j.job_id == 0).unwrap();
        let t1 = schedule.scheduled_jobs.iter().find(|j| j.job_id == 1).unwrap();
        assert!(t1.start_time >= t0.end_time - 1e-10);
    }

    #[test]
    fn fleet_utilization() {
        let tasks = vec![
            FleetTask {
                id: 0,
                processing_time: 4.0,
                task_type: "general".to_string(),
                priority: 1.0,
                deadline: None,
                dependencies: vec![],
            },
            FleetTask {
                id: 1,
                processing_time: 4.0,
                task_type: "general".to_string(),
                priority: 1.0,
                deadline: None,
                dependencies: vec![],
            },
        ];
        let agents = vec![
            Worker {
                id: 0,
                capacity: 1.0,
                specializations: vec![],
            },
            Worker {
                id: 1,
                capacity: 1.0,
                specializations: vec![],
            },
        ];
        let schedule = FleetScheduler::schedule(&tasks, &agents);
        let util = FleetScheduler::worker_utilization(&schedule);
        assert_eq!(util.len(), 2);
        // Both agents should be 100% utilized
        for u in &util {
            assert!((u - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn fleet_load_balance() {
        let tasks = vec![
            FleetTask {
                id: 0,
                processing_time: 4.0,
                task_type: "general".to_string(),
                priority: 1.0,
                deadline: None,
                dependencies: vec![],
            },
            FleetTask {
                id: 1,
                processing_time: 4.0,
                task_type: "general".to_string(),
                priority: 1.0,
                deadline: None,
                dependencies: vec![],
            },
        ];
        let agents = vec![
            Worker {
                id: 0,
                capacity: 1.0,
                specializations: vec![],
            },
            Worker {
                id: 1,
                capacity: 1.0,
                specializations: vec![],
            },
        ];
        let schedule = FleetScheduler::schedule(&tasks, &agents);
        let balance = FleetScheduler::load_balance(&schedule);
        // Perfectly balanced: std dev = 0
        assert!(balance < 1e-10);
    }

    // ── FCFS and LPT rules ──

    #[test]
    fn fcfs_preserves_id_order() {
        let jobs = vec![
            Job::new(2, 5.0),
            Job::new(0, 3.0),
            Job::new(1, 4.0),
        ];
        let order = priority::apply_rule(&jobs, PriorityRule::FCFS);
        assert_eq!(order, vec![1, 2, 0]); // by job id: 0,1,2 -> indices 1,2,0
    }

    #[test]
    fn lpt_orders_longest_first() {
        let jobs = vec![
            Job::new(0, 3.0),
            Job::new(1, 7.0),
            Job::new(2, 5.0),
        ];
        let order = priority::apply_rule(&jobs, PriorityRule::LPT);
        assert_eq!(order, vec![1, 2, 0]);
    }

    // ── Schedule from sequence (B&B) ──

    #[test]
    fn schedule_from_sequence() {
        let jobs = vec![
            Job::new(0, 3.0),
            Job::new(1, 5.0),
            Job::new(2, 2.0),
        ];
        let schedule = BranchAndBound::schedule_from_sequence(&jobs, &vec![2, 0, 1]);
        assert_eq!(schedule.scheduled_jobs.len(), 3);
        assert!((schedule.scheduled_jobs[0].start_time).abs() < 1e-10);
        assert!((schedule.scheduled_jobs[0].end_time - 2.0).abs() < 1e-10);
        assert!((schedule.scheduled_jobs[1].start_time - 2.0).abs() < 1e-10);
        assert!((schedule.scheduled_jobs[1].end_time - 5.0).abs() < 1e-10);
    }

    // ── Johnson's algorithm schedule output ──

    #[test]
    fn johnson_schedule_output() {
        let jobs = vec![
            FlowShopJob { id: 0, time_a: 3.0, time_b: 6.0 },
            FlowShopJob { id: 1, time_a: 2.0, time_b: 8.0 },
        ];
        let schedule = JohnsonAlgorithm::schedule(&jobs);
        assert_eq!(schedule.scheduled_jobs.len(), 4); // 2 jobs × 2 machines
        assert!((schedule.makespan() - 16.0).abs() < 1e-10);
    }

    // ── Serialization ──

    #[test]
    fn job_serialization() {
        let job = Job::new(42, 5.5).with_weight(2.0).with_due_date(10.0);
        let json = serde_json::to_string(&job).unwrap();
        let deserialized: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 42);
        assert!((deserialized.processing_time - 5.5).abs() < 1e-10);
        assert!((deserialized.weight - 2.0).abs() < 1e-10);
    }

    #[test]
    fn schedule_serialization() {
        let schedule = Schedule {
            scheduled_jobs: vec![
                ScheduledJob { job_id: 0, start_time: 0.0, end_time: 5.0, machine: 0 },
            ],
            num_machines: 1,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        let deserialized: Schedule = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.num_machines, 1);
        assert_eq!(deserialized.scheduled_jobs.len(), 1);
    }

    // ── Edge cases ──

    #[test]
    fn empty_jobs_schedule() {
        let jobs: Vec<Job> = vec![];
        let schedule = SingleMachineScheduler::schedule(&jobs, PriorityRule::SPT);
        assert!(schedule.scheduled_jobs.is_empty());
        assert!((schedule.makespan() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn parallel_empty_jobs() {
        let jobs: Vec<Job> = vec![];
        let schedule = ParallelMachineScheduler::schedule_spt(&jobs, 4);
        assert!(schedule.scheduled_jobs.is_empty());
    }

    #[test]
    fn single_machine_one_job() {
        let jobs = vec![Job::new(0, 7.0)];
        let schedule = SingleMachineScheduler::schedule(&jobs, PriorityRule::SPT);
        assert!((schedule.makespan() - 7.0).abs() < 1e-10);
    }

    #[test]
    fn precedence_diamond() {
        // 0 -> 1, 0 -> 2, 1 -> 3, 2 -> 3
        let constraints = vec![
            PrecedenceConstraint { before: 0, after: 1 },
            PrecedenceConstraint { before: 0, after: 2 },
            PrecedenceConstraint { before: 1, after: 3 },
            PrecedenceConstraint { before: 2, after: 3 },
        ];
        let order = PrecedenceScheduler::topological_sort(4, &constraints);
        assert!(PrecedenceScheduler::is_valid_sequence(&order, &constraints));
    }
}
