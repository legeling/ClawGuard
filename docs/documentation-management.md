# Documentation Management

Version: v0.1  
Updated: 2026-03-12

## 1. Purpose

This document defines how OpenClaw Guard documentation is organized, maintained, and localized.

## 2. Language Policy

- English is the source of truth for all product and engineering documents.
- Code comments, code-facing identifiers, and developer-facing operational notes must be written in English.
- Simplified Chinese may be provided as a localized output for end users, but translated documents are derived artifacts.
- When English and Chinese content diverge, the English version wins until reconciled.

## 3. Required Document Set

The project must maintain the following canonical documents:

1. Security insights
2. Product requirements
3. Solution architecture
4. System design
5. Vulnerability tracker
6. ADRs
7. Release notes

## 4. Ownership Model

- Product requirements: product owner or feature lead
- Architecture and system design: engineering lead
- Vulnerability tracker: security owner
- ADRs: author of the decision, reviewed by engineering lead
- Release notes: release owner

## 5. Update Triggers

Update the documentation when any of the following happens:

- A new OpenClaw vulnerability or advisory is disclosed
- A detector or remediator is added or removed
- A product requirement changes
- A platform support decision changes
- Localization scope changes
- Ruleset schema changes

## 6. Traceability Rules

Every new vulnerability entry should link to:

- Source advisory
- Affected product scope
- Detection rule or future rule
- Remediation logic or planned remediation
- Report impact

Every major design change should link to:

- Requirement ID
- ADR entry, if applicable
- Implementation milestone

## 7. Localization Handling

- Canonical docs remain in English
- Localized docs should live in a separate localization path in the future
- Translation status should be trackable by document and version
- Partial translations must clearly indicate their source version

## 8. Review Cadence

- Vulnerability tracker: update on disclosure
- Requirements: update when scope changes
- Architecture and system design: update before implementation shifts
- Full docs review: once per milestone
