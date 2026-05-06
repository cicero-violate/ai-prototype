use ai::validation_harness::{
    root_validation_steps, CHECK_STEP, FAST_TEST_STEP, LOCKFILE_COMPAT_FLAG,
};

#[test]
fn root_validation_requires_lockfile_compatibility_flag() {
    let steps = root_validation_steps();
    assert_eq!(steps.len(), 2);
    for step in &steps {
        assert_eq!(step.args.first(), Some(&LOCKFILE_COMPAT_FLAG));
    }
}

#[test]
fn root_validation_runs_check_before_bounded_fast_tests() {
    let steps = root_validation_steps();
    assert_eq!(steps[0].name, CHECK_STEP);
    assert_eq!(steps[0].args, [LOCKFILE_COMPAT_FLAG, "check", "--all-targets", "--locked"]);

    assert_eq!(steps[1].name, FAST_TEST_STEP);
    assert!(steps[1].args.contains(&"--test"));
    assert!(steps[1].args.contains(&"score_contract"));
}

#[test]
fn validation_command_line_is_stable() {
    let steps = root_validation_steps();
    assert_eq!(
        steps[0].command_line("cargo"),
        "cargo -Znext-lockfile-bump check --all-targets --locked"
    );
}