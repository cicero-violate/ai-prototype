use ai::validation_harness::{
    root_validation_steps, API_TRANSPORT_STEP, CHECK_STEP, FAST_TEST_STEP, LIB_UNIT_STEP,
    LOCKFILE_COMPAT_FLAG, VALIDATION_HARNESS_STEP,
};

#[test]
fn root_validation_requires_lockfile_compatibility_flag() {
    let steps = root_validation_steps();
    assert_eq!(steps.len(), 5);
    for step in &steps {
        assert_eq!(step.args.first(), Some(&LOCKFILE_COMPAT_FLAG));
    }
}

#[test]
fn root_validation_runs_check_before_contract_suites() {
    let steps = root_validation_steps();
    let names = steps.iter().map(|step| step.name).collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            CHECK_STEP,
            FAST_TEST_STEP,
            LIB_UNIT_STEP,
            API_TRANSPORT_STEP,
            VALIDATION_HARNESS_STEP,
        ]
    );

    assert_eq!(
        steps[0].args,
        [LOCKFILE_COMPAT_FLAG, "check", "--all-targets", "--locked"]
    );
    assert!(steps[1].args.contains(&"score_contract"));
    assert!(steps[2].args.contains(&"--lib"));
    assert!(steps[3].args.contains(&"api_transport_contract"));
    assert!(steps[4].args.contains(&"validation_harness_contract"));
}

#[test]
fn validation_command_lines_are_stable() {
    let steps = root_validation_steps();
    assert_eq!(
        steps[0].command_line("cargo"),
        "cargo -Znext-lockfile-bump check --all-targets --locked"
    );
    assert_eq!(
        steps[2].command_line("cargo"),
        "cargo -Znext-lockfile-bump test --lib --locked -- --nocapture"
    );
}
