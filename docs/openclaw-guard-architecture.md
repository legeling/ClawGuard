# Clawguard Solution Architecture

Version: v0.1  
Updated: 2026-03-12

## 1. Architecture Goals

The architecture must satisfy five constraints:

1. Cross-platform delivery on macOS, Windows, and Linux
2. Shared detection and remediation logic across CLI commands
3. Safe, auditable remediation with backup and rollback
4. Fast response to new OpenClaw disclosures through ruleset updates
5. English-first product and engineering workflow with optional Chinese localization

## 2. Recommended Stack

- Core language: Rust
- CLI shell: Rust binary
- Core serialization: `serde` and `serde_json`
- Rules parsing: signed JSON packs plus plain-text local overrides
- Reporting: HTML templates plus PDF export
- Localization: locale-aware report templates and documentation bundles

## 3. System Topology

Clawguard should be organized into one shared engine and one shell:

1. Shared Core
   - Asset discovery
   - Detection orchestration
   - Rules evaluation
   - Risk scoring
   - Remediation planning
   - Remediation execution
   - Backup and rollback
   - Report generation

2. CLI Shell
   - Non-interactive automation
   - CI and scripting support
   - JSON-first output mode

## 4. Logical Modules

### 4.1 `core-engine`

Responsibilities:

- Scan orchestration
- Rule loading
- Finding normalization
- Risk scoring
- Task cancellation and progress reporting

### 4.2 `detectors`

Suggested detector families:

- Local installation detector
- Process and port detector
- Reverse-proxy detector
- Version detector
- Configuration parser
- Permission-chain detector
- Secrets detector
- Skills and provenance detector
- Exposure reachability detector

### 4.3 `remediators`

Responsibilities:

- Build remediation plans from findings
- Validate whether remediation is safe to apply
- Write backups before changes
- Apply changes
- Roll back on failure
- Emit structured remediation records

### 4.4 `rules-runtime`

Responsibilities:

- Load signed rules packs
- Validate version compatibility and Ed25519 signatures
- Evaluate findings against advisories, baselines, and IOC rules
- Support local custom rules layered on top of vendor rules
- Persist imported packs in a local rules store
- Track the active rules version and rollback history

### 4.5 `reporting`

Responsibilities:

- Generate structured report models
- Render interactive UI views
- Export HTML, PDF, and JSON
- Render localized report strings

### 4.6 `platform-adapters`

Responsibilities:

- Abstract filesystem and permission differences
- Support platform-specific process enumeration
- Support platform-specific firewall or networking checks
- Handle platform-specific packaging specifics

## 5. End-to-End Flow

### 5.1 Scan Flow

1. User selects local scan or target input
2. Core engine resolves the scan context
3. Detectors run in parallel where safe
4. Raw observations are normalized into findings
5. Rules runtime enriches findings with severity, explanation, and remediation metadata
6. Risk scoring computes host and scan-level posture
7. Reporting builds the interactive and exportable report model

### 5.2 Remediation Flow

1. User selects recommended fixes
2. Core engine composes a remediation plan
3. Remediators validate prerequisites
4. System creates timestamped backups
5. System applies changes in a controlled sequence
6. Post-remediation verification reruns the affected checks
7. Report captures before and after status
8. Rollback remains available if verification fails or the user cancels

### 5.3 Rules Update Flow

1. Client fetches or imports a signed rules pack
2. Signature and version compatibility are verified before import
3. Rules are stored under a versioned local rules-store path
4. The active ruleset pointer is updated explicitly by activation
5. Previous ruleset remains available for rollback

## 6. Data Model Overview

Core entities should include:

- `Asset`
- `ScanTarget`
- `Observation`
- `Finding`
- `FindingEvidence`
- `Rule`
- `RemediationPlan`
- `RemediationAction`
- `BackupRecord`
- `RollbackRecord`
- `Report`
- `RulesetVersion`
- `LocalizationBundle`

## 7. Findings and Rules Design

Each finding should be normalized to a common schema:

- `id`
- `category`
- `severity`
- `title`
- `summary`
- `evidence`
- `affected_component`
- `permission_impact`
- `exposure_impact`
- `recommended_fix`
- `auto_fix_supported`
- `references`

Rules should be designed to support:

- Version windows
- Config predicates
- Environment predicates
- Platform predicates
- Detection evidence requirements
- Remediation metadata

This keeps the product extensible when new CVEs follow known mechanisms.

## 8. Permission-Chain Design

Because the user explicitly called out permission vulnerabilities, this area needs a dedicated model.

The permission-chain subsystem should evaluate:

- Who can trigger an action
- Which policy decides whether it is allowed
- What the user sees before approval
- What command or action is actually executed
- Whether normalization differs between approval and execution stages
- Whether webhooks or external triggers can bypass intended approval

This subsystem should produce both:

- A technical finding for engineers
- A plain-language explanation for operators

## 9. Internationalization Design

The project should be English-first:

- Product copy authored in English
- Code comments in English only
- Engineering docs in English only
- Chinese provided as an optional report locale and optional translated docs set

Implementation expectations:

- Report templates rendered from localized message bundles
- Missing translations fall back to English

## 10. Documentation Management Design

Documentation should be treated as a product subsystem, not an afterthought.

Required document classes:

- Security insights
- Product requirements
- Solution architecture
- System design
- Vulnerability tracker
- ADRs for major decisions
- Release notes

Update rules:

- New vulnerability disclosure updates the tracker first
- If product scope changes, update requirements
- If implementation approach changes, update design
- If a major technical choice changes, add an ADR

## 11. UX Direction

The product should feel precise and high-trust rather than noisy or "hacker themed".

Recommended UX principles:

- Lead with posture summary, not raw logs
- Separate "critical now" from "important later"
- Treat remediation as a controlled workflow, not a blind button
- Make evidence visible but collapsible
- Make report exports presentation-ready

## 12. Delivery Recommendation

Phase 2 recommendation:

1. Build the Rust core as the source of truth
2. Expose the engine through a stable CLI
3. Treat rules packs as versioned signed content
4. Implement localization from day one
5. Design the report system as a first-class output, not a post-processing layer
