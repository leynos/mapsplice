//! Contract tests for `scripts/check-verification-ledger.sh`.
//!
//! The ledger is what makes the proof claims auditable, so the check that
//! keeps it honest needs its own evidence. Each case materialises a small tree
//! — a ledger and a source file — and asserts the checker's exit status and,
//! where it fails, its single-line diagnostic.

use cap_std::fs_utf8::Dir;
use rstest::{fixture, rstest};
use tempfile::TempDir;

use crate::prover_harness::{TestResult, fixture_text, open_dir, run_ledger_check, utf8};

/// A `(ledger, source)` pair, as text.
type LedgerFixture = (String, String);

const LEDGER_DIAGNOSTIC: &str = "verification ledger names missing symbol:";
const EMPTY_LEDGER_DIAGNOSTIC: &str = "verification ledger states no verifiable claims";

/// Render the real ledger with its kernel column replaced by `symbol`.
fn ledger_naming(symbol: &str) -> TestResult<String> {
    let ledger = fixture_text("real", "md")?;
    Ok(ledger.replace("| `select_resolution` |", &format!("| `{symbol}` |")))
}

/// Build a fixture from a symbol name and the source file that should satisfy it.
fn fixture(symbol: &str, source: &str) -> TestResult<(String, String)> {
    Ok((ledger_naming(symbol)?, source.to_owned()))
}

/// Materialise a `(ledger, source)` pair into a scratch tree.
///
/// # Errors
///
/// Returns an error when the scratch tree cannot be created or populated.
pub fn materialise(fixture: &LedgerFixture) -> TestResult<TempDir> {
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

#[fixture]
fn real_ledger() -> TestResult<LedgerFixture> {
    fixture_text("real", "md").map(|md| (md, String::new()))
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
    let directory = materialise(&(ledger, source.to_owned()))?;
    let output = run_ledger_check(utf8(directory.path())?);

    assert_eq!(
        output.status.success(),
        should_accept,
        "source {source:?} should {} the claim",
        if should_accept { "satisfy" } else { "fail" }
    );
    if !should_accept {
        let diagnostic = String::from_utf8_lossy(&output.stdout);
        assert!(
            diagnostic.starts_with(LEDGER_DIAGNOSTIC),
            "expected a missing-symbol diagnostic, got {diagnostic:?}"
        );
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
    let directory = materialise(&fixture(symbol, source)?)?;
    let output = run_ledger_check(utf8(directory.path())?);

    assert!(
        !output.status.success(),
        "claim {symbol:?} should be rejected"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim_end(),
        format!("{LEDGER_DIAGNOSTIC} {symbol}")
    );
    Ok(())
}

#[test]
fn ledger_check_rejects_a_ledger_stating_no_claims() -> TestResult {
    let directory = materialise(&(
        "# Verification ledger\n\n| Claim | Executable function |\n| ----- | --- |\n".to_owned(),
        "fn select_resolution() {}".to_owned(),
    ))?;
    let output = run_ledger_check(utf8(directory.path())?);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(EMPTY_LEDGER_DIAGNOSTIC),
        "an empty claim table must be a failure, not a vacuous pass"
    );
    Ok(())
}
