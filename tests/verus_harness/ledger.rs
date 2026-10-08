//! Contract tests for `scripts/check-verification-ledger.sh`.
//!
//! The ledger is what makes the proof claims auditable, so the check that
//! keeps it honest needs its own evidence. Each case materializes a small tree
//! — a ledger and a source file — and asserts the checker's exit status and,
//! where it fails, its single-line diagnostic.
//!
//! Failures are reported as `Err` rather than by panicking, because
//! `clippy::panic_in_result_fn` is denied across the workspace and the
//! diagnostics a test observes are also what a reader needs to see.

use cap_std::fs_utf8::Dir;
use rstest::{fixture, rstest};
use tempfile::TempDir;

use crate::prover_harness::{
    TestResult,
    fixture_text,
    open_dir,
    read_repo_file,
    run_ledger_check,
    utf8,
};

/// A `(ledger, source)` pair, as text.
type LedgerFixture = (String, String);

const LEDGER_DIAGNOSTIC: &str = "verification ledger names missing symbol:";
/// The ledger's executable-function column, as the fixture spells it.
const KERNEL_COLUMN: &str = "| `select_resolution` |";
const EMPTY_LEDGER_DIAGNOSTIC: &str = "verification ledger states no verifiable claims";

/// Render the real ledger with its kernel column replaced by `symbol`.
///
/// # Errors
///
/// Returns an error when the fixture no longer carries the kernel column, so a
/// rejection case cannot pass because the substitution silently did nothing.
fn ledger_naming(symbol: &str) -> TestResult<String> {
    let ledger = fixture_text("real", "md")?;
    if !ledger.contains(KERNEL_COLUMN) {
        return Err("the ledger fixture no longer contains the kernel column".into());
    }
    // A substitution that is the identity — `symbol` naming the kernel itself —
    // is the case that puts the claim and the declaration in agreement, so it is
    // expressed here rather than special-cased by the caller.
    Ok(ledger.replace(KERNEL_COLUMN, &format!("| `{symbol}` |")))
}

/// Build a fixture from a symbol name and the source file that should satisfy it.
fn fixture(symbol: &str, source: &str) -> TestResult<LedgerFixture> {
    Ok((ledger_naming(symbol)?, source.to_owned()))
}

/// Materialize a `(ledger, source)` pair into a scratch tree.
///
/// # Errors
///
/// Returns an error when the scratch tree cannot be created or populated.
fn materialize(fixture: &LedgerFixture) -> TestResult<TempDir> {
    let directory = TempDir::new().map_err(|error| format!("create scratch: {error}"))?;
    let root = utf8(directory.path())?;
    let handle = open_dir(root)?;
    handle
        .create_dir("docs")
        .map_err(|error| format!("create docs: {error}"))?;
    handle
        .create_dir("src")
        .map_err(|error| format!("create src: {error}"))?;
    write(&handle, "docs/verification.md", &fixture.0)?;
    write(&handle, "src/kernel.rs", &fixture.1)?;
    Ok(directory)
}

fn write(handle: &Dir, path: &str, contents: &str) -> TestResult {
    handle
        .write(path, contents)
        .map_err(|error| format!("write {path}: {error}"))?;
    Ok(())
}

/// Return an error when `condition` does not hold.
fn require(condition: bool, reason: String) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(reason.into())
    }
}

#[fixture]
fn real_ledger() -> TestResult<LedgerFixture> {
    fixture_text("real", "md").map(|md| (md, String::new()))
}

#[test]
fn the_fixture_is_a_copy_of_the_real_ledger() -> TestResult {
    // Every case above asserts against the fixture rather than `docs/`, so a
    // fixture that drifts from the document it copies would let the checker be
    // tested against a ledger no longer resembling the one that ships. The
    // fixture is named `real` for that reason, and this is what the name means.
    let fixture = fixture_text("real", "md")?;
    let on_disk = read_repo_file("docs/verification.md")?;
    require(
        fixture == on_disk,
        "tests/data/verification_ledger/real.md must be a copy of docs/verification.md; re-copy \
         it after editing the ledger"
            .to_owned(),
    )
}

#[rstest]
#[case::documented_but_undeclared("// fn select_resolution() {}", false)]
#[case::declared_at_column_zero("fn select_resolution() {}", true)]
#[case::declared_with_visibility("pub(crate) fn select_resolution() {}", true)]
#[case::declared_with_generics("pub const fn select_resolution<T: Copy>(a: u8) -> u8 { a }", true)]
#[case::declared_with_indentation("    fn select_resolution() {}", true)]
#[case::declared_with_async("pub async fn select_resolution() {}", true)]
fn ledger_check_accepts_only_a_real_declaration(
    #[case] source: &str,
    #[case] should_accept: bool,
    #[from(real_ledger)] real_ledger: TestResult<LedgerFixture>,
) -> TestResult {
    let (ledger, _) = real_ledger?;
    let directory = materialize(&(ledger, source.to_owned()))?;
    let output = run_ledger_check(utf8(directory.path())?)?;

    require(
        output.status.success() == should_accept,
        format!(
            "source {source:?} should {} the claim, but the checker exited {}. Output: {}",
            if should_accept { "satisfy" } else { "fail" },
            output.status,
            String::from_utf8_lossy(&output.stdout).trim_end(),
        ),
    )?;
    if !should_accept {
        let diagnostic = String::from_utf8_lossy(&output.stdout);
        require(
            diagnostic.starts_with(LEDGER_DIAGNOSTIC),
            format!("expected a missing-symbol diagnostic, got {diagnostic:?}"),
        )?;
    }
    Ok(())
}

#[rstest]
#[case::prefix_only("select_res", "fn select_resolution_spec() {}")]
#[case::absent("absent_kernel", "fn something_else() {}")]
#[case::declared_only_as_a_constant("select_resolution", "const SELECT_RESOLUTION: u8 = 0;")]
fn ledger_check_rejects_a_claim_with_no_declaration(
    #[case] symbol: &str,
    #[case] source: &str,
) -> TestResult {
    let directory = materialize(&fixture(symbol, source)?)?;
    let output = run_ledger_check(utf8(directory.path())?)?;

    require(
        !output.status.success(),
        format!("claim {symbol:?} should be rejected"),
    )?;
    let diagnostic = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_owned();
    require(
        diagnostic == format!("{LEDGER_DIAGNOSTIC} {symbol}"),
        format!("expected the missing-symbol diagnostic, got {diagnostic:?}"),
    )
}

#[test]
fn ledger_check_rejects_a_ledger_stating_no_claims() -> TestResult {
    let directory = materialize(&(
        "# Verification ledger\n\n| Claim | Executable function |\n| ----- | --- |\n".to_owned(),
        "fn select_resolution() {}".to_owned(),
    ))?;
    let output = run_ledger_check(utf8(directory.path())?)?;

    require(
        !output.status.success(),
        "an empty claim table must fail rather than pass vacuously".to_owned(),
    )?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    require(
        stderr.contains(EMPTY_LEDGER_DIAGNOSTIC),
        format!("expected the empty-ledger diagnostic, got {stderr:?}"),
    )
}
