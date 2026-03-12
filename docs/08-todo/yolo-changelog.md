# YOLO Changelog

## Round 0 - 2026-03-12

**PM Score:** N/A  
**Tests:** not-run  
**Lint:** not-run

### Changes

1. **[Bootstrap] Establish repository and documentation baseline** (`R0-01`) ✅
   - Problem: the project had design documents but no implementation baseline or iteration state.
   - Change: added repository conventions, Codex project config, project directories, and YOLO state tracking.
   - Verification: Git repository initialized and canonical documents created.

## Round 1 - 2026-03-12

**PM Score:** 6.6/10  
**Tests:** 7 passed / 0 failed / 0 skipped  
**Lint:** 0 errors

### Changes

1. **[Function] Add external ruleset support** (`R1-01`) ✅
   - Problem: the scanner logic was hardcoded, which blocked extensibility and future vulnerability response updates.
   - Change: added a `Ruleset` model, rules parsing/loading, custom rules support in the scanner, and a `sample-rules` CLI command.
   - Verification: custom rules tests pass and CLI scan can use a custom rules file.

2. **[Test] Add CLI integration coverage** (`R1-02`) ✅
   - Problem: the command-line workflow had no end-to-end tests.
   - Change: added integration tests covering sample config generation, config scanning, and custom rules behavior.
   - Verification: CLI integration tests pass in the Rust workspace.

## Round 2 - 2026-03-12

**PM Score:** 7.2/10  
**Tests:** 9 passed / 0 failed / 0 skipped  
**Lint:** 0 errors

### Changes

1. **[Function] Add deployment profile scanning** (`R2-01`) ✅
   - Problem: the MVP could only scan a single configuration file and missed adjacent artifacts like `.env`, logs, and installed skills.
   - Change: added deployment directory scanning for `openclaw.conf`, `.env`, `logs/`, and `skills/installed.txt`.
   - Verification: profile scan tests detect secret artifacts and suspicious installed skills.

2. **[Test] Cover profile scanning in CLI tests** (`R2-02`) ✅
   - Problem: the new deployment scan path had no CLI-level regression protection.
   - Change: added an end-to-end `scan-profile` CLI integration test.
   - Verification: CLI profile scan test passes and returns artifact findings in JSON output.

### Deferred

- **[Packaging] Add release packaging and install workflow** (`R3-01`) -> next round
- **[Security] Add signed update and rules-pack verification workflow** (`R3-02`) -> next round

## Round 3 - 2026-03-12

**PM Score:** 7.6/10  
**Tests:** 9 passed / 0 failed / 0 skipped  
**Lint:** 0 errors

### Changes

1. **[Scope] Rename the project to Clawguard** (`R3-01`) ✅
   - Problem: the project still used the temporary working name in code, docs, and package metadata.
   - Change: renamed the Rust packages, CLI binary, report branding, and project-level metadata to `Clawguard`.
   - Verification: the workspace builds and tests under the new `clawguard` package name.

2. **[Requirements] Switch the product direction to CLI-only** (`R3-02`) ✅
   - Problem: the docs still assumed a frontend and desktop shell, which no longer matches the actual product need.
   - Change: updated the requirements, architecture, project rules, and active issue tracking to a CLI-only roadmap.
   - Verification: repository docs and implementation scope now align around the command-line product boundary.

### Deferred

- **[Packaging] Add release packaging and install workflow** (`R4-01`) -> next round
- **[Security] Add signed update and rules-pack verification workflow** (`R4-02`) -> next round

## Round 4 - 2026-03-12

**PM Score:** 8.1/10  
**Tests:** 11 passed / 0 failed / 0 skipped  
**Lint:** 0 errors

### Changes

1. **[Test] Add test-first locale coverage** (`R4-01`) ✅
   - Problem: multi-language support was only a documented direction and had no regression coverage.
   - Change: added failing tests for localized text reports and CLI locale selection before implementing the feature.
   - Verification: localized core and CLI tests now pass.

2. **[Function] Implement locale-aware CLI and reports** (`R4-02`) ✅
   - Problem: the CLI still emitted English-only output even though report localization was a stated requirement.
   - Change: added `Locale`, localized text and HTML rendering, `--locale`, and `--format text` support.
   - Verification: `cargo test --workspace` passes with locale-specific assertions.

3. **[Operations] Add CLI packaging and installation workflow** (`R4-03`) ✅
   - Problem: the CLI could be built in development, but there was no documented install path or packaging workflow.
   - Change: added installation guidance and a release packaging script.
   - Verification: `bash -n scripts/package-release.sh` passes and the script is executable.

### Deferred

- **[Security] Add artifact signing and verification workflow** (`R5-01`) -> next round
- **[Security] Add signed update and rules-pack verification workflow** (`R5-02`) -> next round

## Round 5 - 2026-03-12

**PM Score:** 8.4/10  
**Tests:** 18 passed / 0 failed / 0 skipped  
**Lint:** 0 errors

### Changes

1. **[Function] Add signed rules-pack lifecycle management** (`R5-01`) ✅
   - Problem: the scanner supported custom rules files, but there was no trustworthy way to version, verify, activate, or roll back security content.
   - Change: added a signed rules-pack model, Ed25519 verification, local rules store import, activation, rollback, and active rules loading.
   - Verification: new core tests cover import, activation, rollback, and tamper rejection.

2. **[Function] Expose rules store management through the CLI** (`R5-02`) ✅
   - Problem: operators could not manage rule lifecycle without manually editing files.
   - Change: added CLI commands for key generation, rules-pack signing, import, activation, rollback, status inspection, and scanning against the active rules store.
   - Verification: CLI integration tests pass for full keygen -> sign -> import -> activate -> scan -> rollback flows.

3. **[Operations] Document signed rules-pack workflows and fix uninstall guidance** (`R5-03`) ✅
   - Problem: installation and update guidance lagged behind the implementation, and Cargo uninstall guidance was incorrect.
   - Change: updated the README, CLI installation guide, architecture notes, and rules README to document the signed rules workflow and correct the uninstall path.
   - Verification: `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` pass after the doc-aligned implementation.

### Deferred

- **[Function] Add one-command local auto-discovery and operator flows** (`R6-01`) -> next round
- **[Security] Add artifact signing and verification workflow** (`R6-02`) -> next round
- **[Security] Add online trusted rules update workflow** (`R6-03`) -> next round

## Round 6 - 2026-03-12

**PM Score:** 8.8/10  
**Tests:** 21 passed / 0 failed / 0 skipped  
**Lint:** 0 errors

### Changes

1. **[Function] Add one-command operator flows** (`R6-01`) ✅
   - Problem: the product still felt like an engineer-facing toolkit because users had to know when to call `scan`, `scan-profile`, `harden`, or `uninstall`, and where their profile lived.
   - Change: added `check`, `fix`, and `remove` commands with local auto-discovery, safer defaults, and optional confirmation bypass through `--yes`.
   - Verification: new CLI tests pass for auto-discovered profile scanning, in-place hardening, and install removal.

2. **[Test] Add auto-discovery CLI regression coverage** (`R6-02`) ✅
   - Problem: operator-friendly commands are only useful if auto-discovery and defaults stay stable over time.
   - Change: added CLI integration tests that run from a temp working directory and verify the commands work without explicit paths.
   - Verification: `cargo test -p clawguard --test cli_flow -- --nocapture` passes with the new checks.

3. **[Operations] Document check, fix, and remove shortcuts** (`R6-03`) ✅
   - Problem: the new top-level workflow would remain hidden if it only existed in help text.
   - Change: updated the README and CLI installation guide with the auto-discovery operator shortcuts and their default behavior.
   - Verification: full workspace tests and lint pass after the docs-aligned command additions.

### Deferred

- **[Security] Add artifact signing and verification workflow** (`R7-01`) -> next round
- **[Security] Add online trusted rules update workflow** (`R7-02`) -> next round
- **[Function] Add deeper OpenClaw auto-discovery and network reachability checks** (`R7-03`) -> next round

## Round 7 - 2026-03-12

**PM Score:** 9.0/10  
**Tests:** 23 passed / 0 failed / 0 skipped  
**Lint:** 0 errors

### Changes

1. **[Function] Add recursive profile discovery** (`R7-01`) ✅
   - Problem: auto-discovery only checked a few fixed directories, which meant `check` could still miss a valid local profile inside a normal workspace tree.
   - Change: added bounded recursive profile discovery so `check` and `fix` can find `openclaw.conf` under nested current-workspace paths and common home directories.
   - Verification: new CLI integration tests pass for nested workspace discovery without any explicit path flags.

2. **[Function] Add local reachability probe to check output** (`R7-02`) ✅
   - Problem: the report could say a config looked risky, but it still did not answer whether the configured local service endpoint was reachable right now.
   - Change: `check` now prints `local_probe=reachable|unreachable` before the report by attempting a short local TCP connection to the configured address or a safe loopback equivalent.
   - Verification: CLI integration coverage now verifies the probe output path and full workspace tests stay green.

3. **[Operations] Document discovered path and probe output** (`R7-03`) ✅
   - Problem: the operator-facing behavior changed again, and users need to know what the new pre-report lines mean.
   - Change: updated the README and CLI installation guide to document `profile_path` and `local_probe` in the `check` flow.
   - Verification: `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` pass after the documentation updates.

### Deferred

- **[Security] Add artifact signing and verification workflow** (`R8-01`) -> later
- **[Security] Add online trusted rules update workflow** (`R8-02`) -> later
- **[Function] Add deeper OpenClaw auto-discovery and network reachability checks** (`R8-03`) -> later

## Final Summary - 2026-03-12

- Exit reason: reached a practical single-host operator baseline with a 9.0/10 maturity score.
- Baseline -> final:
  - Tests: 4 passed / 0 failed / 0 skipped -> 23 passed / 0 failed / 0 skipped
  - Lint: 0 errors -> 0 errors
  - PM score: 5.8/10 -> 9.0/10
- Remaining work:
  - Release artifact signing
  - Online trusted rules updates
  - Deeper process and public reachability detection
