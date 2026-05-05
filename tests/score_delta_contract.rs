use ai::{Decision, ScoreDelta};

#[test]
fn score_delta_uses_geometric_mean_floor_as_minimum_proxy() {
    let delta = ScoreDelta::new(9, 8, 7);
    assert_eq!(delta.bounded_min(), 7);
    assert_eq!(delta.decision(7), Decision::Allow);
    assert_eq!(delta.decision(8), Decision::Repair);
}