# ClawGuard

<p align="center">
  <strong>Harden OpenClaw before the internet finds it.</strong>
</p>

<p align="center">
  ClawGuard, also known as <strong>小龙虾卫士</strong>, is a CLI-first security scanner and one-click hardening tool for OpenClaw deployments.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-CLI%20Security%20Tool-000000?style=for-the-badge&logo=rust" alt="Rust CLI Security Tool" />
  <img src="https://img.shields.io/badge/OpenClaw-Security%20Audit-c2410c?style=for-the-badge" alt="OpenClaw Security Audit" />
  <img src="https://img.shields.io/badge/Reports-JSON%20%7C%20HTML%20%7C%20Text-0f766e?style=for-the-badge" alt="Reports" />
  <img src="https://img.shields.io/badge/Locale-en%20%7C%20zh--CN-1d4ed8?style=for-the-badge" alt="Locale support" />
</p>

---

## Table of Contents

- [Why ClawGuard](#why-clawguard)
- [What It Does](#what-it-does)
- [Current Status](#current-status)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Example Use Cases](#example-use-cases)
- [Project Layout](#project-layout)
- [Documentation](#documentation)
- [Roadmap](#roadmap)
- [Development](#development)
- [Can Users Install It With npm Today?](#can-users-install-it-with-npm-today)
- [Visual Direction](#visual-direction)
- [Contributing](#contributing)
- [Repository](#repository)
- [License](#license)

## Why ClawGuard

OpenClaw deployments are being exposed with the same failure patterns again and again:

- public control ports
- weak or missing authentication
- broken approval and permission chains
- secrets left in `.env` files and logs
- suspicious skills and untrusted install sources

ClawGuard exists to turn that into a practical operator workflow:

1. inspect the deployment
2. explain the risk
3. harden what can be fixed safely
4. export a report you can actually share

## What It Does

### 🔎 Audit

- Scan a single OpenClaw config file
- Scan a deployment directory
- Detect exposure, auth, permission, secrets, and supply-chain findings
- Load custom rulesets

### 🛠 Harden

- Back up configs before changes
- Restrict risky bind addresses
- Rotate weak tokens
- Disable dangerous debug and status exposure
- Remove suspicious skills from the active config path

### 🧾 Report

- Export `json`, `html`, and `text`
- Use English or Simplified Chinese output
- Auto-detect system language when `--locale` is not provided
- Generate operator-readable findings with evidence and remediation

## Current Status

ClawGuard is currently a working Rust CLI MVP.

Implemented today:

- CLI scanning
- CLI hardening
- CLI uninstall command
- deployment profile scanning
- localized report output
- system locale auto-detection
- signed rules-pack generation, import, activation, and rollback
- ASCII startup banner
- packaging script and installation guide
- test coverage for core and CLI flows

Still in progress:

- signed release artifacts
- online rules-pack update checks
- published install channels
- npm wrapper distribution

## Quick Start

Build the CLI:

```bash
cargo build --release -p clawguard
```

Run help:

```bash
cargo run -p clawguard -- help
```

Generate a sample config:

```bash
cargo run -p clawguard -- sample-config --output example.conf
```

Generate a signing keypair for rules-pack management:

```bash
cargo run -p clawguard -- generate-signing-keypair --output-dir .keys
```

Sign a rules pack from the default rules or a custom rules file:

```bash
cargo run -p clawguard -- sign-rules-pack --output rules-pack.json --version 0.1.0 --private-key .keys/clawguard-rules.private.key
```

Import and activate the signed rules pack:

```bash
cargo run -p clawguard -- import-rules-pack --pack rules-pack.json --public-key .keys/clawguard-rules.public.key --activate
```

Scan a config:

```bash
cargo run -p clawguard -- scan --config example.conf --format json
```

Scan with localized text output:

```bash
cargo run -p clawguard -- scan --config example.conf --format text --locale zh-CN
```

Scan a deployment directory:

```bash
cargo run -p clawguard -- scan-profile --path /path/to/openclaw-profile --format html --output report.html
```

Apply hardening:

```bash
cargo run -p clawguard -- harden --config example.conf --output hardened.conf
```

Uninstall from a target install directory:

```bash
cargo run -p clawguard -- uninstall --install-dir "$HOME/.local/bin"
```

## Installation

Install from the local source tree:

```bash
cargo install --path crates/cli
```

Remove the Cargo-installed binary:

```bash
cargo uninstall clawguard
```

Install with `curl` after release archives are published:

```bash
curl -fsSL https://raw.githubusercontent.com/legeling/ClawGuard/main/scripts/install-clawguard.sh | bash
```

Uninstall the `curl` install:

```bash
curl -fsSL https://raw.githubusercontent.com/legeling/ClawGuard/main/scripts/uninstall-clawguard.sh | bash
```

Install with npm or run with npx after the npm wrapper is published:

```bash
npx clawguard --help
```

```bash
npm install -g clawguard
clawguard --help
```

Detailed installation and packaging notes:

- [CLI Installation](docs/cli-installation.md)

## Example Use Cases

- Audit a publicly reachable OpenClaw host before putting it behind a reverse proxy
- Check whether a home-lab or VPS deployment leaked secrets into `.env` or logs
- Validate installed skills against suspicious pattern rules
- Produce an HTML report for review, documentation, or incident follow-up

## Project Layout

```text
crates/core-engine   Shared scanning, reporting, and remediation logic
crates/cli           Command-line entry point
rules/               Ruleset content
reports/             Report-related assets and conventions
docs/                Product, design, security, and operations docs
scripts/             Packaging and project automation
```

## Documentation

- [Security Insights](docs/openclaw-security-insights.md)
- [Product Requirements](docs/openclaw-guard-requirements.md)
- [Solution Architecture](docs/openclaw-guard-architecture.md)
- [CLI Installation](docs/cli-installation.md)
- [CI/CD](docs/cicd.md)
- [Vulnerability Tracker](docs/vulnerability-tracker.md)

## Roadmap

- Improve artifact signing and verification
- Add online rules-pack update checks
- Expand detector coverage for more real-world OpenClaw deployment patterns
- Add published installation channels
- Add npm wrapper distribution if a Node-based install path is still needed

## Development

Build:

```bash
cargo build --workspace
```

Test:

```bash
cargo test --workspace
```

Lint:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Package a release archive:

```bash
bash scripts/package-release.sh
```

## Can Users Install It With npm Today?

Not yet.

ClawGuard is currently distributed as a Rust CLI, not an npm package.

What exists now:

- Rust workspace
- CLI binary
- local packaging script
- curl installer script
- npm wrapper package scaffold

What does not exist yet:

- published npm package
- `npx clawguard` install path
- published GitHub release archives for the download installers

## FAQ

### Does ClawGuard auto-detect the user locale?

Yes.

If `--locale` is not provided, ClawGuard checks `LC_ALL`, `LC_MESSAGES`, and `LANG`.
If the environment looks Chinese, it switches to `zh-CN`. Otherwise it defaults to English.

### Does ClawGuard have an uninstall flow?

Yes, for the current CLI installation model.

- CLI command: `clawguard uninstall --install-dir <path>`
- Shell script: `scripts/uninstall-clawguard.sh`

### Does ClawGuard show an ASCII banner on startup?

Yes.

The help screen now includes an ASCII banner and a localized tagline for ClawGuard / 小龙虾卫士.

## Visual Direction

ClawGuard should look like a precise, sharp, modern security product, not a generic hacker terminal brand.

## Contributing

Contributions are welcome, but the repository follows a few strict rules:

- repository-facing content stays in English
- code comments stay in English
- security-impacting changes should update the tracker and related docs
- new behavior should land with tests and verification

## Repository

- GitHub: `git@github.com:legeling/ClawGuard.git`

## License

License file not added yet.
