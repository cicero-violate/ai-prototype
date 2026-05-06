use ai::{Packet, PlanDecision, PlanRecord};

#[test]
fn planning_record_decomposes_objective_with_lineage() {
    let mut packet = Packet::empty();
    packet.objective_id = 42;
    packet.objective_required_tasks = 4;
    packet.objective_done_tasks = 1;
    packet.revision = 9;

    let record = PlanRecord::from_packet(packet);

    assert_eq!(record.task_count, 4);
    assert_eq!(record.completed_tasks, 1);
    assert_eq!(record.task_id, 4202);
    assert_eq!(record.ready_tasks, 1);
    assert_eq!(record.tasks.iter().filter(|task| task.ready).count(), 1);
    assert_eq!(record.tasks[0].depends_on, 0);
    assert_eq!(record.tasks[1].depends_on, 4201);
    assert_eq!(record.tasks[2].depends_on, 4202);
    assert!(record.is_valid());

    let mut tampered = record.clone();
    tampered.tasks[1].depends_on = 0;
    assert!(!tampered.is_valid());
}

#[test]
fn planning_record_blocks_when_all_tasks_complete() {
    let mut packet = Packet::empty();
    packet.objective_id = 7;
    packet.objective_required_tasks = 2;
    packet.objective_done_tasks = 2;

    let record = PlanRecord::from_packet(packet);

    assert_eq!(record.ready_tasks, 0);
    assert_eq!(record.decision(), PlanDecision::Blocked);
    assert!(!record.is_valid());
}
