# YOLO State - Clawguard

requirement: "Build Clawguard end-to-end from requirements to implementation"
mode: evolve
started: 2026-03-12
current_round: 8
max_rounds: 10
total_improvements: 22
status: done

toolchain:
  language: "rust,node"
  lint_cmd: "cargo clippy --workspace --all-targets -- -D warnings"
  test_cmd: "cargo test --workspace"
  build_cmd: "cargo build --workspace"

code_map:
  source_files:
    - path: "crates/core-engine/src/lib.rs"
      lines: 1180
      exports:
        - "OpenClawConfig"
        - "Ruleset"
        - "scan_config"
        - "scan_config_with_rules"
        - "scan_profile_dir"
        - "scan_profile_with_rules"
        - "harden_config_file"
        - "load_ruleset"
    - path: "crates/core-engine/src/rules_store.rs"
      lines: 413
      exports:
        - "RulesPackPayload"
        - "SignedRulesPack"
        - "import_rules_pack"
        - "activate_rules_pack"
        - "rollback_rules_pack"
        - "load_active_ruleset"
    - path: "crates/cli/src/main.rs"
      lines: 459
      exports:
        - "main"
        - "run_scan"
        - "run_scan_profile"
        - "run_harden"
        - "run_import_rules_pack"
        - "run_activate_rules"
        - "run_rollback_rules"
  entrypoints:
    - "crates/cli/src/main.rs"
  test_mappings:
    - source: "crates/core-engine/src/lib.rs"
      tests:
        - "crates/core-engine/tests/ruleset.rs"
        - "crates/core-engine/tests/profile_scan.rs"
        - "crates/core-engine/tests/localization.rs"
    - source: "crates/core-engine/src/rules_store.rs"
      tests:
        - "crates/core-engine/tests/rules_pack_store.rs"
    - source: "crates/cli/src/main.rs"
      tests:
        - "crates/cli/tests/cli_flow.rs"

git:
  baseline_commit: "45c3c022b7ed24db114dae543476c337f8b4331f"
  current_commit: "4bf11fc3d9aba552bd484f78a74556a84176f375"
  start_tag: "yolo-start-20260312-115050"

baseline:
  build_ok: true
  lint_errors: 0
  test_summary: "4 passed, 0 failed, 0 skipped"

conductor:
  trend: "rising"
  blocked_dimensions: []
  failure_patterns:
    - "CLI integration tests initially assumed Cargo would inject a binary path env var in this workspace layout."
    - "Adding cryptographic signing required fetching new crates, so workspace verification now depends on network-enabled dependency resolution."
    - "Release verification spans bash, Node, and GitHub Actions, so a change in one layer can silently desynchronize the others without explicit packaging tests."
  efficiency: "high"
  strategy: "Continue hardening the release path so the security tool ships trustworthy artifacts, then pause on a clean checkpoint for publishing and repo positioning."

rounds:
  - round: 0
    test_summary: "4 passed, 0 failed, 0 skipped"
    lint_errors: 0
    pm_score: 5.8
    improvements:
      - id: R0-01
        dimension: "function"
        title: "Bootstrap Rust core and CLI"
        status: done
        files_changed:
          - "Cargo.toml"
          - "crates/core-engine/src/lib.rs"
          - "crates/cli/src/main.rs"
  - round: 1
    test_summary: "7 passed, 0 failed, 0 skipped"
    lint_errors: 0
    pm_score: 6.6
    improvements:
      - id: R1-01
        dimension: "function"
        title: "Add external ruleset support"
        status: done
        files_changed:
          - "crates/core-engine/src/lib.rs"
          - "crates/core-engine/tests/ruleset.rs"
          - "crates/cli/src/main.rs"
      - id: R1-02
        dimension: "test"
        title: "Add CLI integration coverage"
        status: done
        files_changed:
          - "crates/cli/tests/cli_flow.rs"
  - round: 2
    test_summary: "9 passed, 0 failed, 0 skipped"
    lint_errors: 0
    pm_score: 7.2
    improvements:
      - id: R2-01
        dimension: "function"
        title: "Add deployment profile scanning"
        status: done
        files_changed:
          - "crates/core-engine/src/lib.rs"
          - "crates/core-engine/tests/profile_scan.rs"
          - "crates/cli/src/main.rs"
      - id: R2-02
        dimension: "test"
        title: "Cover profile scanning in CLI tests"
        status: done
        files_changed:
          - "crates/cli/tests/cli_flow.rs"
  - round: 3
    test_summary: "9 passed, 0 failed, 0 skipped"
    lint_errors: 0
    pm_score: 7.6
    improvements:
      - id: R3-01
        dimension: "scope"
        title: "Rename the project to Clawguard"
        status: done
        files_changed:
          - "README.md"
          - "AGENTS.md"
          - ".codex/config.toml"
          - "crates/cli/Cargo.toml"
          - "crates/core-engine/Cargo.toml"
      - id: R3-02
        dimension: "requirements"
        title: "Switch the product direction to CLI-only"
        status: done
        files_changed:
          - "docs/openclaw-guard-requirements.md"
          - "docs/openclaw-guard-architecture.md"
          - "docs/openclaw-security-insights.md"
          - "docs/09-issues/active/cli-packaging-distribution.md"
  - round: 4
    test_summary: "11 passed, 0 failed, 0 skipped"
    lint_errors: 0
    pm_score: 8.1
    improvements:
      - id: R4-01
        dimension: "test"
        title: "Add test-first locale coverage"
        status: done
        files_changed:
          - "crates/core-engine/tests/localization.rs"
          - "crates/cli/tests/cli_flow.rs"
      - id: R4-02
        dimension: "function"
        title: "Implement locale-aware CLI and reports"
        status: done
        files_changed:
          - "crates/core-engine/src/lib.rs"
          - "crates/cli/src/main.rs"
      - id: R4-03
        dimension: "operations"
        title: "Add CLI packaging and installation workflow"
        status: done
        files_changed:
          - "docs/cli-installation.md"
          - "scripts/package-release.sh"
          - "README.md"
  - round: 5
    test_summary: "18 passed, 0 failed, 0 skipped"
    lint_errors: 0
    pm_score: 8.4
    improvements:
      - id: R5-01
        dimension: "function"
        title: "Add signed rules-pack lifecycle management"
        status: done
        files_changed:
          - "crates/core-engine/Cargo.toml"
          - "crates/core-engine/src/lib.rs"
          - "crates/core-engine/src/rules_store.rs"
          - "crates/core-engine/tests/rules_pack_store.rs"
      - id: R5-02
        dimension: "function"
        title: "Expose rules store management through the CLI"
        status: done
        files_changed:
          - "crates/cli/src/main.rs"
          - "crates/cli/tests/cli_flow.rs"
      - id: R5-03
        dimension: "operations"
        title: "Document signed rules-pack workflows and fix uninstall guidance"
        status: done
        files_changed:
          - "README.md"
          - "docs/cli-installation.md"
          - "docs/openclaw-guard-architecture.md"
          - "rules/README.md"
  - round: 6
    test_summary: "21 passed, 0 failed, 0 skipped"
    lint_errors: 0
    pm_score: 8.8
    improvements:
      - id: R6-01
        dimension: "function"
        title: "Add one-command operator flows"
        status: done
        files_changed:
          - "crates/cli/src/main.rs"
      - id: R6-02
        dimension: "test"
        title: "Add auto-discovery CLI regression coverage"
        status: done
        files_changed:
          - "crates/cli/tests/cli_flow.rs"
      - id: R6-03
        dimension: "operations"
        title: "Document check, fix, and remove shortcuts"
        status: done
        files_changed:
          - "README.md"
          - "docs/cli-installation.md"
  - round: 7
    test_summary: "23 passed, 0 failed, 0 skipped"
    lint_errors: 0
    pm_score: 9.0
    improvements:
      - id: R7-01
        dimension: "function"
        title: "Add recursive profile discovery"
        status: done
        files_changed:
          - "crates/cli/src/main.rs"
          - "crates/cli/tests/cli_flow.rs"
      - id: R7-02
        dimension: "function"
        title: "Add local reachability probe to check output"
        status: done
        files_changed:
          - "crates/cli/src/main.rs"
          - "crates/cli/tests/cli_flow.rs"
      - id: R7-03
        dimension: "operations"
        title: "Document discovered path and probe output"
        status: done
        files_changed:
          - "README.md"
          - "docs/cli-installation.md"
  - round: 8
    test_summary: "23 passed, 0 failed, 0 skipped"
    lint_errors: 0
    pm_score: 9.2
    improvements:
      - id: R8-01
        dimension: "security"
        title: "Add signed release manifest generation"
        status: done
        files_changed:
          - "scripts/package-release.sh"
          - "keys/release-public.pem"
          - "scripts/test-release-package.js"
      - id: R8-02
        dimension: "security"
        title: "Verify release manifests during curl and npm installs"
        status: done
        files_changed:
          - "scripts/install-clawguard.sh"
          - "packages/npm/clawguard/scripts/install-lib.js"
          - "packages/npm/clawguard/scripts/test-install.js"
          - "packages/npm/clawguard/README.md"
      - id: R8-03
        dimension: "operations"
        title: "Enforce release signing in CI and document the signing secret"
        status: done
        files_changed:
          - ".github/workflows/ci.yml"
          - ".github/workflows/release.yml"
          - "docs/cicd.md"
          - "README.md"
          - "docs/cli-installation.md"

deferred_issues:
  - id: R9-01
    title: "Add online trusted rules update workflow"
    impact: 5
    reason: "Rules packs can be signed and imported locally, but there is no online update check, trusted keyring, or staged download flow."
  - id: R9-02
    title: "Add deeper OpenClaw auto-discovery and network reachability checks"
    impact: 4
    reason: "Local profile auto-discovery now exists, but the product still lacks process, port, and public reachability detection."
  - id: R9-03
    title: "Publish signed release channels and verify them end to end"
    impact: 4
    reason: "The signing workflow is implemented, but it still needs a real repository secret, published artifacts, and a smoke-tested release run."

failure_lessons:
  - round: 1
    improvement_id: "R1-02"
    failure_type: "test_failure"
    description: "CLI integration tests assumed a compile-time cargo binary environment variable."
    takeaway: "Infer the test binary path from current_exe or use a runtime-provided path instead of compile-time macros."
  - round: 4
    improvement_id: "R4-02"
    failure_type: "lint_error"
    description: "Locale support introduced a Clippy lifetime warning and an unused import during the green phase."
    takeaway: "Treat lint as part of the TDD verify step and keep CLI imports minimal after refactors."
  - round: 5
    improvement_id: "R5-01"
    failure_type: "build_error"
    description: "The signed rules-pack implementation introduced new cryptography dependencies that could not be fetched under sandboxed network restrictions."
    takeaway: "When a feature requires new ecosystem crates, expect dependency download to be part of the verification path and request escalation early."
  - round: 6
    improvement_id: "R6-01"
    failure_type: "test_failure"
    description: "The first red-phase test run used an invalid cargo test invocation pattern, so the failure signal was about the harness command instead of the missing feature."
    takeaway: "When targeting specific Rust integration tests, validate the cargo test syntax first so the red phase reflects the feature gap rather than command misuse."
  - round: 7
    improvement_id: "R7-02"
    failure_type: "test_environment"
    description: "The sandbox does not permit binding a test TCP listener, so a positive probe test could not rely on a real local socket."
    takeaway: "Keep the production probe real, but let tests override probe results through a dedicated environment hook when sandbox networking is restricted."
  - round: 8
    improvement_id: "R8-01"
    failure_type: "integration_gap"
    description: "The release signing feature initially lacked a packaging-level regression test, which would have allowed the shell script, npm verifier, and workflow expectations to drift apart."
    takeaway: "For cross-layer release logic, add a dedicated package-level test before touching the implementation."

round_decisions:
  - round: 1
    note: "Prioritized extensibility and CLI coverage over any shell expansion."
  - round: 2
    note: "Prioritized real deployment directory scanning because config-only scanning was too narrow for the documented requirements."
  - round: 3
    note: "The product direction is now explicitly CLI-only, so desktop and frontend work was removed from the active roadmap."
  - round: 4
    note: "Prioritized locale-aware output and packaging because the MVP was runnable but still weak on operator usability and distribution."
  - round: 5
    note: "Prioritized a signed rules runtime before adding more detectors, because updateability and trust are product-critical for a security tool."
  - round: 6
    note: "Prioritized check/fix/remove because the product needed user-facing command ergonomics more than additional low-level subcommands."
  - round: 7
    note: "Stopped after recursive discovery and local probe reporting because the CLI now satisfies the single-host operator workflow that drove this iteration."
  - round: 8
    note: "Resumed to harden the release supply chain, then paused again on a publish-ready checkpoint."
