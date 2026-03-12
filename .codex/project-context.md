# OpenClaw Guard Project Context

## Overview

OpenClaw Guard is a cross-platform security audit and one-click hardening product for OpenClaw deployments and related ecosystem components.

The product must detect:

- Public exposure
- Unsafe configuration
- Known advisories and vulnerability windows
- Permission-chain weaknesses
- Secrets leakage
- Supply-chain risk from skills, plugins, and installers

The product must also provide:

- Guided remediation
- Backup and rollback
- Evidence-driven reporting
- Updatable signed rulesets

## Collaboration Rules

- Repository artifacts must remain in English.
- Code comments must remain in English.
- UI copy is authored in English first and localized second.

## Canonical Documents

- `docs/openclaw-security-insights.md`
- `docs/openclaw-guard-requirements.md`
- `docs/openclaw-guard-architecture.md`
- `docs/documentation-management.md`
- `docs/vulnerability-tracker.md`

## Current Product Direction

- Build a shared Rust core.
- Expose the core through Tauri desktop and CLI shells.
- Keep detectors, remediators, reporting, and platform adapters modular.
- Treat documentation and vulnerability tracking as first-class project assets.

## Change Discipline

Before major implementation starts:

1. Requirements should be stable enough for the next milestone.
2. Architecture should define module boundaries and data flow.
3. Vulnerability tracker should reflect current public issues relevant to the product.
4. Language and localization policy should be enforced consistently.
