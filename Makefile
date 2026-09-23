.PHONY: help all clean test build release lint fmt check-fmt markdownfmt \
	markdownlint markdownlint-paths nixie typecheck test-workflow-contracts \
	spelling check-ripgrep check-verification-ledger check-prover-tools \
	verus-install verus verus-selftest


TARGET ?= mapsplice

CARGO ?= cargo
WHITAKER ?= whitaker
BUILD_JOBS ?=
RUST_FLAGS ?=
RUST_FLAGS := -D warnings $(RUST_FLAGS)
RUSTDOC_FLAGS ?=
RUSTDOC_FLAGS := -D warnings $(RUSTDOC_FLAGS)
CARGO_FLAGS ?= --workspace --all-targets --all-features
CLIPPY_FLAGS ?= $(CARGO_FLAGS) -- $(RUST_FLAGS)
TEST_FLAGS ?= $(CARGO_FLAGS)
NEXTTEST_CMD := nextest run --no-tests pass
TEST_CMD := $(if $(shell $(CARGO) nextest --version 2>/dev/null),$(NEXTTEST_CMD),test)
CARGO_FMT_WORKSPACE_FLAG := $(if $(shell $(CARGO) fmt --help 2>/dev/null | grep -q -- '--workspace' && echo yes),--workspace,--all)
JQ ?= jq
DOC_TEST_TARGETS ?= $(shell if command -v $(JQ) >/dev/null 2>&1; then $(CARGO) metadata --no-deps --format-version 1 2>/dev/null | $(JQ) -r 'any(.packages[].targets[]; (.kind | index("lib")) or (.kind | index("proc-macro")))' 2>/dev/null; else echo jq-missing; fi)
MDLINT ?= markdownlint-cli2
# `make fmt` and `make check-fmt` call mdtablefix directly. The selected paths
# include tracked Markdown and unignored Markdown files, so a new document is
# formatted before it is staged. The two preservation fixtures are deliberately
# excluded: they contain non-contiguous ordered-looking indented-code lines
# whose byte-identical preservation is the behaviour under test.
# Both modes need mdtablefix 0.6.0 or later; CI pins the version at the
# install-mdtablefix step.
MDTABLEFIX ?= mdtablefix
MDTABLEFIX_SELECT = $(shell git ls-files --cached --others --exclude-standard -- '*.md' \
	':!tests/fixtures/golden/insert_task_preserves_indented_code_markers/target.md' \
	':!tests/fixtures/golden/insert_task_preserves_indented_code_markers/expected.md')
MDTABLEFIX_RULES = --wrap --renumber --breaks --ellipsis --fences
MDFIX ?= $(MDTABLEFIX)
MARKDOWN_PATHS ?=
MARKDOWN_FORMAT_FLAGS ?= $(MDTABLEFIX_RULES) --in-place
MERMAN ?= merman-cli
NIXIE_RENDERER_THREADS ?= 1
NIXIE_MAX_CONCURRENCY ?= 1
NIXIE_FLAGS ?= -j $(NIXIE_MAX_CONCURRENCY)
NIXIE_PATHS ?= $(shell git ls-files '*.md')
UV ?= uv
UV_ENV = UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools
TYPOS_CONFIG_BUILDER_VERSION ?= v0.1.1
TYPOS_CONFIG_BUILDER = $(UV_ENV) $(UV) tool run --python 3.14 --from \
	"git+https://github.com/leynos/typos-config-builder.git@$(TYPOS_CONFIG_BUILDER_VERSION)" \
	typos-config-builder
RG ?= rg
# `rust-prover-tools` is pinned to a commit, not a release: it resolves the
# Verus download, checksum, and invocation, and CI must get the same runner the
# proofs were developed against. `uvx` runs it from a cached environment.
PROVER_TOOLS ?= uvx --from git+https://github.com/leynos/rust-prover-tools@$(shell cat tools/rust-prover-tools/REF) prover-tools
# The pinned Verus release refuses to run under a rustup-managed toolchain
# override, and this repository pins `nightly-2026-03-26` in
# `rust-toolchain.toml`. The override is cleared for the proof run only; cargo
# builds keep the repository toolchain.
VERUS_RUN ?= env -u RUSTUP_TOOLCHAIN $(PROVER_TOOLS) verus run --repo-root .

define require_markdown_paths
$(if $(strip $(MARKDOWN_PATHS)),,$(error set MARKDOWN_PATHS='docs/users-guide.md [more.md...]'))
endef

define require_markdown_in_place
$(if $(filter --in-place,$(MARKDOWN_FORMAT_FLAGS)),,$(error set MARKDOWN_FORMAT_FLAGS with --in-place for markdownfmt))
endef

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: check-fmt lint typecheck test spelling ## Perform a comprehensive check of code and prose

clean: ## Remove build artifacts
	$(CARGO) clean

test: ## Run tests with warnings treated as errors
	@if [ "$(DOC_TEST_TARGETS)" = "jq-missing" ]; then \
		echo "error: jq is required to detect doctest-capable packages; install jq or set DOC_TEST_TARGETS=true/false" >&2; \
		exit 2; \
	fi
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) $(TEST_CMD) $(TEST_FLAGS) $(BUILD_JOBS)
ifeq ($(DOC_TEST_TARGETS),true)
	RUSTFLAGS="$(RUST_FLAGS)" RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) test --doc --workspace --all-features
endif

test-workflow-contracts: ## Validate the mutation-testing caller contract
	uv run --with 'pytest>=8' --with 'pyyaml>=6' pytest tests/workflow_contracts -q

target/%/$(TARGET): ## Build binary in debug or release mode
	$(CARGO) build $(BUILD_JOBS) $(if $(findstring release,$(@)),--release) --bin $(TARGET)

lint: check-verification-ledger ## Run Clippy and the Whitaker Dylint suite with warnings denied
	RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) doc --no-deps
	$(CARGO) clippy $(CLIPPY_FLAGS)
	RUSTFLAGS="$(RUST_FLAGS)" $(WHITAKER) --all -- $(CARGO_FLAGS)

typecheck: ## Type-check without building
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) check $(CARGO_FLAGS)

fmt: ## Format Rust and Markdown sources
	$(CARGO) fmt $(CARGO_FMT_WORKSPACE_FLAG)
	$(MDTABLEFIX) --in-place $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)
	$(MDLINT) --fix "**/*.md"

check-fmt: ## Verify formatting
	$(CARGO) fmt $(CARGO_FMT_WORKSPACE_FLAG) -- --check
	$(MDTABLEFIX) --check $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)

markdownfmt: ## Format Markdown files listed in MARKDOWN_PATHS
	$(call require_markdown_paths)
	$(call require_markdown_in_place)
	$(MDFIX) $(MARKDOWN_FORMAT_FLAGS) $(MARKDOWN_PATHS)
	$(MDLINT) --fix --no-globs -- $(MARKDOWN_PATHS)

markdownlint: spelling ## Lint Markdown files and enforce spelling
	$(MDLINT) '**/*.md'

spelling: ## Enforce en-GB-oxendict spelling
	$(TYPOS_CONFIG_BUILDER) gate --repository .

markdownlint-paths: ## Lint Markdown files listed in MARKDOWN_PATHS
	$(call require_markdown_paths)
	$(MDLINT) --no-globs -- $(MARKDOWN_PATHS)

check-ripgrep: ## Verify ripgrep is available
	@command -v "$(firstword $(RG))" >/dev/null 2>&1 || { \
		echo "ripgrep (rg) is required for the verification-ledger check" >&2; \
		exit 1; \
	}

check-verification-ledger: check-ripgrep ## Verify verification-ledger symbols exist
	@RG='$(RG)' scripts/check-verification-ledger.sh .

check-prover-tools: ## Verify the configured prover-tools runner is available
	@command -v "$(firstword $(PROVER_TOOLS))" >/dev/null 2>&1 || { \
		echo "prover-tools runner ($(firstword $(PROVER_TOOLS))) is required for Verus verification" >&2; \
		exit 1; \
	}

verus-install: check-prover-tools ## Install the pinned Verus release
	$(PROVER_TOOLS) verus install --repo-root .

verus: verus-install ## Verify the production-used Verus proof entry point
	$(VERUS_RUN) --proof-file verus/lib.rs

verus-selftest: verus-install ## Confirm Verus rejects the deliberately false smoke proof
	@output="$$(mktemp)"; \
	if $(VERUS_RUN) --proof-file verus/smoke.rs >"$$output" 2>&1; then \
		cat "$$output"; \
		rm -f "$$output"; \
		echo "Verus smoke proof unexpectedly succeeded" >&2; \
		exit 1; \
	fi; \
	if ! grep -Fq "Verus proofs failed" "$$output"; then \
		cat "$$output"; \
		rm -f "$$output"; \
		echo "Verus smoke proof did not reach the verifier" >&2; \
		exit 1; \
	fi; \
	cat "$$output"; \
	rm -f "$$output"

nixie: ## Validate Mermaid diagrams
	set -e; artefacts_dir="$$(mktemp -d)"; trap 'rm -rf "$$artefacts_dir"' EXIT; \
	for path in $(NIXIE_PATHS); do \
		RAYON_NUM_THREADS="$(NIXIE_RENDERER_THREADS)" $(MERMAN) $(NIXIE_FLAGS) \
			-i "$$path" -a "$$artefacts_dir"; \
	done

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'
