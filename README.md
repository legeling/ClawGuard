# OpenClaw Guard

OpenClaw Guard is a cross-platform security audit and one-click hardening tool for OpenClaw deployments and adjacent ecosystem components.

## Current Status

This repository is in the design and project bootstrap stage.

Canonical project documents live in `docs/`:

- `docs/openclaw-security-insights.md`
- `docs/openclaw-guard-requirements.md`
- `docs/openclaw-guard-architecture.md`
- `docs/documentation-management.md`
- `docs/vulnerability-tracker.md`

## Planned Stack

- Rust core engine
- Tauri 2 desktop shell
- React + TypeScript frontend
- Signed YAML and JSON rules packs
- HTML, PDF, and JSON reporting

## Current Bootstrap Scope

The repository currently ships a Rust workspace with:

- `crates/core-engine`: shared scanning, reporting, and remediation logic
- `crates/cli`: command-line entry point for scanning and hardening configuration profiles

## CLI Usage

Generate a sample configuration:

```bash
cargo run -p openclaw-guard -- sample-config --output example.conf
```

Generate a sample ruleset:

```bash
cargo run -p openclaw-guard -- sample-rules --output rules/default.rules
```

Scan a configuration:

```bash
cargo run -p openclaw-guard -- scan --config example.conf --format json
```

Scan with a custom ruleset:

```bash
cargo run -p openclaw-guard -- scan --config example.conf --rules rules/default.rules
```

Scan a deployment directory:

```bash
cargo run -p openclaw-guard -- scan-profile --path /path/to/openclaw-profile --format html --output report.html
```

Apply hardening to a new output file:

```bash
cargo run -p openclaw-guard -- harden --config example.conf --output hardened.conf
```

## Repository Conventions

- Repository content is English-first.
- UI defaults to English and supports Simplified Chinese as an optional locale.
- Code comments must be written in English.
- Security-impacting changes should update both the tracker and the relevant docs.
