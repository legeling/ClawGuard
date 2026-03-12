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

- **[Desktop] Bootstrap desktop shell and frontend workspace** (`R3-01`) -> next round
- **[Security] Add signed update and rules-pack verification workflow** (`R3-02`) -> next round
