#![forbid(unsafe_code)]

fn main() {
    let path = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(ai::default_canonical_tlog_path);
    match ai::introspect_canonical_tlog(&path) {
        Ok(report) => {
            println!("canonical_path={}", report.canonical_path.display());
            println!(
                "latest_phase={}",
                report.latest_phase.as_deref().unwrap_or("missing")
            );
            println!("event_count={}", report.event_count);
            println!(
                "latest_evaluator_result={}",
                report
                    .latest_evaluator_result
                    .as_deref()
                    .unwrap_or("missing")
            );
            println!(
                "latest_validation_result={}",
                report
                    .latest_validation_result
                    .as_deref()
                    .unwrap_or("missing")
            );
            println!(
                "latest_score_report_hash={}",
                report
                    .latest_score_report_hash
                    .map(|hash| hash.to_string())
                    .unwrap_or_else(|| "missing".to_string())
            );
            println!("worker_state={}", report.worker_state.status);
            if !report.worker_state.legacy_paths.is_empty() {
                let paths = report
                    .worker_state
                    .legacy_paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                println!("legacy_worker_tlogs={paths}");
            }
        }
        Err(err) => {
            eprintln!("tlog introspection failed: {err}");
            std::process::exit(1);
        }
    }
}
