//! Exercise unwind behaviour under the selected development backend.

#[test]
fn catch_unwind_catches_a_development_panic() {
    let result = std::panic::catch_unwind(|| panic!("unwind probe"));
    assert!(result.is_err(), "catch_unwind must catch the probe panic");
}

#[test]
fn spawned_thread_panic_does_not_abort_the_process() {
    let thread = std::thread::spawn(|| panic!("thread probe"));
    assert!(
        thread.join().is_err(),
        "the spawned thread panic must be joinable"
    );
}

#[test]
#[should_panic(expected = "should-panic probe")]
fn should_panic_tests_keep_their_expected_result() {
    panic!("should-panic probe");
}
