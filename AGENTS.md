# AGENTS.md

## Project Identity

- Project name: `Clawguard`
- Mission: build a cross-platform OpenClaw security audit and one-click hardening tool
- Canonical repository language: English

## Language Policy

- Keep all repository-facing artifacts in English.
- Write code comments in English only.
- Write product, design, architecture, tracker, and operational docs in English.
- Use English for config keys, identifiers, rule names, report schemas, and file names.
- UI must default to English and support Simplified Chinese as an optional locale.
- When a Chinese translation diverges from the English source, the English source wins until reconciled.

## Documentation Requirements

The following documents are canonical and must stay current:

- `docs/openclaw-security-insights.md`
- `docs/openclaw-guard-requirements.md`
- `docs/openclaw-guard-architecture.md`
- `docs/documentation-management.md`
- `docs/vulnerability-tracker.md`

When product scope or security posture changes:

1. Update the vulnerability tracker first if the change is security-driven.
2. Update requirements if scope, user-facing behavior, or acceptance criteria change.
3. Update architecture or later system design docs if implementation structure changes.
4. Record major technical decisions as ADRs once the ADR directory exists.

## Engineering Principles

- Prefer a shared Rust core for scanning, rule evaluation, remediation, rollback, and reporting.
- Keep the project CLI-first. Do not introduce a frontend shell unless explicitly requested.
- Treat rule packs as versioned and signature-verified content.
- Design remediation to be safe-by-default, auditable, and reversible.
- Favor modular detectors, remediators, report generators, and platform adapters.
- Model security issues by mechanism, not only by individual CVE.

## Security Principles

- Least privilege by default
- Backup before mutation
- Rollback on failed remediation
- No silent auto-fix for service-breaking changes
- Evidence required for every finding
- Signature verification for updateable content
- Secrets redaction in logs and reports unless explicitly needed for local proof

## Delivery Rules

- Start with docs when requirements or architecture are still moving.
- Keep repository content consistent with the English-first policy from day one.
- Do not introduce localized source strings directly into code; use i18n keys.
- Keep reports shareable and operator-readable, not just technically accurate.
- Prefer additive changes and avoid destructive cleanup unless explicitly requested.

## File and Structure Conventions

- Put product and design docs in `docs/`.
- Put Codex project configuration in `.codex/`.
- Keep future ADRs under `docs/adr/`.
- Keep future localized docs under a dedicated locale path such as `docs/i18n/zh-CN/`.

## Definition of Done

A meaningful feature or design change is not complete unless:

1. The relevant docs are updated.
2. Security impact is reflected in the vulnerability tracker when applicable.
3. Language policy is respected.
4. Acceptance criteria or success conditions are stated.
5. Any new remediation behavior defines backup and rollback expectations.
