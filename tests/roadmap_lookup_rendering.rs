//! Focused regression tests for lookup, rendering, and metrics seams.

use camino::Utf8PathBuf;
use cap_std::{
    ambient_authority,
    fs_utf8::{Dir, PermissionsExt},
};
use mapsplice::{
    CliRequest,
    CommandKind,
    GlobalOptions,
    MapspliceError,
    metrics_snapshot,
    parse_anchor,
    run_from_args,
    run_request,
};
use rstest::rstest;
use tempfile::tempdir;

const TARGET_TWO_PHASES: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n\n",
    "## 2. Phase two\n\n",
    "### 2.1. Step two\n\n",
    "- [ ] 2.1.1. Second task. Requires 2.1.1.\n",
);

const PHASE_FRAGMENT: &str = concat!(
    "## 9. Inserted phase\n\n",
    "### 9.1. Added step\n\n",
    "- [ ] 9.1.1. Added task.\n",
);

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[rstest]
#[case::insert("insert")]
#[case::delete("delete")]
#[case::replace("replace")]
#[serial_test::serial(cli_env)]
fn phase_operations_report_missing_phase_anchor(#[case] command: &str) -> TestResult {
    let workspace = Workspace::create()?;
    workspace.write_target(TARGET_TWO_PHASES)?;
    workspace.write_fragment(PHASE_FRAGMENT)?;

    let result = match command {
        "insert" => run_from_args([
            "mapsplice",
            "insert",
            workspace.target.as_str(),
            "99",
            workspace.fragment.as_str(),
        ]),
        "delete" => run_from_args(["mapsplice", "delete", workspace.target.as_str(), "99"]),
        "replace" => run_from_args([
            "mapsplice",
            "replace",
            workspace.target.as_str(),
            "99",
            workspace.fragment.as_str(),
        ]),
        other => return Err(format!("unsupported command case `{other}`").into()),
    };
    let error = expect_error(result, "missing phase anchor should fail")?;

    let MapspliceError::AnchorNotFound { anchor } = error else {
        return Err(format!("expected AnchorNotFound, got {error:?}").into());
    };
    expect_equal(
        &anchor.to_string(),
        &"99".to_owned(),
        "missing anchor payload",
    )?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn metrics_record_equal_dependency_rewrites_for_output_modes() -> TestResult {
    let stdout_workspace = Workspace::create()?;
    stdout_workspace.write_target(TARGET_TWO_PHASES)?;
    let in_place_workspace = Workspace::create()?;
    in_place_workspace.write_target(TARGET_TWO_PHASES)?;
    let before = metrics_snapshot();

    run_from_args(["mapsplice", "delete", stdout_workspace.target.as_str(), "1"])?;
    let after_stdout = metrics_snapshot();
    run_from_args([
        "mapsplice",
        "--in-place",
        "delete",
        in_place_workspace.target.as_str(),
        "1",
    ])?;
    let after_in_place = metrics_snapshot();

    let stdout_dependency_rewrites = after_stdout
        .dependency_rewrites
        .saturating_sub(before.dependency_rewrites);
    let in_place_dependency_rewrites = after_in_place
        .dependency_rewrites
        .saturating_sub(after_stdout.dependency_rewrites);
    let stdout_in_place_rewrites = after_stdout
        .in_place_rewrites
        .saturating_sub(before.in_place_rewrites);
    let in_place_rewrites = after_in_place
        .in_place_rewrites
        .saturating_sub(after_stdout.in_place_rewrites);

    expect_equal(
        &stdout_dependency_rewrites,
        &1,
        "stdout dependency rewrites",
    )?;
    expect_equal(
        &in_place_dependency_rewrites,
        &stdout_dependency_rewrites,
        "in-place dependency rewrites",
    )?;
    expect_equal(&stdout_in_place_rewrites, &0, "stdout in-place rewrites")?;
    expect_equal(&in_place_rewrites, &1, "in-place rewrite counter")?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn failed_in_place_rewrite_does_not_record_dependency_rewrites() -> TestResult {
    let workspace = Workspace::create()?;
    workspace.write_target(TARGET_TWO_PHASES)?;
    let before = metrics_snapshot();
    workspace.make_parent_read_only()?;

    let request = delete_in_place_request(workspace.target.clone(), "1")?;
    let result = run_request(request);

    workspace.restore_parent_permissions()?;
    let error = expect_error(result, "read-only parent should reject in-place rewrite")?;
    if !matches!(error, MapspliceError::Io { .. }) {
        return Err(format!("expected IO error, got {error:?}").into());
    }
    let after = metrics_snapshot();
    expect_equal(
        &after.dependency_rewrites,
        &before.dependency_rewrites,
        "failed in-place dependency rewrites",
    )?;
    Ok(())
}

fn expect_error<T>(result: mapsplice::Result<T>, context: &str) -> TestResult<MapspliceError> {
    match result {
        Ok(_) => Err(context.to_owned().into()),
        Err(error) => Ok(error),
    }
}

fn expect_equal<T>(actual: &T, expected: &T, label: &str) -> TestResult
where
    T: std::fmt::Debug + PartialEq,
{
    if actual == expected {
        return Ok(());
    }
    Err(format!("{label}: expected {expected:?}, got {actual:?}").into())
}

fn delete_in_place_request(target: Utf8PathBuf, anchor: &str) -> TestResult<CliRequest> {
    Ok(CliRequest {
        global: GlobalOptions { in_place: true },
        target,
        command: CommandKind::Delete {
            anchor: parse_anchor(anchor)?,
        },
    })
}

struct Workspace {
    dir: Dir,
    target: Utf8PathBuf,
    fragment: Utf8PathBuf,
    _tempdir: tempfile::TempDir,
}

impl Workspace {
    fn create() -> TestResult<Self> {
        let tempdir = tempdir()?;
        let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf()).map_err(|path| {
            format!("temporary directory is not valid UTF-8: {}", path.display())
        })?;
        let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
        Ok(Self {
            dir,
            target: root.join("target.md"),
            fragment: root.join("fragment.md"),
            _tempdir: tempdir,
        })
    }

    fn write_target(&self, contents: &str) -> TestResult {
        self.dir.write("target.md", contents)?;
        Ok(())
    }

    fn write_fragment(&self, contents: &str) -> TestResult {
        self.dir.write("fragment.md", contents)?;
        Ok(())
    }

    fn make_parent_read_only(&self) -> TestResult {
        let mut permissions = self.dir.metadata(".")?.permissions();
        permissions.set_mode(0o555);
        self.dir.set_permissions(".", permissions)?;
        Ok(())
    }

    fn restore_parent_permissions(&self) -> TestResult {
        let mut permissions = self.dir.metadata(".")?.permissions();
        permissions.set_mode(0o755);
        self.dir.set_permissions(".", permissions)?;
        Ok(())
    }
}
