use ai::{tick, verify_tlog, Phase, RuntimeConfig, State, TLog};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = RuntimeConfig::default();
    let mut state = State::default();
    let mut tlog: TLog = Vec::new();

    println!("start phase={:?}", state.phase);

    for step in 1..=cfg.max_steps {
        if state.phase == Phase::Done {
            break;
        }

        tick(&mut state, &mut tlog, cfg)?;
        let event = tlog.last().expect("tick appends one event");

        println!(
            "step={step:02} seq={} {:?}->{:?} kind={:?} cause={:?} evidence={:?} decision={:?} gate={:?} failure={:?} recovery={:?} hash={:016x}",
            event.seq,
            event.from,
            event.to,
            event.kind,
            event.cause,
            event.evidence,
            event.decision,
            event.affected_gate,
            event.failure,
            event.recovery_action,
            event.self_hash,
        );
    }

    verify_tlog(&tlog)?;
    println!(
        "done phase={:?} success={} events={}",
        state.phase,
        state.is_success(),
        tlog.len()
    );

    Ok(())
}
