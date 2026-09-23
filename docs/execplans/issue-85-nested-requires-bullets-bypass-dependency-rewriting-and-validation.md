# ExecPlan: Nested `Requires` bullets bypass dependency rewriting and validation (issue #85)

## Purpose

Issue #85: a `Requires` clause written as a nested task-body bullet was
silently ignored during renumbering and dangling-dependency validation.
Insertion redirected a consumer to an unrelated new task; deleting a
prerequisite produced a self-dependency; in-place deletion wrote that invalid
result to disk.

All four tasks of the coding plan are complete and committed, and every
deterministic gate is green. This plan tracks the CodeRabbit review and the PR.

## Progress so far

| Task | Scope                                                | State |
| ---- | ---------------------------------------------------- | ----- |
| 1    | Recognizer fix across all four clause positions      | Done  |
| 2    | Document the supported clause grammar and exclusions | Done  |
| 3    | CLI regressions, golden fixtures, property suite     | Done  |
| 4    | Verus proofs, ledger, Makefile and CI wiring         | Done  |

_Table 1: The coding plan's four tasks and their state._

Commits, oldest first. This list is deliberately exhaustive: the round-tally
error below was hidden for a while by a list that was missing entries, and a
reconciliation against a list that is itself incomplete cannot find anything.

The four coding-plan tasks:

- `db94b11` — Rewrite dependency clauses in nested task-body bullets.
- `5ba0ab2` — Document the supported Requires clause grammar.
- `962d5eb` — Add CLI and golden regressions for nested Requires clauses (#85).
- `0dd99de` — Add property suite and split oversized test modules (#85).
- `7b9032b` — Extract nested-Requires regressions to honour the 400-line file
  rule (#85).
- `b7fdcda` — Scope dependency property assertions to each item's own block.
- `55e3484` — Extract dependency resolution into a production-used Verus kernel.
- `9a9b7c8` — Wire the Verus harness into the build and continuous integration.
- `9202dbf` — Share the verified kernel body as a macro, not an include splice.
- `4daffb0` — Record the ICE resolution and the macro trade in the ExecPlan.

The six CodeRabbit rounds and their response commits:

| Round | Findings | Answered by | Evidence artefact                                      |
| ----- | -------- | ----------- | ------------------------------------------------------ |
| 1     | 8        | `05b49d3`   | `/tmp/coderabbit-mapsplice-issue-85.out`               |
| 2     | 6        | `67ab4b9`   | `.../tasks/bh6xlkhs1.output`                           |
| 3     | 7        | `16ef675`   | `.../tasks/bfs030ibl.output`                           |
| 4     | 7        | `fcbe9c6`   | `/tmp/coderabbit-issue-85-...out`                      |
| 5     | 4        | `92dc9bb`   | `/tmp/coderabbit-mapsplice-issue-85-...out.raw`        |
| 6     | 6        | `08b62f9`   | `/tmp/coderabbit-mapsplice-issue85-round6-1bd792a.out` |

_Table 2: the review rounds, re-derived from the artefacts by matching each
round's findings against the files its response commit touched._

Supporting commits that are not themselves a round response:

- `de24f4c` — Use `is_multiple_of` where the crate denies integer remainder.
- `87edd94` — Record the fourth review round and its lessons in the ExecPlan.
- `0c084a8` — Record the green CI run in the ExecPlan.
- `1477434` — Correct the review-round count in the ExecPlan.
- `e277c1f` — Fix the spelling and formatting the gates caught in the ExecPlan.
- `67becfe` — Give the two masked oracle defects their own survival counts.
- `72ff94f` — Correct the evidence-glob lesson to what was measured.
- `0c222e4` — Re-wrap the corrected lesson paragraphs to the 80-column limit.
- `e083639` — Answer the sixth CodeRabbit review of the issue #85 work
  (amended to `08b62f9`, which is the commit the round is answered by; the
  original said "fourth", continuing an off-by-one the round table corrects).
- `97da0e4` — Fix the formatting and lint the gates caught in the round-6
  ExecPlan edit.
- `a720876` — Record the two formatter lessons from the round-6 gate fix.

## Task 4 outcome: the splice is a macro, and why

The construction described below was correct about Verus and wrong about
Whitaker. A bare `include!` inside a function body _is_ verified — that part
held — but it also aborts Whitaker's `bumpy_road_function` lint with an
internal compiler error, which fails `make lint` for the whole crate.

The mechanism, established in a twenty-line reproduction crate rather than by
reading:

- Tokens spliced by `include!` report `span.from_expansion() == false` while
  still carrying the _included_ file's line numbers. `bumpy_road_function`
  skips expansion spans precisely because such line numbers can point outside
  the enclosing function, but that guard cannot see through `include!`. It
  therefore compares the body file's lines against the function's range in a
  different coordinate system and panics in `push_segment`
  (`crates/bumpy_road_function/src/driver/segment_builder.rs`).
- The trigger is _any_ `include!` inside a function body. An `include!` at item
  level is fine; a nested `include!` inside an included item is not. The lint
  has no `excluded_paths` support (unlike `no_std_fs_operations`), and every
  `allow`/`expect` form is rejected earlier by the attribute lints this
  repository already denies, so attribute suppression cannot reach it either.
- Expansion context is the discriminator the lint dispatches on. Sharing the
  body as a `macro_rules!` definition gives the tokens the expansion context
  the lint already skips, while the proof and the product still expand one text.

The shared text therefore moved from `verus/kernels/select_resolution.body.rs`
(a body fragment) to `verus/kernels/select_resolution.macro.rs` (a macro
definition), and both `verus/lib.rs` and `src/roadmap/ops/remap_kernel.rs`
expand `select_resolution_body!` and include the macro file at item level.

Confirmed after the change: `make verus` reports `4 verified, 0 errors`;
`make lint` exits 0 with zero ICEs and the Whitaker toolchain banner present
(so the suite genuinely loaded); and all six deterministic gates pass. The fix
was verified both cold (a run that recompiled the crate) and warm.

The trade this makes is recorded in the macro file and in
`docs/verification.md`: the kernel body is now opaque to `bumpy_road_function`
in both crates. Nothing is lost for a body holding one `if` inside one match
arm, but a future kernel with genuinely nested conditionals would go unflagged
by that lint.

## Task 4 design decisions (evidence-based)

The reference implementation is `leynos/mdtablefix` (ADR 0011 and
`docs/verification.md`). Its scaffolding — `tools/verus/VERSION`,
`tools/verus/SHA256SUMS`, `verus/smoke.rs`, the workflow shape — is already
present in this repository and byte-matches the reference. Its _proof_ content
is not transferable, because ADR 0011's "include the production module with
`#[path]`" convention does not actually verify anything. This was established
by direct experiment against the pinned Verus binary, not by reading:

| Probe                                                              | Result                                                                                                                                         |
| ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Plain-Rust module `#[path]`-included, function called from a proof | `error: cannot use function ... which is ignored because it is either declared outside the verus! macro or it is marked as external`           |
| Same, with `--no-external-by-default`                              | Still unverifiable; a false assertion about the body is not caught                                                                             |
| Module carrying its own `verus!` block, `#[path]`-included         | Verified as a _separate crate item_; the module's own body is proved, but this forces `verus!` syntax into a production file                   |
| `include!` at item level inside `verus!`                           | Function is again "ignored" — the macro does not see it                                                                                        |
| **`include!` inside a function body inside `verus!`**              | **The included text is verified.** A false postcondition over the included body is caught, with the diagnostic pointing into the included file |

_Table 3: Verus splice probes and what each establishes._

So Verus verifies only text that is literally inside the `verus!` macro. A body
spliced by `include!` _is_ verified, and that same file is also plain Rust that
cargo compiles. That is the construction used here: one body file, spliced into
the production `const fn` by cargo and into the verified `fn` by Verus.

Constraints confirmed by experiment:

- `vstd` and `builtin_macros` are **not** on crates.io, so production files
  cannot carry `verus!` syntax without breaking the ordinary build.
- Generic kernels with `Option<T>` verify, provided `T: Copy`.
- A non-private `spec fn` must be `pub open spec fn` (or `pub closed`).
- `#[must_use]` plus clippy's `missing_const_for_fn` (deny) means the
  production kernel must be `const fn`; Verus accepts the `const` form.

## Evidence recorded

- Pinned Verus binary: `.verus/0.2025.04.19.1b16620/verus/verus` (gitignored).
- Self-test marker for this binary:
  `verification results:: 0 verified, 1 errors`, exit status 1.
- `rust-prover-tools` is reachable at
  `https://github.com/leynos/rust-prover-tools`; the reference pins commit
  `2ba17da6b7e24160c4041e8d41ba1ba6cca87b35`.

## Completed work

1. Extracted the resolution decision into `src/roadmap/ops/remap_kernel.rs`,
   with its body shared via `verus/kernels/select_resolution.macro.rs` and
   called from `RenumberPlan::resolve_reference`.
2. Added `verus/lib.rs` as the proof entry point: identity preservation,
   local-mapping precedence, and non-vacuity — three theorems, all discharged,
   measured as `4 verified, 0 errors` (the three `proof fn`s plus the verified
   kernel `fn`).

   Two further obligations were drafted and then removed, and neither removal
   was discretionary. "Source-span preservation" concluded equality of results
   from pairwise-equal arguments, which Verus discharges from the signature
   alone, so no kernel defect could falsify it; a CodeRabbit review found that
   independently. "Deleted-target rejection" looked falsifiable but was not:
   its theorem assumed both options absent and concluded `None`, and with
   neither option holding a `T` there is no value to fabricate, so the
   conclusion is forced by the type rather than by the kernel. A six-defect
   battery established it — the theorem survived all six specification defects
   while each of its three neighbours rejected at least one. Table 2 of
   `docs/verification.md` records the defect that each surviving theorem
   rejects; deleted-target rejection is covered instead by the identity
   obligation in kernel form and by the `unresolved` collection in production.

   One body defect was injected as well, into the shared macro body alone with
   the specification left correct. Verus rejected it at the `select_resolution`
   `ensures` clause (4 verified, 1 error), which is the evidence that the proof
   is about the text the product compiles rather than about the specification.
3. Added the `docs/verification.md` ledger with a claim row per theorem, and
   `scripts/check-verification-ledger.sh` wired into `make lint`.
4. Wired `make verus` and `make verus-selftest`; the selftest runs a
   deliberately false proof and fails unless Verus rejects it.
5. Added `tests/verus_harness.rs` (this also discharged the outstanding
   CodeRabbit finding about `verus/smoke.rs` referencing targets that did not
   exist).
6. Added `.github/workflows/verus.yml`.

## Remaining work

1. Push and open the draft PR. **Done** — the branch is pushed and
   [PR #86](https://github.com/leynos/mapsplice/pull/86) is open as a draft.
2. Run `coderabbit review --agent` and clear all concerns. **Done for six
   rounds** — 8, 6, 7, 7, 4 and 6 findings, every one actioned or dismissed
   with recorded evidence, counts re-derived from the artefacts (Table 2).
   Round 6 returned six findings that are four distinct issues, because two
   pairs are the same finding stated twice. Two were accepted and fixed (the
   ExecPlan's table captions were out of document order; the `verus.yml`
   concurrency comment described a `push` trigger the workflow does not have).
   Two were dismissed against evidence already on file: the ExecPlan rename
   re-raises round 1's finding with a new justification, and the ledger-fixture
   finding would undo what round 3's major finding asked for. A seventh round
   has not been run.
3. Follow the CI result for the PR. **Done** — the first run failed `make lint`
   (see the lesson below), and a later one failed `make spelling` on a commit
   hash written into the ExecPlan. Both are fixed. The current tip is
   `5e38bcb`, where run `35871439602` reports `build-test success` and
   `35871439535` reports `verify success`, with the `Spelling` step reaching
   `refreshed: typos.toml` and no error.
4. The code gates for the branch as a whole. **Done locally at `08b62f9`.**
   The full set was run there — `check-fmt`, `lint`, `typecheck`, `test`,
   `markdownlint`, `nixie` and `spelling` — and five passed, with `check-fmt`
   and `markdownlint` failing on the ExecPlan text this branch had just added.
   Both were fixed, and all four Markdown gates were re-run green at `a720876`.
   `lint`, `typecheck` and `test` were green at `08b62f9` and nothing after
   that touched a Rust file or the `Makefile`, so they were not re-run; the
   committed `verus.yml` edit _is_ covered by them, because
   `tests/verus_harness/workflow.rs` reads that file through `include_str!` and
   again from disk. Logs are under `/tmp/<gate>-mapsplice-issue85-08b62f9.out`
   and `...-97da0e4.out`.
5. Local Markdown gates. **Done twice** — at `5e38bcb` (203 files left
   unchanged; `Linting: 60 file(s)`, `Summary: 0 error(s)`) and again at
   `a720876` after the round-6 text was added and formatted. Both runs were
   sequential, with HEAD unmoved and `typos.toml` reported in sync (`current:`)
   rather than drifting. This tracked 42 of the 61 paths the branch changes
   relative to `main`, which is why item 4 was widened rather than inherited
   from here.

## Lessons

- **A local gate run can be vacuous about a later step in the same target.** An
  early `make lint` failure (a denied `%`) aborted the target two steps before
  Whitaker, so the run's silence about the Dylint suite proved nothing. A gate
  is only green when it reaches its last step; the log was checked for the
  Dylint banner rather than for a zero exit alone.
- **A new prerequisite for `make lint` is a new CI prerequisite.** Wiring
  `check-verification-ledger` into `lint` made `rg` a hard dependency of the
  lint job. It is present locally, so the gate passed here and failed on the
  runner. The install step now provides it.
- **The formatter has to be run with the gate's own flags, or it is a different
  tool.** `make check-fmt` runs `mdtablefix --check` over its selected files
  _with the flags held in `MDTABLEFIX_RULES`_. Fixing a table-alignment
  complaint with a bare `mdtablefix --in-place <file>`, without those flags,
  produced a file that looked repaired and failed the very next `check-fmt` —
  the alignment was fixed but the paragraph wrapping the default rules leave
  alone was not. The second attempt passed the rule set and succeeded. The
  flags are declared once in the Makefile, so they are visible rather than
  hidden; the mistake was to reach for the bare command instead of reading how
  the gate calls it. Run the gate's command, not the tool.
- **Two failures in one file can have one cause, and fixing the second can
  reintroduce work in the first.** Seven `markdownlint` errors and a
  `check-fmt` complaint all landed in this file at once. Converting the three
  `*emphasis*` spans to `_emphasis_` shortened those lines, which re-flowed a
  paragraph `--wrap` had already settled, so `check-fmt` failed again after the
  MD049 fix. Format and lint are not independent when the formatter re-packs
  lines to a column limit: run the formatter _last_, after every content edit,
  and re-run it if any edit lands afterwards.
- **A fixture can be a test of nothing.** The C2 golden case originally deleted
  the _last_ task, which shifts no later number and so rewrote no clause: it
  was a test that a trailing delete disturbs nothing, which is true but not
  what the case name claimed. It now deletes a middle task, so renumbering
  actually happens while the incidental prose is held under test.
- **Not every clause position of a theorem is falsifiable.** Two obligations
  were removed after a defect battery showed no defect could falsify them. The
  ledger records the battery; the proofs state three obligations, not five.
- **An unreachable bug is still a bug, and reachability must be checked rather
  than assumed.** Two oracle defects survived every review round that ran after
  they were written — five rounds for the identity helper that divided by the
  wrong constant twice, four for the summary match that accepted a prefix —
  because every input the generator produces happens to mask them. Neither
  could fail on today's inputs. Both were fixed, because the cost of the fix is
  a few lines and the cost of the failure is a test suite that passes more
  easily while asserting less. The distinction between "live" and "reachable"
  was established by enumerating the generated inputs, not by reading the code
  and judging it unlikely.
- **A review finding's own framing can be wrong.** CodeRabbit described the
  identity-helper change as preserving existing behaviour. It does not: the two
  readings differ for any step beyond the first few. The finding was right that
  something should change and wrong about why, so the reasoning was checked
  against the code before the edit, and the record says which part was taken.
- **A count in a plan is a claim, and this one was wrong twice over.** The round
  tally stood at "four rounds (7, 7, 7, 4 findings)" and was carried from the
  plan into the pull-request body without being re-derived. Both halves were
  false. There were five rounds, and the per-round counts are 8, 6, 7, 7 and 4,
  for 32 findings in total. The two errors nearly cancelled: the wrong
  breakdown summed to 25, which is what the correct count was wrongly recorded
  as, so a re-read of the total confirmed a number that was itself groundless.
  (A sixth round has since run, so the live tally is 8, 6, 7, 7, 4, 6 for 38;
  Table 2 is the current record. This paragraph is left as written because it
  is the record of what the correction established at the time, and the lesson
  it draws would be weakened by editing the numbers that make the point. The
  paragraph has already gone stale once for exactly the reason it describes,
  which is the cleanest illustration of it available.)
- **A missing round announces itself as a count that does not close.** Five
  commits name CodeRabbit; only four review artefacts could be found. That
  one-line reconciliation was available from the start and would have caught
  the omission immediately, where re-reading the tally did not. Prefer a check
  that compares two independently-derived numbers over a re-read of the number
  in question.
- **The evidence glob was narrower than the evidence.** Round artefacts were
  enumerated with `ls /tmp/*coderabbit*issue-85*`, which found four of the
  five. The fifth was captured only inside a subagent task-output file under
  `/tmp/claude-1000/.../tasks/`, which any `/tmp` glob rooted at the top level
  cannot see. The four that did match were then indexed against the wrong
  response commits — the file named `.out.round3` actually holds the round
  answered by `67ab4b9`, which the plan's own commit list did not mention.
  Enumerate by content, not by a filename pattern that only holds when the
  convention was followed. Measured, rather than assumed: the narrower pattern
  `/tmp/coderabbit-*.out` reaches only two of the five; the one actually used
  reaches four; a content sweep for the branch name **and** a review marker
  (`Review completed` or `"severity"`) reaches all five, as six file hits,
  because round 2 was captured twice. Two cautions, both measured against this
  machine rather than reasoned about: the branch name alone is worthless as a
  key — it matches 49 files here, including other sessions' gate logs and every
  `tee` output on the branch — and the doubled hit means hits must be
  deduplicated by comparing finding lists before they are counted as rounds.
  The authoritative re-derivation matched each artefact's findings against the
  files each response commit touched, rather than trusting timestamps or
  filenames.
- **Two unrelated things shared the label "round 3", and correcting the first
  error caused a second.** The note file `cr-triage-round3.md` and the artefact
  file `.out.round3` are different rounds. The first correction asserted that
  the note documented `67ab4b9`'s round on the strength of the filename match
  alone; the file-touch check showed the note's F1–F7 table corresponds to
  `fcbe9c6` and the note is round 4, not round 2. A correction written from the
  same faulty signal as the error it fixes is not a correction.
- **A scope decision that is right for the diff can still understate the
  branch.** Gating the seven commits after `92dc9bb` as docs-only is correct —
  they touch no code — but "these gates pass" then means something narrower
  than it sounds, because the branch also changes 42 non-Markdown paths whose
  last local gate run was earlier. The green result was read next to a CI run
  covering the full set at the same commit, which is what made the narrow scope
  sound. State which gates ran and which were replaced by other evidence, as
  the Remaining work section now does, rather than letting a clean local result
  imply coverage it did not have.
- **Writing a commit hash into prose can fail the spelling gate.** `5ba0ab2`
  is a valid revision, but Typos splits the hex string into letter-runs and
  reads the middle two characters of that run as a typo — a misspelling of a
  short English word. The failure surfaced twice: locally in `make spelling`,
  and in CI at 13:53 (run `35869927771`, the `Spelling` step), both naming the
  same line. Two things are worth keeping. The exemption went into
  `typos.local.toml` — the tracked input — rather than `typos.toml`, which is
  generated and says so; the generated file is committed alongside it because
  the builder reports the drift otherwise. And the pattern was scoped to the
  backticked 7–40 hex form rather than to that one revision, with a negative
  control run in a scratch repository to show the wider scope is still safe: a
  common sending-verb misspelling and the US spelling of `colour` both kept
  failing while the SHA was ignored. Of the 33 backticked hex tokens in the
  tracked docs, all are commit or Verus hashes and none is an English word, so
  the form is the right thing to exclude. The lesson's own first draft wrote
  the misspelling out as an example and was itself rejected by the gate — which
  is the gate behaving correctly, and a small demonstration that quoting an
  error verbatim is not free.
- **A grep over a freshly-written file can compare it with itself.** Checking
  that the pull-request body matched the local draft, the first attempt wrote
  the live body to a file inside a pipeline and then diffed that same file
  against the draft. It printed `IDENTICAL` while the byte counts in the very
  same output differed by 26. The bytes disagreed with the verdict, and the
  bytes were right: the extraction had added a trailing newline. When two
  measurements of the same thing disagree, that is the finding — re-measure by
  an independent route (here, hashing both sides) rather than trusting the one
  that agrees with what was expected.
- **A re-review can re-raise a settled finding, and a _checkable_ new reason
  deserves a check rather than a repeat of the old argument.** Round 6 asked
  again for the ExecPlan to be renamed to `docs/execplans/roadmap-1-2-1.md`,
  which round 1 had already raised and which the round-1 triage had already
  refused. The difference was the justification: round 6 said the rename was
  needed "so the renumber planner can classify and carry the plan correctly".
  That is a factual claim about the repository and it is false in two
  independent ways, both one command from being shown so —
  `git grep -n "execplans" -- src/` finds no such subcommand, and
  `git grep -n "execplans/issue-85" -- .` finds no reference to the file. There
  is no planner, no classifier, and no link. Re-arguing the original refusal
  would have been the wrong move twice over: it would have treated a new claim
  as though it were the old one, and it would have left the claim standing
  unrefuted. When a re-raise arrives with a reason that can be tested, test it.
  When the reason turns out to be false, that is the strongest possible form of
  the dismissal — and the weakest possible reason to change the code.
- **Two of round 6's six findings were the same finding stated twice, so the
  round's own count is not its issue count.** The ExecPlan rename came back
  once as "rename to the conventional roadmap name for item 1.2.1" and once as
  "rename the file to `docs/execplans/roadmap-1-2-1.md`"; the ledger-fixture
  finding came back twice differing only in a subordinate clause. Six findings,
  four issues. Deduplicate by the change requested, not by the number the
  report prints — the count in a summary line is a measure of how many times
  the scanner spoke, not of how many defects exist.
- **A comment can be the defect, and the honest fix can be smaller than the
  one requested.** Round 6 noticed that `verus.yml`'s concurrency comment
  mentions "a push to `main`" and asked for a push trigger to be added so the
  comment would be true. Measured across the repository,
  `grep -rn "push:" .github/workflows/` returns nothing: no workflow here
  triggers on push, including `ci.yml`. Adding one to `verus.yml` on its own
  would have made proofs run on every merge to `main` while lint and the test
  suite did not — trading a comment's inaccuracy for a CI layout that is harder
  to reason about and still inaccurate about every other workflow. The comment
  was the defect; the triggers were correct as they stood. Where a finding
  identifies two things that disagree and only one of them is wrong, the work
  is to determine _which_, not to move the one that is easier to move.
- **Three of round 6's findings pointed at two files spelled with the same
  table numbers, and only two of the three could be right.** The ExecPlan had
  its captions out of document order (`Table 3` above `Table 2`), while line
  171's "Table 2 of `docs/verification.md`" is a cross-_document_ reference
  into the ledger's own `Table 2` and was correct. A search-and-replace over
  "Table 2"/"Table 3" would have swapped all of them and introduced exactly the
  defect the round was reporting. The review had scoped its own suggestion to
  the local captions and the local cross-reference, and that scoping was
  correct — worth noting because a plausible-looking mechanical fix here was
  the one that would have broken something.

## Constraints that must hold

- No `assume`, admitted lemma, or `external_body` that merely restates the
  invariant under proof.
- Frame the risk honestly: the _clause recognizer_ is the large surface here,
  and it is not proved. `docs/verification.md` states that boundary rather than
  implying the recognizer is verified.
- The macro wrapper means the kernel body is opaque to `bumpy_road_function`.
  This is a real, if narrow, loss of lint coverage, and it is recorded in both
  the macro file and the ledger rather than left implicit.
- Byte offsets and Unicode scalar indices are distinct types in any spec that
  touches them.
