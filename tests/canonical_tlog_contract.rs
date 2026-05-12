use std::process::Command;
use std::thread;
use std::time::Duration;

fn temp_root(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("target/test-tmp/canonical-tlog")
        .join(format!("{name}-{}", std::process::id()))
}

fn passing_eval_record() -> ai::EvalRecord {
    ai::EvalRecord {
        score: 91,
        threshold_used: 80,
        dimensions: vec![
            ai::EvalDimension {
                id: "correctness",
                score: 90,
                threshold: 80,
            },
            ai::EvalDimension {
                id: "replay",
                score: 92,
                threshold: 80,
            },
        ],
    }
}

#[test]
fn canonical_ledger_replays_and_introspects_from_repo_root() {
    let root = temp_root("replay");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("tlog")).expect("test tlog dir");
    let canonical = root.join("tlog/canon-agent.tlog.ndjson");
    let relative = canonical.to_string_lossy().to_string();

    let (state, tlog) =
        ai::run_until_done(ai::State::default(), ai::RuntimeConfig::default()).expect("run");
    assert_eq!(state.phase, ai::Phase::Done);
    ai::write_tlog_ndjson(&canonical, &tlog).expect("write canonical tlog");
    ai::append_eval_scorecard_receipt_ndjson(&canonical, passing_eval_record().scorecard_receipt())
        .expect("append eval result");
    ai::append_validation_result_ndjson(&canonical, "root_validate", true, 0x5151)
        .expect("append validation result");
    ai::append_score_report_update_ndjson(&canonical, 0x5c02e).expect("append score update");

    let replay = ai::replay_report_ndjson(ai::State::default(), &canonical).expect("replay");
    assert_eq!(replay.event_count, tlog.len());
    assert_eq!(replay.final_state.phase, ai::Phase::Done);

    let report = ai::introspect_canonical_tlog(&canonical).expect("introspect");
    assert_eq!(report.latest_phase.as_deref(), Some("Done"));
    assert_eq!(report.event_count, tlog.len());
    assert_eq!(report.latest_evaluator_result.as_deref(), Some("pass"));
    assert_eq!(
        report.latest_validation_result.as_deref(),
        Some("root_validate:pass")
    );
    assert_eq!(report.latest_score_report_hash, Some(0x5c02e));

    let output = Command::new(env!("CARGO_BIN_EXE_tlog_introspect"))
        .arg(relative)
        .output()
        .expect("tlog_introspect should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("latest_phase=Done"));
    assert!(stdout.contains("latest_evaluator_result=pass"));
    assert!(stdout.contains("latest_validation_result=root_validate:pass"));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn root_validate_mirrors_evaluator_result_into_canonical_ledger() {
    let root = temp_root("root-validate");
    let _ = std::fs::remove_dir_all(&root);
    let canonical = root.join("tlog/canon-agent.tlog.ndjson");

    let output = Command::new(env!("CARGO_BIN_EXE_root_validate"))
        .arg("--policy-reuse-evidence-external-evaluator-result-smoke")
        .env("AI_CANONICAL_TLOG", &canonical)
        .output()
        .expect("root_validate evaluator mode should run");
    assert!(output.status.success());

    let report = ai::introspect_canonical_tlog(&canonical).expect("introspect");
    assert_eq!(report.latest_evaluator_result.as_deref(), Some("pass"));
    assert_eq!(
        report.latest_validation_result.as_deref(),
        Some("policy-reuse-evidence-external-evaluator-result-smoke:pass")
    );
    assert_eq!(report.event_count, 0);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn stale_legacy_worker_tlogs_are_reported() {
    let root = temp_root("legacy");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("tlog")).expect("test tlog dir");
    let legacy = root.join("tlog/worker-tlog.ndjson");
    let canonical = root.join("tlog/canon-agent.tlog.ndjson");

    let (state, legacy_tlog) =
        ai::run_until_done(ai::State::ready(), ai::RuntimeConfig::default()).expect("legacy run");
    assert_eq!(state.phase, ai::Phase::Done);
    ai::write_tlog_ndjson(&legacy, &legacy_tlog).expect("write legacy worker tlog");
    thread::sleep(Duration::from_millis(30));
    ai::write_tlog_ndjson(&canonical, &legacy_tlog).expect("write canonical tlog");

    let report = ai::introspect_canonical_tlog(&canonical).expect("introspect");
    assert_eq!(report.worker_state.status, "stale_legacy_worker_tlog");
    assert_eq!(report.worker_state.legacy_paths, vec![legacy]);

    let _ = std::fs::remove_dir_all(root);
}
