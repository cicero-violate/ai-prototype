#![forbid(unsafe_code)]

fn main() {
    match ai::run_demo() {
        Ok(report) => {
            println!("ready_events={}", report.ready_tlog.len());
            println!("repaired_events={}", report.repaired_tlog.len());
            println!("success={}", report.both_succeeded());
        }
        Err(err) => {
            eprintln!("canonical run failed: {err}");
            std::process::exit(1);
        }
    }
}
