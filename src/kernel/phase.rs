#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Phase {
    Delta = 1,
    Invariant = 2,
    Analysis = 3,
    Judgment = 4,
    Plan = 5,
    Execute = 6,
    Verify = 7,
    Eval = 8,
    Recovery = 9,
    Learn = 10,
    Persist = 11,
    Done = 12,
}

pub const PHASES: [Phase; 12] = [
    Phase::Delta,
    Phase::Invariant,
    Phase::Analysis,
    Phase::Judgment,
    Phase::Plan,
    Phase::Execute,
    Phase::Verify,
    Phase::Eval,
    Phase::Recovery,
    Phase::Learn,
    Phase::Persist,
    Phase::Done,
];
