# Clawguard Product Requirements

Version: v0.2  
Updated: 2026-03-12

## 1. Product Positioning

Clawguard is a CLI-first security audit and one-click hardening tool for the OpenClaw ecosystem. It is designed to deliver the following workflow for command-line users and automation pipelines:

1. Discover assets and exposure points
2. Identify known vulnerabilities and unsafe configurations
3. Audit authentication, authorization, and approval-chain risks
4. Apply guided remediation with rollback support
5. Generate professional, shareable security reports

## 2. Problem Statement

Recent public reporting around OpenClaw shows that real-world risk clusters around:

- Public IP exposure and open high-risk ports
- Unsafe bind addresses and overexposed reverse proxies
- Missing or weak authentication
- Permissions and approval flows that diverge from actual execution
- Supply-chain abuse through skills, plugins, installers, and shell scripts
- Path traversal, arbitrary file access, and secrets leakage

The market need is not a pure scanner. It is a practical "inspect, remediate, update, and report" product.

## 3. Target Users

### 3.1 Individual Operators

Need a fast way to determine whether their OpenClaw deployment is publicly exposed, misconfigured, or immediately exploitable, and then fix it with minimal friction.

### 3.2 Small-Team Administrators

Need repeatable checks across multiple hosts, a common security baseline, and reports suitable for leadership or security review.

### 3.3 Security Engineers

Need extensible rules, evidence-backed findings, rollback behavior, and traceable remediation history.

## 4. Product Goals

### 4.1 Core Goals

1. Reduce time-to-detect for common OpenClaw exposure issues to under 5 minutes
2. Turn common high-risk findings into one-click or one-flow remediation
3. Produce reports that can be used directly in audit and review workflows
4. Allow new CVEs and baseline rules to ship without requiring a full client release

### 4.2 Non-Goals

1. No offensive exploitation framework
2. No internet-wide scanning platform in v1
3. No deep SIEM/SOC integration in v1
4. No broad AI-agent security platform in v1 beyond the OpenClaw ecosystem

## 5. Scope

### 5.1 Must-Have Capabilities

1. Local OpenClaw installation discovery
2. Remote target checks for IPs, domains, and URLs
3. Port and service identification for OpenClaw-native and reverse-proxy deployments
4. Version and advisory matching
5. Configuration auditing
6. Permission-management and approval-chain auditing
7. Skill, plugin, and integration provenance auditing
8. Secrets and sensitive-file exposure checks
9. Guided remediation and rollback
10. Rich report rendering and export
11. Ruleset updates and extensibility
12. English-first reporting and documentation with optional Chinese localization

### 5.2 Should-Have Capabilities

1. Batch asset scanning
2. Scheduled inspections
3. IOC checks
4. Baseline policy templates
5. CLI automation integration

## 6. Functional Requirements

### FR-01 Asset Discovery

The system must:

- Detect whether OpenClaw is installed locally
- Detect deployment mode: standalone binary, source install, container, or reverse-proxy publish
- Detect listening addresses, ports, and process metadata
- Determine whether the target is reachable from the public internet

Supported inputs:

- Local auto-discovery
- Manual IP, domain, or URL input
- Imported asset list

### FR-02 Version and Advisory Detection

The system must:

- Identify OpenClaw version, component versions, and build source when possible
- Match against the local advisory ruleset
- Provide severity, impact, evidence, and remediation guidance

The ruleset must support:

- CVEs
- GitHub Advisories
- Unsafe-configuration rules
- IOC rules

### FR-03 Exposure and Network Baseline Checks

The system must check:

- Whether services are bound to `0.0.0.0` or public interfaces
- Whether high-risk ports are exposed
- Whether the control plane is internet-facing behind a reverse proxy
- Whether TLS is enabled
- Whether source IP restrictions exist
- Whether health, status, or debug endpoints are exposed without auth

### FR-04 Authentication, Authorization, and Permission Management

The system must check:

- Whether authentication is enabled
- Whether default, weak, or empty tokens are used
- Whether webhook signature validation exists and can fail open
- Whether deep links, approval screens, and allowlists match actual execution semantics
- Whether permissions are too broad
- Whether high-risk auto-approval policies exist
- Whether administrator-only actions are clearly separated from lower-privilege actions

This module must render a dedicated "Permission Chain Risk" section in reports.

### FR-05 File and Secrets Inspection

The system must check:

- Whether `.env`, configuration files, logs, caches, and backups contain sensitive data
- Whether sensitive files are world-readable
- Whether path traversal risk exists in current configuration or integration paths
- Whether skill or plugin status output can disclose secrets

### FR-06 Skills and Supply-Chain Inspection

The system must check:

- Installed skill or plugin origin
- Whether the origin is trusted
- Whether install scripts, package signatures, or download locations are suspicious
- Whether installed artifacts match known malicious-skill rules
- Whether binary, package, or script hashes are unexpected

### FR-07 Guided Remediation

The system must support one-click or guided remediation actions for:

- Backing up current configuration
- Restricting bind addresses to `127.0.0.1` or approved internal addresses
- Closing public exposure or generating firewall guidance
- Creating or rotating strong authentication tokens
- Applying reverse-proxy authentication, TLS, and source restrictions
- Disabling high-risk skills or integrations
- Turning off dangerous debug interfaces

Remediation requirements:

- Every remediation must produce a backup
- Every remediation must record a change summary
- Every failed remediation must support rollback
- Service-disrupting changes must require explicit confirmation

### FR-08 Reporting

The system must generate reports that include:

- Overall risk score
- Finding list
- Evidence for each finding
- Impact summary
- Recommended or applied remediation
- Change log of remediation actions
- Before/after comparison
- Remaining manual actions

Supported outputs:

- Terminal summary
- HTML export
- PDF export
- JSON export

### FR-09 Ruleset Updates and Extensibility

The system must support:

- Ruleset updates independent of client releases
- Online update checks
- Offline rules-pack import
- Rollback to an earlier ruleset version
- Custom rules
- Ruleset signature verification

Extensibility goals:

- New CVEs should not require a client release
- New detectors, remediators, and report templates should be pluggable

### FR-10 Cross-Platform Support

The first release must support:

- macOS
- Windows
- Linux

Requirements:

- Shared scanning logic across platforms
- Platform differences isolated in adapters
- CLI entry point with scriptable output

### FR-11 Internationalization

The system must support:

- English as the default command and report language
- Simplified Chinese as an optional report locale
- Locale-aware help, interactive prompts, and report rendering
- Explicit locale selection through CLI flags and environment-based auto-detection

The project must also enforce:

- English-only code comments
- English as the source-of-truth for product and engineering docs
- Localized docs treated as derived artifacts, not canonical sources

### FR-12 Documentation and Traceability

The project must maintain:

- A requirements document
- A design document
- A vulnerability tracker
- A decisions log for major architectural choices
- Links between findings, rules, remediation logic, and documentation updates

## 7. Non-Functional Requirements

### NFR-01 Security

- All remediations must default to least privilege
- All backups must include timestamps and integrity metadata
- All app updates and ruleset packs must be signature-verified
- Logs must redact sensitive fields by default

### NFR-02 Performance

- A standard single-host scan should complete in under 5 minutes
- Report generation should complete in under 30 seconds
- Most checks should run concurrently when safe

### NFR-03 Usability

- Findings and remediation results must be understandable to non-specialists
- The UI should surface recommended actions by default
- High-risk issues must be visually prioritized

### NFR-04 Maintainability

- Detectors, remediators, and report templates must be modular
- The ruleset must be versioned
- Platform adapters must stay isolated from core logic

### NFR-05 Localization Quality

- English strings are authored first
- Translation coverage must be trackable
- Missing translations must gracefully fall back to English

## 8. CLI and Reporting Expectations

The product should not feel like a traditional log-dump security tool. It should:

1. Show the answer immediately
   - Is the target publicly exposed?
   - Are there critical vulnerabilities?
   - Should remediation happen now?

2. Keep finding cards readable
   - What is wrong
   - What evidence supports it
   - Why it matters
   - What the fix will change

3. Produce externally shareable reports
   - Suitable for managers
   - Suitable for clients
   - Suitable for security review

4. Keep remediation deliberate
   - Show change summary first
   - Execute remediation second
   - Show before/after diff last

## 9. Recommended Technology Stack

Recommended stack:

- Core engine: Rust
- CLI shell: Rust binary
- Reporting: local HTML rendering with PDF export
- Rules engine: Rust plus YAML/JSON rules packs
- Updates: signed rules packs and signed app release metadata
- Localization: locale-aware report and documentation bundles

Rationale:

- Rust fits cross-platform binaries, networking, file operations, and security-sensitive logic
- A CLI-first shell simplifies distribution and operational automation
- Independent ruleset updates improve response time to newly disclosed issues

## 10. Initial Delivery Milestones

### Milestone 1: MVP

- Local scan
- Remote target check
- Public exposure check
- Version and advisory matching
- Configuration baseline checks
- 3 to 5 one-click remediations
- HTML report export

### Milestone 2: Permission and Supply Chain

- Webhook, approval-chain, and allowlist risk checks
- Skill provenance and malicious-skill matching
- Suspicious installer-source checks
- PDF and JSON reports
- English and Chinese UI localization

### Milestone 3: Extensibility and Team Workflows

- Online rules updates
- Offline rules packs
- Batch asset scans
- Scheduled inspection
- Documentation and tracker automation

## 11. Acceptance Criteria

At minimum, the release must:

1. Start on macOS, Windows, and Linux
2. Detect common OpenClaw deployment modes on a local host
3. Identify at least one real high-risk configuration and one real known vulnerability
4. Successfully apply and roll back at least one remediation
5. Generate a readable, shareable report
6. Upgrade the ruleset independently of the client
7. Render reports in English by default and support Chinese as an alternate locale where implemented
