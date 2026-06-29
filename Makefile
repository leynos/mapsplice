.PHONY: help all clean test build release lint fmt check-fmt markdownlint nixie typecheck


TARGET ?= mapsplice

CARGO ?= cargo
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
NIXIE ?= nixie

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: check-fmt lint test ## Perform a comprehensive check of code

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

target/%/$(TARGET): ## Build binary in debug or release mode
	$(CARGO) build $(BUILD_JOBS) $(if $(findstring release,$(@)),--release) --bin $(TARGET)

lint: ## Run Clippy with warnings denied
	RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) doc --no-deps
	$(CARGO) clippy $(CLIPPY_FLAGS)
	RUSTFLAGS="$(RUST_FLAGS)" whitaker --all -- $(CARGO_FLAGS)

typecheck: ## Type-check without building
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) check $(CARGO_FLAGS)

fmt: ## Format Rust and Markdown sources
	$(CARGO) fmt $(CARGO_FMT_WORKSPACE_FLAG)
	mdformat-all

check-fmt: ## Verify formatting
	$(CARGO) fmt $(CARGO_FMT_WORKSPACE_FLAG) -- --check

markdownlint: ## Lint Markdown files
	$(MDLINT) '**/*.md'

nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'
