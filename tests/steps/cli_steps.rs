//! Step definitions backing the behavioural CLI scenarios.

use std::{error::Error, process::Command};

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use rstest::fixture;
use rstest_bdd_macros::{given, then, when};
use tempfile::TempDir;

pub(crate) type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub(crate) type CliFixture = TestResult<CliState>;

const TARGET_TWO_PHASES: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n\n",
    "## 2. Phase two\n\n",
    "### 2.1. Step two\n\n",
    "- [ ] 2.1.1. Second task. Requires 2.1.1.\n",
);

const TARGET_TWO_TASKS: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n",
    "- [ ] 1.1.2. Second task. Depends on 1.1.1 and 1.1.2.\n",
);

const TARGET_THREE_PHASES: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n\n",
    "## 2. Phase two\n\n",
    "### 2.1. Step two\n\n",
    "- [ ] 2.1.1. Middle task.\n\n",
    "## 3. Phase three\n\n",
    "### 3.1. Step three\n\n",
    "- [ ] 3.1.1. Final task. Requires 3.1.1.\n",
);

const PHASE_FRAGMENT: &str = concat!(
    "## 9. Inserted phase\n\n",
    "### 9.1. Added step\n\n",
    "- [ ] 9.1.1. Added task. Requires 9.1.1.\n",
);

const TASK_FRAGMENT: &str = "- [ ] 9.9.9. Inserted task. Requires 9.9.9.\n";

const REPLACEMENT_FRAGMENT: &str = concat!(
    "## 7. Replacement phase A\n\n",
    "### 7.1. Step A\n\n",
    "- [ ] 7.1.1. Replacement task A.\n\n",
    "## 8. Replacement phase B\n\n",
    "### 8.1. Step B\n\n",
    "- [ ] 8.1.1. Replacement task B. Requires 8.1.1.\n",
);

#[derive(Debug)]
pub(crate) struct CliState {
    _tempdir: TempDir,
    dir: Dir,
    binary: Utf8PathBuf,
    target: Utf8PathBuf,
    fragment: Utf8PathBuf,
    stdout: String,
    stderr: String,
    success: bool,
}

impl CliState {
    fn write_target(&self, contents: &str) -> TestResult {
        self.dir.write("target.md", contents)?;
        Ok(())
    }

    fn write_fragment(&self, contents: &str) -> TestResult {
        self.dir.write("fragment.md", contents)?;
        Ok(())
    }

    fn read_target(&self) -> TestResult<String> { Ok(self.dir.read_to_string("target.md")?) }

    fn run<I, S>(&mut self, args: I) -> TestResult
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let output = Command::new(self.binary.as_str())
            .args(args.into_iter().map(|arg| arg.as_ref().to_owned()))
            .output()?;
        self.stdout = String::from_utf8(output.stdout)?;
        self.stderr = String::from_utf8(output.stderr)?;
        self.success = output.status.success();
        Ok(())
    }
}

#[fixture]
pub(crate) fn cli_state() -> CliFixture {
    let tempdir = tempfile::tempdir()?;
    let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf())
        .map_err(|path| format!("temporary directory is not valid UTF-8: {}", path.display()))?;
    let binary = Utf8PathBuf::from(env!("CARGO_BIN_EXE_mapsplice"));
    let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
    Ok(CliState {
        _tempdir: tempdir,
        dir,
        binary,
        target: root.join("target.md"),
        fragment: root.join("fragment.md"),
        stdout: String::new(),
        stderr: String::new(),
        success: false,
    })
}

fn state_mut(cli_state: &mut CliFixture) -> TestResult<&mut CliState> {
    match cli_state {
        Ok(state) => Ok(state),
        Err(error) => Err(error.to_string().into()),
    }
}

#[given("the target roadmap with two phases")]
fn target_two_phases(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_target(TARGET_TWO_PHASES)
}

#[given("the target roadmap with one step and two tasks")]
fn target_two_tasks(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_target(TARGET_TWO_TASKS)
}

#[given("the target roadmap with three phases")]
fn target_three_phases(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_target(TARGET_THREE_PHASES)
}

#[given("the phase fragment roadmap")]
fn phase_fragment(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_fragment(PHASE_FRAGMENT)
}

#[given("the task fragment roadmap")]
fn task_fragment(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_fragment(TASK_FRAGMENT)
}

#[given("the replacement fragment roadmap")]
fn replacement_fragment(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_fragment(REPLACEMENT_FRAGMENT)
}

#[when("I append the phase fragment")]
fn append_phase_fragment(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target.clone();
    let fragment = state.fragment.clone();
    state.run(["append", target.as_str(), fragment.as_str()])
}

#[when("I insert the phase fragment before phase 2")]
fn insert_before_phase(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target.clone();
    let fragment = state.fragment.clone();
    state.run(["insert", target.as_str(), "2", fragment.as_str()])
}

#[when("I insert the task fragment after task 1.1.1")]
fn insert_after_task(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target.clone();
    let fragment = state.fragment.clone();
    state.run([
        "insert",
        "--after",
        target.as_str(),
        "1.1.1",
        fragment.as_str(),
    ])
}

#[when("I delete phase 2")]
fn delete_phase_two(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target.clone();
    state.run(["delete", target.as_str(), "2"])
}

#[when("I replace phase 2 with the replacement fragment")]
fn replace_phase_two(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target.clone();
    let fragment = state.fragment.clone();
    state.run(["replace", target.as_str(), "2", fragment.as_str()])
}

#[when("I delete phase 1 in place")]
fn delete_in_place(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target.clone();
    state.run(["--in-place", "delete", target.as_str(), "1"])
}

#[when("I try to insert the mismatched fragment before phase 2")]
fn insert_mismatch(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target.clone();
    let fragment = state.fragment.clone();
    state.run(["insert", target.as_str(), "2", fragment.as_str()])
}

#[then("the command succeeds")]
fn command_succeeds(cli_state: &mut CliFixture) -> TestResult {
    assert!(state_mut(cli_state)?.success, "expected command to succeed");
    Ok(())
}

#[then("the command fails")]
fn command_fails(cli_state: &mut CliFixture) -> TestResult {
    assert!(!state_mut(cli_state)?.success, "expected command to fail");
    Ok(())
}

#[then("stdout contains the appended phase as phase 3")]
fn stdout_contains_phase_three(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert!(state.stdout.contains("## 3. Inserted phase"));
    Ok(())
}

#[then("the target file remains unchanged")]
fn target_unchanged(cli_state: &mut CliFixture) -> TestResult {
    assert_eq!(state_mut(cli_state)?.read_target()?, TARGET_TWO_PHASES);
    Ok(())
}

#[then("stdout renumbers phase two to phase 3 and rewrites its dependency")]
fn stdout_renumbers_phase_and_dependency(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert!(state.stdout.contains("## 3. Phase two"));
    assert!(state.stdout.contains("Requires 3.1.1."));
    Ok(())
}

#[then("stdout renumbers the old second task to 1.1.3")]
fn stdout_renumbers_second_task(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert!(state.stdout.contains("- [ ] 1.1.3. Second task."));
    Ok(())
}

#[then("stdout removes phase 2 and rewrites the remaining dependency to 2.1.1")]
fn stdout_rewrites_after_delete(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert!(state.stdout.contains("## 2. Phase three"));
    assert!(state.stdout.contains("Requires 2.1.1."));
    Ok(())
}

#[then("stdout contains replacement phases 2 and 3")]
fn stdout_contains_replacements(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert!(state.stdout.contains("## 2. Replacement phase A"));
    assert!(state.stdout.contains("## 3. Replacement phase B"));
    Ok(())
}

#[then("stdout is empty")]
fn stdout_is_empty(cli_state: &mut CliFixture) -> TestResult {
    assert!(state_mut(cli_state)?.stdout.trim().is_empty());
    Ok(())
}

#[then("the target file now starts with phase 1 titled Phase two")]
fn target_rewritten_in_place(cli_state: &mut CliFixture) -> TestResult {
    let rewritten = state_mut(cli_state)?.read_target()?;
    assert!(rewritten.contains("## 1. Phase two"));
    Ok(())
}

#[then("stderr mentions the phase versus task mismatch")]
fn stderr_mentions_mismatch(cli_state: &mut CliFixture) -> TestResult {
    let stderr = &state_mut(cli_state)?.stderr;
    assert!(stderr.contains("cannot use task content with phase anchor `2`"));
    Ok(())
}
