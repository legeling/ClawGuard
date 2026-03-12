# OpenClaw Security Insights

Updated: 2026-03-12

## 1. Executive Summary

The current OpenClaw risk profile is not dominated by a single CVE. It is shaped by a repeatable set of compromise paths:

1. Publicly exposed control and gateway services
2. Broken authentication, authorization, and approval boundaries
3. Insufficient input validation across `gatewayUrl`, WebSocket, SSH, and browser execution chains
4. Local file read, path traversal, and secrets exposure
5. Supply-chain abuse through skill marketplaces, fake installers, and third-party scripts

For a security audit and one-click hardening product, this means the product cannot stop at port scanning or version matching. It must also cover:

- Exposure discovery
- Configuration and permission baseline auditing
- Version and advisory matching
- Skill, plugin, and installer provenance checks
- Guided remediation and rollback
- Updatable rules and evidence-driven reporting

## 2. Current Threat Landscape

### 2.1 Public Exposure Is the Primary Risk Surface

On 2026-01-31, Censys reported `21,639` publicly exposed OpenClaw instances and highlighted `TCP/18789` as a recurring exposure point. That strongly suggests many operators expose the control plane or gateway directly to the internet instead of using SSH tunneling, private networking, or zero-trust access.

Implications for the product:

- Check bind addresses, open ports, reverse-proxy exposure, and source restrictions first
- Distinguish between "locally reachable" and "publicly reachable" risk levels
- Ship exposure-closing remediation, not just detection

### 2.2 Early 2026 Vulnerabilities Show a Clear Pattern

Between January and February 2026, OpenClaw disclosed multiple high-impact issues. Representative examples include:

1. `CVE-2026-25157`  
   SSH argument handling led to command injection

2. `CVE-2026-26322`  
   Gateway `gatewayUrl` handling enabled SSRF

3. `CVE-2026-26324`  
   SSRF protections were bypassed using IPv6-mapped addresses

4. `CVE-2026-26321`  
   Media path handling allowed arbitrary local file exfiltration

5. `CVE-2026-26329`  
   Browser upload handling enabled path traversal

6. `CVE-2026-26326`  
   `skills.status` leaked secrets from configuration

7. `CVE-2026-27009`  
   Control UI was vulnerable to XSS

These issues indicate a recurring failure mode: external input reaches powerful execution or data access surfaces with weak normalization and weak policy enforcement.

### 2.3 Permissions and Access Control Are Core Failure Domains

Permission management is not secondary. It is part of the main attack surface:

1. `CVE-2026-26319`  
   Telnyx webhooks failed open when `TELNYX_PUBLIC_KEY` was missing, allowing spoofed events

2. `CVE-2026-26320`  
   The macOS `openclaw://` deep-link preview could differ from the action actually executed

3. `CVE-2026-26325`  
   Command approval allowlists could be bypassed through normalization inconsistencies such as whitespace handling

What this means:

- Approval checks must use the same normalization logic as execution
- "Preview then approve" workflows must guarantee display and execution parity
- Webhooks, deep links, external triggers, and plugins must never be trusted by default

### 2.4 Supply-Chain Abuse Has Already Moved Beyond Theory

In late January and early February 2026, ClawHub hosted malicious skills disguised as useful automation or utility extensions. Public reporting showed that users could install hostile behavior while believing they were adding normal functionality.

Around 2026-02-09, Huntress also reported fake "OpenClaw Windows" GitHub installers delivered through search and recommendation poisoning, ultimately dropping credential-theft malware.

Implications for the product:

- Skills, plugins, and installer provenance must be first-class scan targets
- The product should detect risky install scripts, suspicious origins, unsigned artifacts, and hash mismatches
- The report should speak to trust and authenticity, not just runtime configuration

### 2.5 Adjacent Leaks Indicate Weak Secrets and Configuration Hygiene

Adjacent ecosystem incidents, including public reporting around Moltbook data exposure, show that operators in this space often lack strong secrets hygiene, exposure control, and least-privilege configuration practices.

Implications for the product:

- Scan `.env`, logs, backups, public directories, and reverse-proxy configs for sensitive data leakage
- Generate least-exposure and least-privilege hardening guidance by default

## 3. Vulnerability Mechanism Model

To keep the product extensible, detection and remediation should be modeled by attack mechanism rather than by hard-coded per-CVE logic.

### 3.1 Exposure Mechanisms

- Services bound to `0.0.0.0` or public interfaces
- High-risk ports directly reachable from the internet
- Reverse proxies without authentication, source filtering, or TLS
- Debug, status, or health endpoints exposed externally

### 3.2 Authentication and Authorization Mechanisms

- Default, weak, or missing tokens
- Missing or fail-open webhook verification
- Approval screens, deep-link flows, or allowlists that do not match actual execution semantics
- Missing role boundaries between operator and administrator actions

### 3.3 Input-to-Execution Mechanisms

- Command injection
- Argument injection
- SSRF via external URLs
- Host command execution through browser, gateway, SSH, or shell bridges

### 3.4 File and Data Exfiltration Mechanisms

- Path traversal
- Arbitrary file read
- Secrets leakage via diagnostic or state endpoints
- Sensitive files left in temp, cache, backup, or log locations

### 3.5 Supply-Chain Mechanisms

- Malicious skills
- Fake installers
- Unsigned or untrusted update packages
- Risky shell snippets copied from third-party pages

## 4. Product Implications for OpenClaw Guard

To cover the actual risk surface, the product needs at least:

1. Asset discovery
   - Detect local installations
   - Detect desktop, source, container, and reverse-proxy deployments
   - Detect public reachability and open exposure points

2. Rule-driven auditing
   - Version and advisory matching
   - Configuration baseline checks
   - Permissions and approval-chain checks
   - Skill and installer provenance checks

3. Safe remediation
   - Configuration backup before changes
   - Bind-address and exposure reduction
   - Authentication, TLS, source restriction, and signature enforcement fixes
   - Ability to disable risky skills and integrations

4. Evidence and reporting
   - Evidence for every finding
   - Before/after remediation comparison
   - Clear status on what was fixed and what still needs human review

5. Rules lifecycle
   - Independently update the ruleset
   - Add CVEs, IOCs, and baseline rules without client releases
   - Support both online updates and offline import

## 5. Design Constraints Derived From the Threat Model

The implementation should favor:

- Strong cross-platform support
- Safe handling of files, networking, processes, and privilege-sensitive operations
- Shared core logic for GUI and CLI
- Signed rule updates and offline packs
- A clean boundary between detectors, remediators, and report generation

Recommended baseline:

- Core engine: Rust
- Desktop shell: Tauri 2
- Frontend: React + TypeScript
- Rules format: YAML or JSON with version and signature metadata
- Reporting: HTML-first with PDF export

## 6. References

1. Censys, 2026-01-31
   https://censys.com/blog/openclaw-in-the-wild-mapping-the-public-exposure-of-a-viral-ai-assistant

2. NVD: `CVE-2026-25157`  
   https://nvd.nist.gov/vuln/detail/CVE-2026-25157

3. NVD: `CVE-2026-26319`  
   https://nvd.nist.gov/vuln/detail/CVE-2026-26319

4. NVD: `CVE-2026-26320`  
   https://nvd.nist.gov/vuln/detail/CVE-2026-26320

5. NVD: `CVE-2026-26321`  
   https://nvd.nist.gov/vuln/detail/CVE-2026-26321

6. NVD: `CVE-2026-26322`  
   https://nvd.nist.gov/vuln/detail/CVE-2026-26322

7. GitHub Advisory Database: `CVE-2026-26324`  
   https://github.com/advisories/GHSA-6f7p-g3wh-4hhw

8. GitHub Advisory Database: `CVE-2026-26325`  
   https://github.com/advisories/GHSA-r3rf-9mx6-h4j3

9. NVD: `CVE-2026-26326`  
   https://nvd.nist.gov/vuln/detail/CVE-2026-26326

10. NVD: `CVE-2026-26329`  
    https://nvd.nist.gov/vuln/detail/CVE-2026-26329

11. NVD: `CVE-2026-27009`  
    https://nvd.nist.gov/vuln/detail/CVE-2026-27009

12. Huntress, 2026-02
    https://www.huntress.com/blog/openclaw-github-ghostsocks-infostealer

13. SecurityWeek, 2026-02
    https://www.securityweek.com/openclaw-security-issues-continue-as-secureclaw-open-source-tool-debuts/

14. SecurityScorecard, 2026-02-09  
    https://securityscorecard.com/blog/beyond-the-hype-moltbots-real-risk-is-exposed-infrastructure-not-ai-superintelligence/
