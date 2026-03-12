use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    fn weight(&self) -> u8 {
        match self {
            Self::Critical => 30,
            Self::High => 20,
            Self::Medium => 10,
            Self::Low => 5,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        };
        write!(f, "{value}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Category {
    Exposure,
    Authentication,
    Permission,
    Secrets,
    SupplyChain,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Exposure => "exposure",
            Self::Authentication => "authentication",
            Self::Permission => "permission",
            Self::Secrets => "secrets",
            Self::SupplyChain => "supply-chain",
        };
        write!(f, "{value}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    En,
    ZhCn,
}

impl Locale {
    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "en" | "en-US" => Some(Self::En),
            "zh-CN" | "zh" => Some(Self::ZhCn),
            _ => None,
        }
    }

    fn lang_attr(&self) -> &'static str {
        match self {
            Self::En => "en",
            Self::ZhCn => "zh-CN",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub id: &'static str,
    pub title: &'static str,
    pub severity: Severity,
    pub category: Category,
    pub summary: String,
    pub evidence: String,
    pub remediation: String,
    pub auto_fix_supported: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanReport {
    pub profile_name: String,
    pub findings: Vec<Finding>,
    pub risk_score: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ruleset {
    pub trusted_origins: Vec<String>,
    pub suspicious_skill_patterns: Vec<String>,
    pub weak_tokens: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardenOutcome {
    pub backup_path: Option<PathBuf>,
    pub output_path: PathBuf,
    pub before_score: u8,
    pub after_score: u8,
    pub applied_actions: Vec<String>,
    pub manual_actions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenClawConfig {
    pub profile_name: String,
    pub bind_address: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub auth_token: Option<String>,
    pub source_allowlist: Vec<String>,
    pub webhook_enabled: bool,
    pub webhook_public_key: Option<String>,
    pub approval_preview_consistent: bool,
    pub command_allowlist_normalized: bool,
    pub debug_enabled: bool,
    pub skills_status_exposes_secrets: bool,
    pub suspicious_skills: Vec<String>,
    pub installer_origin: Option<String>,
}

impl Default for OpenClawConfig {
    fn default() -> Self {
        Self {
            profile_name: "default".to_string(),
            bind_address: "127.0.0.1".to_string(),
            port: 18789,
            tls_enabled: true,
            auth_token: Some(generate_token()),
            source_allowlist: vec!["127.0.0.1/32".to_string()],
            webhook_enabled: false,
            webhook_public_key: None,
            approval_preview_consistent: true,
            command_allowlist_normalized: true,
            debug_enabled: false,
            skills_status_exposes_secrets: false,
            suspicious_skills: Vec::new(),
            installer_origin: Some("official".to_string()),
        }
    }
}

impl OpenClawConfig {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut config = Self::default();

        for (index, raw_line) in input.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid line {}: {line}", index + 1))?;

            let key = key.trim();
            let value = value.trim();

            match key {
                "profile_name" => config.profile_name = value.to_string(),
                "bind_address" => config.bind_address = value.to_string(),
                "port" => {
                    config.port = value
                        .parse::<u16>()
                        .map_err(|_| format!("invalid port on line {}", index + 1))?
                }
                "tls_enabled" => config.tls_enabled = parse_bool(value, index)?,
                "auth_token" => {
                    config.auth_token = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "source_allowlist" => config.source_allowlist = parse_list(value),
                "webhook_enabled" => config.webhook_enabled = parse_bool(value, index)?,
                "webhook_public_key" => {
                    config.webhook_public_key = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "approval_preview_consistent" => {
                    config.approval_preview_consistent = parse_bool(value, index)?
                }
                "command_allowlist_normalized" => {
                    config.command_allowlist_normalized = parse_bool(value, index)?
                }
                "debug_enabled" => config.debug_enabled = parse_bool(value, index)?,
                "skills_status_exposes_secrets" => {
                    config.skills_status_exposes_secrets = parse_bool(value, index)?
                }
                "suspicious_skills" => config.suspicious_skills = parse_list(value),
                "installer_origin" => {
                    config.installer_origin = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                _ => return Err(format!("unknown key '{key}' on line {}", index + 1)),
            }
        }

        Ok(config)
    }

    pub fn serialize(&self) -> String {
        [
            format!("profile_name={}", self.profile_name),
            format!("bind_address={}", self.bind_address),
            format!("port={}", self.port),
            format!("tls_enabled={}", self.tls_enabled),
            format!(
                "auth_token={}",
                self.auth_token.clone().unwrap_or_default()
            ),
            format!("source_allowlist={}", self.source_allowlist.join(",")),
            format!("webhook_enabled={}", self.webhook_enabled),
            format!(
                "webhook_public_key={}",
                self.webhook_public_key.clone().unwrap_or_default()
            ),
            format!(
                "approval_preview_consistent={}",
                self.approval_preview_consistent
            ),
            format!(
                "command_allowlist_normalized={}",
                self.command_allowlist_normalized
            ),
            format!("debug_enabled={}", self.debug_enabled),
            format!(
                "skills_status_exposes_secrets={}",
                self.skills_status_exposes_secrets
            ),
            format!("suspicious_skills={}", self.suspicious_skills.join(",")),
            format!(
                "installer_origin={}",
                self.installer_origin.clone().unwrap_or_default()
            ),
        ]
        .join("\n")
    }
}

impl Default for Ruleset {
    fn default() -> Self {
        Self {
            trusted_origins: vec![
                "official".to_string(),
                "signed-release".to_string(),
                "managed-package".to_string(),
            ],
            suspicious_skill_patterns: vec![
                "wallet".to_string(),
                "crypto".to_string(),
                "exfil".to_string(),
                "shell".to_string(),
            ],
            weak_tokens: vec!["default-token".to_string(), "changeme".to_string()],
        }
    }
}

impl Ruleset {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut ruleset = Self::default();

        for (index, raw_line) in input.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid rules line {}: {line}", index + 1))?;

            match key.trim() {
                "trusted_origins" => ruleset.trusted_origins = parse_list(value.trim()),
                "suspicious_skill_patterns" => {
                    ruleset.suspicious_skill_patterns = parse_list(value.trim())
                }
                "weak_tokens" => ruleset.weak_tokens = parse_list(value.trim()),
                _ => return Err(format!("unknown rules key '{}' on line {}", key.trim(), index + 1)),
            }
        }

        Ok(ruleset)
    }
}

pub fn load_config(path: &Path) -> Result<OpenClawConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    OpenClawConfig::parse(&content)
}

pub fn write_config(path: &Path, config: &OpenClawConfig) -> Result<(), String> {
    fs::write(path, config.serialize())
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

pub fn sample_config() -> String {
    OpenClawConfig::default().serialize()
}

pub fn scan_config(config: &OpenClawConfig) -> ScanReport {
    scan_config_with_rules(config, &Ruleset::default())
}

pub fn load_ruleset(path: &Path) -> Result<Ruleset, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ruleset::parse(&content)
}

pub fn default_ruleset_text() -> String {
    let rules = Ruleset::default();
    [
        format!("trusted_origins={}", rules.trusted_origins.join(",")),
        format!(
            "suspicious_skill_patterns={}",
            rules.suspicious_skill_patterns.join(",")
        ),
        format!("weak_tokens={}", rules.weak_tokens.join(",")),
    ]
    .join("\n")
}

pub fn scan_profile_dir(path: &Path) -> Result<ScanReport, String> {
    scan_profile_with_rules(path, &Ruleset::default())
}

pub fn scan_profile_with_rules(path: &Path, rules: &Ruleset) -> Result<ScanReport, String> {
    let config_path = path.join("openclaw.conf");
    let config = load_config(&config_path)?;
    let mut findings = scan_config_with_rules(&config, rules).findings;

    let env_path = path.join(".env");
    if env_path.exists() {
        let env_content = fs::read_to_string(&env_path)
            .map_err(|error| format!("failed to read {}: {error}", env_path.display()))?;
        if contains_secret_material(&env_content) {
            findings.push(Finding {
                id: "SECR-0002",
                title: "Secrets are stored in a local environment file",
                severity: Severity::High,
                category: Category::Secrets,
                summary: "A deployment-local .env file contains token or key material.".to_string(),
                evidence: format!("secret-like values found in {}", env_path.display()),
                remediation: "Move secrets to a secure store and avoid leaving raw credentials in deployment folders."
                    .to_string(),
                auto_fix_supported: false,
            });
        }
    }

    let logs_dir = path.join("logs");
    if logs_dir.exists() {
        for log_path in collect_log_files(&logs_dir)? {
            let log_content = fs::read_to_string(&log_path)
                .map_err(|error| format!("failed to read {}: {error}", log_path.display()))?;
            if log_contains_secrets(&log_content) {
                findings.push(Finding {
                    id: "SECR-0003",
                    title: "Log files contain credential material",
                    severity: Severity::High,
                    category: Category::Secrets,
                    summary: "One or more log files appear to contain raw tokens, keys, or auth parameters."
                        .to_string(),
                    evidence: format!("secret-like log content found in {}", log_path.display()),
                    remediation: "Rotate exposed credentials, sanitize logs, and add log redaction guards."
                        .to_string(),
                    auto_fix_supported: false,
                });
                break;
            }
        }
    }

    let skills_path = path.join("skills").join("installed.txt");
    if skills_path.exists() {
        let skills = fs::read_to_string(&skills_path)
            .map_err(|error| format!("failed to read {}: {error}", skills_path.display()))?;
        let installed = skills
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        let matches = suspicious_skill_matches(&installed, rules);
        if !matches.is_empty() {
            findings.push(Finding {
                id: "SUP-0003",
                title: "Installed skills match suspicious profile patterns",
                severity: Severity::Critical,
                category: Category::SupplyChain,
                summary: "A local skill manifest contains entries that match suspicious ruleset patterns."
                    .to_string(),
                evidence: format!(
                    "skills manifest {} matched {}",
                    skills_path.display(),
                    matches.join(",")
                ),
                remediation: "Disable the matched skills and verify their package source and signing chain."
                    .to_string(),
                auto_fix_supported: false,
            });
        }
    }

    Ok(build_report(config.profile_name, findings))
}

pub fn scan_config_with_rules(config: &OpenClawConfig, rules: &Ruleset) -> ScanReport {
    let mut findings = Vec::new();
    let public_bind = is_public_bind(&config.bind_address);

    if public_bind {
        findings.push(Finding {
            id: "EXP-0001",
            title: "Control plane is bound to a public interface",
            severity: Severity::Critical,
            category: Category::Exposure,
            summary: format!(
                "The service listens on {}, which makes the control plane reachable beyond the local host.",
                config.bind_address
            ),
            evidence: format!(
                "bind_address={} and port={}",
                config.bind_address, config.port
            ),
            remediation: "Restrict the bind address to 127.0.0.1 or an approved internal interface."
                .to_string(),
            auto_fix_supported: true,
        });
    }

    if public_bind && config.port == 18789 {
        findings.push(Finding {
            id: "EXP-0002",
            title: "Default OpenClaw control port is publicly exposed",
            severity: Severity::Critical,
            category: Category::Exposure,
            summary: "The default OpenClaw control port is exposed together with a public bind address."
                .to_string(),
            evidence: "port=18789".to_string(),
            remediation: "Keep the control port private or front it with a locked-down reverse proxy."
                .to_string(),
            auto_fix_supported: false,
        });
    }

    if token_strength(config.auth_token.as_deref(), rules) < 2 {
        findings.push(Finding {
            id: "AUTH-0001",
            title: "Authentication token is missing or weak",
            severity: Severity::Critical,
            category: Category::Authentication,
            summary: "The deployment does not have a strong authentication token.".to_string(),
            evidence: format!(
                "auth_token_strength={}",
                token_strength(config.auth_token.as_deref(), rules)
            ),
            remediation: "Generate a token with at least 24 characters and rotate any default value."
                .to_string(),
            auto_fix_supported: true,
        });
    }

    if public_bind && !config.tls_enabled {
        findings.push(Finding {
            id: "AUTH-0002",
            title: "TLS is disabled on an exposed endpoint",
            severity: Severity::High,
            category: Category::Authentication,
            summary: "Traffic to the exposed control plane is not protected by TLS.".to_string(),
            evidence: "tls_enabled=false".to_string(),
            remediation:
                "Enable TLS or place the control plane behind a TLS-terminating reverse proxy."
                    .to_string(),
            auto_fix_supported: false,
        });
    }

    if public_bind && config.source_allowlist.is_empty() {
        findings.push(Finding {
            id: "AUTH-0003",
            title: "Publicly exposed service lacks source restrictions",
            severity: Severity::High,
            category: Category::Authentication,
            summary: "No source allowlist is configured for a publicly reachable service.".to_string(),
            evidence: "source_allowlist is empty".to_string(),
            remediation: "Restrict access to approved admin IPs, loopback, or a tunnel endpoint."
                .to_string(),
            auto_fix_supported: true,
        });
    }

    if config.webhook_enabled && config.webhook_public_key.is_none() {
        findings.push(Finding {
            id: "PERM-0001",
            title: "Webhook verification can fail open",
            severity: Severity::High,
            category: Category::Permission,
            summary: "Webhook support is enabled but no public verification key is configured."
                .to_string(),
            evidence: "webhook_enabled=true and webhook_public_key is missing".to_string(),
            remediation: "Configure the provider public key before accepting webhook traffic."
                .to_string(),
            auto_fix_supported: true,
        });
    }

    if !config.approval_preview_consistent || !config.command_allowlist_normalized {
        findings.push(Finding {
            id: "PERM-0002",
            title: "Approval and execution policies are inconsistent",
            severity: Severity::High,
            category: Category::Permission,
            summary: "Preview, approval, and execution settings do not use a consistent policy surface."
                .to_string(),
            evidence: format!(
                "approval_preview_consistent={}, command_allowlist_normalized={}",
                config.approval_preview_consistent, config.command_allowlist_normalized
            ),
            remediation:
                "Normalize policy checks and require preview-to-execution parity for approved actions."
                    .to_string(),
            auto_fix_supported: true,
        });
    }

    if config.debug_enabled || config.skills_status_exposes_secrets {
        findings.push(Finding {
            id: "SECR-0001",
            title: "Debug or status surfaces can disclose secrets",
            severity: Severity::High,
            category: Category::Secrets,
            summary: "Debug exposure or skill status output can leak sensitive values.".to_string(),
            evidence: format!(
                "debug_enabled={}, skills_status_exposes_secrets={}",
                config.debug_enabled, config.skills_status_exposes_secrets
            ),
            remediation: "Disable debug endpoints and redact secrets from any skill status output."
                .to_string(),
            auto_fix_supported: true,
        });
    }

    let matched_skills = suspicious_skill_matches(&config.suspicious_skills, rules);
    if !matched_skills.is_empty() {
        findings.push(Finding {
            id: "SUP-0001",
            title: "Suspicious skills are installed",
            severity: Severity::Critical,
            category: Category::SupplyChain,
            summary: "The deployment lists skills that should be disabled or reviewed immediately."
                .to_string(),
            evidence: format!("suspicious_skills={}", matched_skills.join(",")),
            remediation: "Disable the listed skills and verify their provenance before re-enabling."
                .to_string(),
            auto_fix_supported: true,
        });
    }

    if let Some(origin) = &config.installer_origin {
        if !is_trusted_origin(origin, rules) {
            findings.push(Finding {
                id: "SUP-0002",
                title: "Installer or package origin is not trusted",
                severity: Severity::High,
                category: Category::SupplyChain,
                summary: "The recorded installer origin does not match a trusted release source."
                    .to_string(),
                evidence: format!("installer_origin={origin}"),
                remediation: "Reinstall or verify the deployment using an official signed release path."
                    .to_string(),
                auto_fix_supported: false,
            });
        }
    }

    build_report(config.profile_name.clone(), findings)
}

pub fn render_report_json(report: &ScanReport) -> String {
    let findings = report
        .findings
        .iter()
        .map(|finding| {
            format!(
                concat!(
                    "{{",
                    "\"id\":\"{}\",",
                    "\"title\":\"{}\",",
                    "\"severity\":\"{}\",",
                    "\"category\":\"{}\",",
                    "\"summary\":\"{}\",",
                    "\"evidence\":\"{}\",",
                    "\"remediation\":\"{}\",",
                    "\"auto_fix_supported\":{}",
                    "}}"
                ),
                json_escape(finding.id),
                json_escape(finding.title),
                json_escape(&finding.severity.to_string()),
                json_escape(&finding.category.to_string()),
                json_escape(&finding.summary),
                json_escape(&finding.evidence),
                json_escape(&finding.remediation),
                finding.auto_fix_supported
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"profile_name\":\"{}\",\"risk_score\":{},\"finding_count\":{},\"findings\":[{}]}}",
        json_escape(&report.profile_name),
        report.risk_score,
        report.findings.len(),
        findings
    )
}

pub fn render_report_html(report: &ScanReport) -> String {
    render_report_html_with_locale(report, Locale::En)
}

pub fn render_report_html_with_locale(report: &ScanReport, locale: Locale) -> String {
    let labels = labels_for(locale);
    let rows = report
        .findings
        .iter()
        .map(|finding| {
            format!(
                "<article class=\"finding\"><header><span class=\"severity {severity_class}\">{severity}</span><h2>{title}</h2></header><p>{summary}</p><p><strong>{evidence_label}:</strong> {evidence}</p><p><strong>{remediation_label}:</strong> {remediation}</p></article>",
                severity_class = finding.severity,
                severity = localized_severity(&finding.severity, locale),
                title = html_escape(localized_finding_title(finding, locale)),
                summary = html_escape(&finding.summary),
                evidence_label = labels.evidence,
                evidence = html_escape(&finding.evidence),
                remediation_label = labels.remediation,
                remediation = html_escape(&finding.remediation)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        concat!(
            "<!doctype html><html lang=\"{lang}\"><head><meta charset=\"utf-8\">",
            "<title>{title}</title>",
            "<style>",
            "body{{font-family:Georgia,serif;background:#f4f0e8;color:#1f2933;margin:0;padding:32px;}}",
            ".hero{{background:linear-gradient(135deg,#0f4c5c,#e36414);color:#fff;padding:24px;border-radius:18px;margin-bottom:24px;}}",
            ".metrics{{display:flex;gap:16px;flex-wrap:wrap;margin-top:12px;}}",
            ".metric{{background:rgba(255,255,255,.15);padding:12px 16px;border-radius:12px;}}",
            ".finding{{background:#fff;border-radius:16px;padding:20px;margin-bottom:16px;box-shadow:0 12px 30px rgba(15,76,92,.08);}}",
            ".severity{{display:inline-block;padding:4px 8px;border-radius:999px;text-transform:uppercase;font-size:12px;letter-spacing:.08em;}}",
            ".critical{{background:#7f1d1d;color:#fee2e2;}}",
            ".high{{background:#9a3412;color:#ffedd5;}}",
            ".medium{{background:#854d0e;color:#fef3c7;}}",
            ".low{{background:#166534;color:#dcfce7;}}",
            "h1,h2{{margin:0 0 12px 0;}}",
            "</style></head><body>",
            "<section class=\"hero\"><h1>{title}</h1><p>{profile_label}: {profile}</p>",
            "<div class=\"metrics\"><div class=\"metric\">{risk_score_label}: {score}</div><div class=\"metric\">{findings_label}: {count}</div></div></section>",
            "{rows}",
            "</body></html>"
        ),
        lang = locale.lang_attr(),
        title = labels.report_title,
        profile_label = labels.profile,
        profile = html_escape(&report.profile_name),
        risk_score_label = labels.risk_score,
        score = report.risk_score,
        findings_label = labels.findings,
        count = report.findings.len(),
        rows = rows
    )
}

pub fn render_report_text_with_locale(report: &ScanReport, locale: Locale) -> String {
    let labels = labels_for(locale);
    let findings = report
        .findings
        .iter()
        .enumerate()
        .map(|(index, finding)| {
            format!(
                "{n}. [{severity}] {title}\n   {summary_label}: {summary}\n   {evidence_label}: {evidence}\n   {remediation_label}: {remediation}",
                n = index + 1,
                severity = localized_severity(&finding.severity, locale),
                title = localized_finding_title(finding, locale),
                summary_label = labels.summary,
                summary = finding.summary,
                evidence_label = labels.evidence,
                evidence = finding.evidence,
                remediation_label = labels.remediation,
                remediation = finding.remediation
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "{title}\n{profile_label}: {profile}\n{risk_score_label}: {score}\n{findings_label}: {count}\n\n{finding_lines}",
        title = labels.report_title,
        profile_label = labels.profile,
        profile = report.profile_name,
        risk_score_label = labels.risk_score,
        score = report.risk_score,
        findings_label = labels.findings,
        count = report.findings.len(),
        finding_lines = findings
    )
}

pub fn harden_config_file(
    input_path: &Path,
    output_path: Option<&Path>,
    in_place: bool,
) -> Result<HardenOutcome, String> {
    let config = load_config(input_path)?;
    let before = scan_config(&config);
    let (hardened, applied_actions, manual_actions) = harden_config(&config);
    let after = scan_config(&hardened);

    let destination = if in_place {
        input_path.to_path_buf()
    } else {
        output_path
            .map(Path::to_path_buf)
            .unwrap_or_else(|| default_hardened_path(input_path))
    };

    let mut backup_path = None;
    if in_place {
        let backup = backup_path_for(input_path);
        fs::copy(input_path, &backup).map_err(|error| {
            format!(
                "failed to create backup {}: {error}",
                backup.as_path().display()
            )
        })?;
        backup_path = Some(backup);
    }

    if let Err(error) = write_config(&destination, &hardened) {
        if let Some(backup) = &backup_path {
            let _ = fs::copy(backup, input_path);
        }
        return Err(error);
    }

    Ok(HardenOutcome {
        backup_path,
        output_path: destination,
        before_score: before.risk_score,
        after_score: after.risk_score,
        applied_actions,
        manual_actions,
    })
}

pub fn harden_config(config: &OpenClawConfig) -> (OpenClawConfig, Vec<String>, Vec<String>) {
    let mut hardened = config.clone();
    let mut applied = Vec::new();
    let mut manual = Vec::new();

    if is_public_bind(&hardened.bind_address) {
        hardened.bind_address = "127.0.0.1".to_string();
        applied.push("Restricted bind_address to 127.0.0.1".to_string());
    }

    if token_strength(hardened.auth_token.as_deref(), &Ruleset::default()) < 2 {
        hardened.auth_token = Some(generate_token());
        applied.push("Rotated authentication token".to_string());
    }

    if hardened.source_allowlist.is_empty() {
        hardened.source_allowlist = vec!["127.0.0.1/32".to_string()];
        applied.push("Added loopback source allowlist".to_string());
    }

    if hardened.webhook_enabled && hardened.webhook_public_key.is_none() {
        hardened.webhook_public_key = Some("REQUIRED_PUBLIC_KEY".to_string());
        applied.push("Inserted placeholder webhook public key requirement".to_string());
    }

    if !hardened.approval_preview_consistent {
        hardened.approval_preview_consistent = true;
        applied.push("Forced approval preview consistency".to_string());
    }

    if !hardened.command_allowlist_normalized {
        hardened.command_allowlist_normalized = true;
        applied.push("Enabled command allowlist normalization".to_string());
    }

    if hardened.debug_enabled {
        hardened.debug_enabled = false;
        applied.push("Disabled debug surfaces".to_string());
    }

    if hardened.skills_status_exposes_secrets {
        hardened.skills_status_exposes_secrets = false;
        applied.push("Disabled skill status secret exposure".to_string());
    }

    if !hardened.suspicious_skills.is_empty() {
        hardened.suspicious_skills.clear();
        applied.push("Disabled suspicious skills".to_string());
    }

    if !hardened.tls_enabled {
        manual.push(
            "Enable TLS or configure a trusted TLS-terminating reverse proxy before public exposure."
                .to_string(),
        );
    }

    if hardened.port == 18789 {
        manual.push(
            "Verify that port 18789 is not reachable from the public internet after hardening."
                .to_string(),
        );
    }

    if let Some(origin) = &hardened.installer_origin {
        if !is_trusted_origin(origin, &Ruleset::default()) {
            manual.push(
                "Reinstall from an official signed release channel and verify package provenance."
                    .to_string(),
            );
        }
    }

    (hardened, applied, manual)
}

fn parse_bool(value: &str, index: usize) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("invalid boolean on line {}", index + 1)),
    }
}

fn parse_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .filter_map(|part| {
            let trimmed = part.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .collect()
}

fn build_report(profile_name: String, findings: Vec<Finding>) -> ScanReport {
    let total_weight: u16 = findings
        .iter()
        .map(|finding| u16::from(finding.severity.weight()))
        .sum();
    let risk_score = 100u8.saturating_sub(total_weight.min(95) as u8);

    ScanReport {
        profile_name,
        findings,
        risk_score,
    }
}

fn token_strength(token: Option<&str>, rules: &Ruleset) -> u8 {
    match token {
        None => 0,
        Some("") => 0,
        Some(value) if rules.weak_tokens.iter().any(|item| item == value) => 1,
        Some(value) if value.len() < 24 => 1,
        Some(_) => 2,
    }
}

fn is_public_bind(address: &str) -> bool {
    !matches!(address, "127.0.0.1" | "::1" | "localhost")
}

fn suspicious_skill_matches(skills: &[String], rules: &Ruleset) -> Vec<String> {
    skills
        .iter()
        .filter(|skill| {
            let normalized = skill.to_ascii_lowercase();
            rules
                .suspicious_skill_patterns
                .iter()
                .any(|pattern| normalized.contains(&pattern.to_ascii_lowercase()))
        })
        .cloned()
        .collect()
}

fn is_trusted_origin(origin: &str, rules: &Ruleset) -> bool {
    rules.trusted_origins.iter().any(|item| item == origin)
}

fn contains_secret_material(content: &str) -> bool {
    let upper = content.to_ascii_uppercase();
    upper.contains("TOKEN=") || upper.contains("SECRET=") || upper.contains("_KEY=")
}

fn log_contains_secrets(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    lower.contains("auth_token=") || lower.contains("api_key=") || lower.contains("secret=")
}

fn collect_log_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let entries = fs::read_dir(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read log entry: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to read file type: {error}"))?;
        if file_type.is_file() {
            files.push(entry.path());
        }
    }

    Ok(files)
}

fn json_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

struct ReportLabels {
    report_title: &'static str,
    profile: &'static str,
    risk_score: &'static str,
    findings: &'static str,
    summary: &'static str,
    evidence: &'static str,
    remediation: &'static str,
}

fn labels_for(locale: Locale) -> ReportLabels {
    match locale {
        Locale::En => ReportLabels {
            report_title: "Clawguard Report",
            profile: "Profile",
            risk_score: "Risk Score",
            findings: "Findings",
            summary: "Summary",
            evidence: "Evidence",
            remediation: "Remediation",
        },
        Locale::ZhCn => ReportLabels {
            report_title: "Clawguard 安全报告",
            profile: "配置档案",
            risk_score: "风险评分",
            findings: "发现项",
            summary: "摘要",
            evidence: "证据",
            remediation: "修复建议",
        },
    }
}

fn localized_severity(severity: &Severity, locale: Locale) -> &'static str {
    match locale {
        Locale::En => match severity {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
        },
        Locale::ZhCn => match severity {
            Severity::Critical => "严重",
            Severity::High => "高危",
            Severity::Medium => "中危",
            Severity::Low => "低危",
        },
    }
}

fn localized_finding_title(finding: &Finding, locale: Locale) -> &str {
    match locale {
        Locale::En => finding.title,
        Locale::ZhCn => match finding.id {
            "EXP-0001" => "控制面绑定到了公网接口",
            "EXP-0002" => "默认控制端口处于公网暴露状态",
            "AUTH-0001" => "认证令牌缺失或过弱",
            "AUTH-0002" => "暴露端点未启用 TLS",
            "AUTH-0003" => "公网服务缺少来源限制",
            "PERM-0001" => "Webhook 校验可能处于 fail-open",
            "PERM-0002" => "审批策略与真实执行策略不一致",
            "SECR-0001" => "调试或状态接口可能泄露敏感信息",
            "SECR-0002" => "本地环境文件存放了敏感凭证",
            "SECR-0003" => "日志文件包含凭证内容",
            "SUP-0001" => "检测到可疑技能",
            "SUP-0002" => "安装来源不可信",
            "SUP-0003" => "已安装技能命中可疑规则",
            _ => finding.title,
        },
    }
}

fn generate_token() -> String {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let mut state = (seed as u64) ^ 0x9e37_79b9_7f4a_7c15;
    let mut output = String::with_capacity(32);

    while output.len() < 32 {
        state ^= state << 7;
        state ^= state >> 9;
        state ^= state << 8;
        output.push(char::from(b'a' + (state % 26) as u8));
    }

    output
}

fn default_hardened_path(input_path: &Path) -> PathBuf {
    let mut candidate = input_path.to_path_buf();
    candidate.set_extension("hardened.conf");
    candidate
}

fn backup_path_for(input_path: &Path) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let file_name = format!(
        "{}.bak.{}",
        input_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("openclaw.conf"),
        millis
    );
    input_path.with_file_name(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn parses_config_lines() {
        let config = OpenClawConfig::parse(
            "profile_name=edge\nbind_address=0.0.0.0\nport=18789\ntls_enabled=false\nauth_token=changeme\nsource_allowlist=\nwebhook_enabled=true\nwebhook_public_key=\napproval_preview_consistent=false\ncommand_allowlist_normalized=false\ndebug_enabled=true\nskills_status_exposes_secrets=true\nsuspicious_skills=wallet-sync\ninstaller_origin=community-fork",
        )
        .expect("config should parse");

        assert_eq!(config.profile_name, "edge");
        assert_eq!(config.bind_address, "0.0.0.0");
        assert!(config.webhook_enabled);
        assert_eq!(config.suspicious_skills, vec!["wallet-sync"]);
    }

    #[test]
    fn scan_finds_expected_risks() {
        let config = OpenClawConfig::parse(
            "profile_name=edge\nbind_address=0.0.0.0\nport=18789\ntls_enabled=false\nauth_token=changeme\nsource_allowlist=\nwebhook_enabled=true\nwebhook_public_key=\napproval_preview_consistent=false\ncommand_allowlist_normalized=false\ndebug_enabled=true\nskills_status_exposes_secrets=true\nsuspicious_skills=wallet-sync,crypto-helper\ninstaller_origin=community-fork",
        )
        .expect("config should parse");
        let report = scan_config(&config);

        assert!(report.findings.len() >= 8);
        assert!(report.risk_score < 30);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.id == "PERM-0002")
        );
    }

    #[test]
    fn hardening_reduces_risk_score() {
        let config = OpenClawConfig::parse(
            "profile_name=edge\nbind_address=0.0.0.0\nport=18789\ntls_enabled=false\nauth_token=changeme\nsource_allowlist=\nwebhook_enabled=true\nwebhook_public_key=\napproval_preview_consistent=false\ncommand_allowlist_normalized=false\ndebug_enabled=true\nskills_status_exposes_secrets=true\nsuspicious_skills=wallet-sync\ninstaller_origin=official",
        )
        .expect("config should parse");

        let before = scan_config(&config);
        let (after_config, applied, manual) = harden_config(&config);
        let after = scan_config(&after_config);

        assert!(!applied.is_empty());
        assert!(!manual.is_empty());
        assert!(after.risk_score > before.risk_score);
        assert_eq!(after_config.bind_address, "127.0.0.1");
        assert!(after_config.auth_token.unwrap().len() >= 24);
        assert!(after_config.suspicious_skills.is_empty());
    }

    #[test]
    fn in_place_hardening_creates_backup() {
        let config = OpenClawConfig::parse(
            "profile_name=edge\nbind_address=0.0.0.0\nport=18789\ntls_enabled=true\nauth_token=changeme\nsource_allowlist=\nwebhook_enabled=false\nwebhook_public_key=\napproval_preview_consistent=true\ncommand_allowlist_normalized=true\ndebug_enabled=false\nskills_status_exposes_secrets=false\nsuspicious_skills=\ninstaller_origin=official",
        )
        .expect("config should parse");

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let path = env::temp_dir().join(format!("clawguard-{unique}.conf"));
        write_config(&path, &config).expect("config should be written");

        let outcome = harden_config_file(&path, None, true).expect("hardening should succeed");

        assert!(outcome.backup_path.is_some());
        assert!(outcome.after_score > outcome.before_score);

        if let Some(backup) = outcome.backup_path {
            let _ = fs::remove_file(backup);
        }
        let _ = fs::remove_file(path);
    }
}
