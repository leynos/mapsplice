.PHONY: help all clean test build release lint lint-clippy lint-whitaker fmt check-fmt markdownfmt \
	markdownlint markdownlint-paths nixie typecheck test-workflow-contracts \
	spelling install-build-tools check-build-tools

# A composite gate must not overlap its Cargo invocations under `make -j`.
.NOTPARALLEL:


TARGET ?= mapsplice

CARGO ?= cargo
WHITAKER ?= whitaker
WHITAKER_PACKAGES ?= mapsplice
WHITAKER_ENV = CARGO_ENCODED_RUSTFLAGS="$(call append_filtered_encoded_rust_flags,$(RUST_FLAGS))" RUSTFLAGS="$(RUST_FLAGS)" CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm CARGO_PROFILE_TEST_CODEGEN_BACKEND=llvm
BUILD_TOOLS_PREFIX ?= $(HOME)/.local
BUILD_JOBS ?=
RUST_FLAGS ?=
RUST_FLAGS := -D warnings $(RUST_FLAGS)
# Cargo selects one rustflags source. Recipe-level RUSTFLAGS therefore repeat
# the auto-discovered development flags as well as the warning policy.
HOST_OS := $(shell uname -s)
HOST_ARCH := $(shell uname -m)
# Cargo only has committed `mold` linker routes for these Linux targets. Parse
# command-line, environment and --config target selectors. Conflicting input
# fails closed so an uncertain effective target never receives the ELF linker.
SUPPORTED_LINKER_TARGETS = x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
CARGO_CONFIG_BUILD_TARGET := $(shell python3 scripts/resolve-cargo-build-target.py)
ifneq ($(.SHELLSTATUS),0)
CARGO_CONFIG_BUILD_TARGET := $(error $(CARGO_CONFIG_BUILD_TARGET))
endif
targets_from_words = $(if $(strip $(1)),$(if $(filter --target,$(firstword $(1))),$(word 2,$(1)) $(call targets_from_words,$(wordlist 3,$(words $(1)),$(1))),$(if $(filter --target=%,$(firstword $(1))),$(patsubst --target=%,%,$(firstword $(1))) $(call targets_from_words,$(wordlist 2,$(words $(1)),$(1))),$(call targets_from_words,$(wordlist 2,$(words $(1)),$(1))))))
config_targets_from_words = $(if $(strip $(1)),$(if $(filter --config,$(firstword $(1))),$(patsubst build.target=%,%,$(filter build.target=%,$(word 2,$(1)))) $(call config_targets_from_words,$(wordlist 3,$(words $(1)),$(1))),$(if $(filter --config=build.target=%,$(firstword $(1))),$(patsubst --config=build.target=%,%,$(firstword $(1))) $(call config_targets_from_words,$(wordlist 2,$(words $(1)),$(1))),$(call config_targets_from_words,$(wordlist 2,$(words $(1)),$(1))))))
target_input_present = $(strip $(filter --target --target=%,$(1)) $(filter --config=build.target=%,$(1)) $(call config_targets_from_words,$(1)) $(CARGO_BUILD_TARGET) $(CARGO_CONFIG_BUILD_TARGET))
requested_targets = $(strip $(call targets_from_words,$(1)) $(call config_targets_from_words,$(1)) $(CARGO_BUILD_TARGET) $(CARGO_CONFIG_BUILD_TARGET))
native_linker_flag = $(if $(filter Linux,$(HOST_OS)),$(if $(filter x86_64 aarch64,$(HOST_ARCH)),-C link-arg=-fuse-ld=mold))
linker_flag_for = $(if $(call target_input_present,$(1)),$(if $(filter 1,$(words $(sort $(call requested_targets,$(1))))),$(if $(filter $(SUPPORTED_LINKER_TARGETS),$(call requested_targets,$(1))),-C link-arg=-fuse-ld=mold)),$(native_linker_flag))
gate_rust_flags = $(RUST_FLAGS) -Zthreads=8 $(call linker_flag_for,$(1))
empty :=
space := $(empty) $(empty)
unit_separator := $(shell printf '\037')
encode_rust_flags = $(subst $(space),$(unit_separator),$(strip $(1)))
# Keep caller policy flags, but remove only development-owned arguments before
# selecting a route. This prevents a caller's encoded Rust flags from leaking
# Cranelift, the parallel frontend, or `mold` into release and verification.
filter_encoded_rust_flags = $$(bash -c 'set -euo pipefail; encoded=$${CARGO_ENCODED_RUSTFLAGS-}; separator=$$(printf "\\037"); arguments=(); if [[ -n $$encoded ]]; then IFS=$$separator read -r -a arguments <<< "$$encoded"; fi; pending=; kept=(); for current in "$${arguments[@]}"; do if [[ -n $$pending ]]; then case "$$pending:$$current" in -Z:threads=8|-Z:codegen-backend=cranelift|-C:link-arg=-fuse-ld=mold) pending=; continue;; *) kept+=("$$pending"); pending=;; esac; fi; case $$current in -Z|-C) pending=$$current;; -Zthreads=8|-Zcodegen-backend=cranelift|-Clink-arg=-fuse-ld=mold) ;; *) kept+=("$$current");; esac; done; if [[ -n $$pending ]]; then kept+=("$$pending"); fi; (IFS=$$separator; printf "%s" "$${kept[*]}")')
append_filtered_encoded_rust_flags = $$(filtered=$(call filter_encoded_rust_flags); if [ -n "$$filtered" ]; then printf '%s$(unit_separator)%s' "$$filtered" "$(call encode_rust_flags,$(1))"; else printf '%s' "$(call encode_rust_flags,$(1))"; fi)
RUSTDOC_FLAGS ?=
override RUSTDOC_FLAGS := --cfg docsrs -D warnings $(RUSTDOC_FLAGS)
CARGO_FLAGS ?= --workspace --all-targets --all-features
CLIPPY_FLAGS ?= $(CARGO_FLAGS) -- $(RUST_FLAGS)
TEST_FLAGS ?= $(CARGO_FLAGS)
NEXTTEST_CMD := nextest run --no-tests pass
TEST_CMD := $(if $(shell $(CARGO) nextest --version 2>/dev/null),$(NEXTTEST_CMD),test)
CARGO_FMT_WORKSPACE_FLAG := $(if $(shell $(CARGO) fmt --help 2>/dev/null | grep -q -- '--workspace' && echo yes),--workspace,--all)
JQ ?= jq
DOC_TEST_TARGETS ?= $(shell if command -v $(JQ) >/dev/null 2>&1; then $(CARGO) metadata --no-deps --format-version 1 2>/dev/null | $(JQ) -r 'any(.packages[].targets[]; (.kind | index("lib")) or (.kind | index("proc-macro")))' 2>/dev/null; else echo jq-missing; fi)
MDLINT ?= markdownlint-cli2
# Git selection includes tracked and unignored untracked Markdown, including
# documents before they are staged. Fixture data with intentionally unusual
# Markdown is stored as .txt so it is not mistaken for maintainable prose.
# Both modes need mdtablefix 0.6.0 or later; CI pins the installed binary.
MDTABLEFIX ?= mdtablefix
MDTABLEFIX_SELECT = --git --include-untracked
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
TYPOS_CONFIG_BUILDER_VERSION ?= v0.1.3
TYPOS_CONFIG_BUILDER = $(UV_ENV) $(UV) tool run --python 3.14 --from \
	"git+https://github.com/leynos/typos-config-builder.git@$(TYPOS_CONFIG_BUILDER_VERSION)" \
	typos-config-builder

define require_markdown_paths
$(if $(strip $(MARKDOWN_PATHS)),,$(error set MARKDOWN_PATHS='docs/users-guide.md [more.md...]'))
endef

define require_markdown_in_place
$(if $(filter --in-place,$(MARKDOWN_FORMAT_FLAGS)),,$(error set MARKDOWN_FORMAT_FLAGS with --in-place for markdownfmt))
endef

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: check-fmt lint typecheck test spelling ## Perform a comprehensive check of code and prose

clean: ## Remove build artefacts
	$(CARGO) clean

test: check-build-tools ## Run tests with warnings treated as errors
	@if [ "$(DOC_TEST_TARGETS)" = "jq-missing" ]; then \
		echo "error: jq is required to detect doctest-capable packages; install jq or set DOC_TEST_TARGETS=true/false" >&2; \
		exit 2; \
	fi
	CARGO_ENCODED_RUSTFLAGS="$(call append_filtered_encoded_rust_flags,$(call gate_rust_flags,$(TEST_FLAGS)))" RUSTFLAGS="$(call gate_rust_flags,$(TEST_FLAGS))" $(CARGO) $(TEST_CMD) $(TEST_FLAGS) $(BUILD_JOBS)
ifeq ($(DOC_TEST_TARGETS),true)
	CARGO_ENCODED_RUSTFLAGS="$(call append_filtered_encoded_rust_flags,$(call gate_rust_flags,))" RUSTFLAGS="$(call gate_rust_flags,)" RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) test --doc --workspace --all-features
endif

test-workflow-contracts: ## Validate the mutation-testing caller contract
	uv run --with 'pytest>=8' --with 'pyyaml>=6' pytest tests/workflow_contracts -q

target/debug/$(TARGET): | check-build-tools ## Build the development binary
	CARGO_ENCODED_RUSTFLAGS="$(call append_filtered_encoded_rust_flags,$(call gate_rust_flags,))" RUSTFLAGS="$(call gate_rust_flags,)" $(CARGO) build $(BUILD_JOBS) --bin $(TARGET)

target/release/$(TARGET): ## Build the production binary with LLVM
	CARGO_ENCODED_RUSTFLAGS="$(call append_filtered_encoded_rust_flags,$(RUST_FLAGS))" RUSTFLAGS="$(RUST_FLAGS)" CARGO_PROFILE_RELEASE_CODEGEN_BACKEND=llvm \
		$(CARGO) build $(BUILD_JOBS) --release --bin $(TARGET)

lint: lint-clippy lint-whitaker ## Run Clippy, then the Whitaker Dylint suite

lint-clippy: check-build-tools ## Check Rust documentation and run Clippy
	CARGO_ENCODED_RUSTFLAGS="$(call append_filtered_encoded_rust_flags,$(call gate_rust_flags,))" RUSTFLAGS="$(call gate_rust_flags,)" RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) doc --workspace --no-deps
	CARGO_ENCODED_RUSTFLAGS="$(call append_filtered_encoded_rust_flags,$(call gate_rust_flags,$(CLIPPY_FLAGS)))" RUSTFLAGS="$(call gate_rust_flags,$(CLIPPY_FLAGS))" $(CARGO) clippy $(CLIPPY_FLAGS)

# Whitaker uses its installer-managed toolchain. A warnings-only RUSTFLAGS
# assignment keeps its route separate from the parallel development frontend
# and linker; explicit profile overrides also reject inherited backend input.
lint-whitaker: ## Run the rolling Whitaker suite over every workspace package
	$(WHITAKER_ENV) $(WHITAKER) --all $(foreach package,$(WHITAKER_PACKAGES),--package $(package)) -- $(CARGO_FLAGS)

typecheck: check-build-tools ## Type-check without building
	CARGO_ENCODED_RUSTFLAGS="$(call append_filtered_encoded_rust_flags,$(call gate_rust_flags,$(CARGO_FLAGS)))" RUSTFLAGS="$(call gate_rust_flags,$(CARGO_FLAGS))" $(CARGO) check $(CARGO_FLAGS)

install-build-tools: ## Install the pinned nightly and Linux linker
	@bash scripts/install-build-tools.sh

check-build-tools: ## Check the development compiler and linker prerequisites
	@bash scripts/check-build-tools.sh

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

markdownlint: ## Lint Markdown files
	$(MDLINT) '**/*.md'

spelling: ## Enforce en-GB-oxendict spelling
	$(TYPOS_CONFIG_BUILDER) gate --repository . --scope all

markdownlint-paths: ## Lint Markdown files listed in MARKDOWN_PATHS
	$(call require_markdown_paths)
	$(MDLINT) --no-globs -- $(MARKDOWN_PATHS)

nixie: ## Validate Mermaid diagrams
	set -e; artefacts_dir="$$(mktemp -d)"; trap 'rm -rf "$$artefacts_dir"' EXIT; \
	for path in $(NIXIE_PATHS); do \
		RAYON_NUM_THREADS="$(NIXIE_RENDERER_THREADS)" $(MERMAN) $(NIXIE_FLAGS) \
			-i "$$path" -a "$$artefacts_dir"; \
	done

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'
