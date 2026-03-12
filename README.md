# Clawguard

Clawguard is a CLI-first security audit and one-click hardening tool for OpenClaw deployments and adjacent ecosystem components.

## Current Status

This repository is in the design and project bootstrap stage.

Canonical project documents live in `docs/`:

- `docs/openclaw-security-insights.md`
- `docs/openclaw-guard-requirements.md`
- `docs/openclaw-guard-architecture.md`
- `docs/cli-installation.md`
- `docs/documentation-management.md`
- `docs/vulnerability-tracker.md`

## Current Stack

- Rust core engine
- Signed YAML and JSON rules packs
- HTML, PDF, and JSON reporting

## Current Bootstrap Scope

The repository currently ships a Rust workspace with:

- `crates/core-engine`: shared scanning, reporting, and remediation logic
- `crates/cli`: command-line entry point for scanning and hardening configuration profiles

## CLI Usage

Generate a sample configuration:

```bash
cargo run -p clawguard -- sample-config --output example.conf
```

Generate a sample ruleset:

```bash
cargo run -p clawguard -- sample-rules --output rules/default.rules
```

Scan a configuration:

```bash
cargo run -p clawguard -- scan --config example.conf --format json
```

Scan with localized text output:

```bash
cargo run -p clawguard -- scan --config example.conf --format text --locale zh-CN
```

Scan with a custom ruleset:

```bash
cargo run -p clawguard -- scan --config example.conf --rules rules/default.rules
```

Scan a deployment directory:

```bash
cargo run -p clawguard -- scan-profile --path /path/to/openclaw-profile --format html --output report.html
```

Apply hardening to a new output file:

```bash
cargo run -p clawguard -- harden --config example.conf --output hardened.conf
```

## Repository Conventions

- Repository content is English-first.
- Reports default to English and can support Simplified Chinese as an optional locale.
- Code comments must be written in English.
- Security-impacting changes should update both the tracker and the relevant docs.
