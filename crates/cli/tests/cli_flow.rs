use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!("clawguard-{name}-{unique}"))
}

fn cli_bin_path() -> PathBuf {
    let test_bin = env::current_exe().expect("test binary path should exist");
    let debug_dir = test_bin
        .parent()
        .and_then(|value| value.parent())
        .expect("debug directory should exist");

    #[cfg(target_os = "windows")]
    let bin_name = "clawguard.exe";
    #[cfg(not(target_os = "windows"))]
    let bin_name = "clawguard";

    debug_dir.join(bin_name)
}

#[test]
fn sample_config_and_scan_work_end_to_end() {
    let bin = cli_bin_path();
    let config_path = temp_path("sample.conf");
    let report_path = temp_path("report.json");

    let sample = Command::new(&bin)
        .args(["sample-config", "--output"])
        .arg(&config_path)
        .output()
        .expect("sample-config should run");
    assert!(sample.status.success());

    let scan = Command::new(&bin)
        .args(["scan", "--config"])
        .arg(&config_path)
        .args(["--format", "json", "--output"])
        .arg(&report_path)
        .output()
        .expect("scan should run");
    assert!(scan.status.success());

    let report = fs::read_to_string(&report_path).expect("report should exist");
    assert!(report.contains("\"risk_score\""));

    let _ = fs::remove_file(config_path);
    let _ = fs::remove_file(report_path);
}

#[test]
fn scan_respects_custom_rules() {
    let bin = cli_bin_path();
    let config_path = temp_path("custom.conf");
    let rules_path = temp_path("custom.rules");

    fs::write(
        &config_path,
        "profile_name=edge\nbind_address=127.0.0.1\nport=18789\ntls_enabled=true\nauth_token=abcdefghijklmnopqrstuvwxyz\nsource_allowlist=127.0.0.1/32\nwebhook_enabled=false\nwebhook_public_key=\napproval_preview_consistent=true\ncommand_allowlist_normalized=true\ndebug_enabled=false\nskills_status_exposes_secrets=false\nsuspicious_skills=ops-helper\ninstaller_origin=enterprise-mirror",
    )
    .expect("config should be written");
    fs::write(
        &rules_path,
        "trusted_origins=official,enterprise-mirror\nsuspicious_skill_patterns=wallet,exfil\nweak_tokens=default-token,changeme",
    )
    .expect("rules should be written");

    let scan = Command::new(&bin)
        .args(["scan", "--config"])
        .arg(&config_path)
        .args(["--rules"])
        .arg(&rules_path)
        .output()
        .expect("scan should run");
    assert!(scan.status.success());

    let stdout = String::from_utf8(scan.stdout).expect("stdout should be valid utf-8");
    assert!(!stdout.contains("\"SUP-0001\""));
    assert!(!stdout.contains("\"SUP-0002\""));

    let _ = fs::remove_file(config_path);
    let _ = fs::remove_file(rules_path);
}

#[test]
fn scan_profile_reports_artifact_findings() {
    let bin = cli_bin_path();
    let root = temp_path("profile");
    let logs_dir = root.join("logs");

    fs::create_dir_all(&logs_dir).expect("logs directory should exist");
    fs::write(
        root.join("openclaw.conf"),
        "profile_name=edge\nbind_address=127.0.0.1\nport=18789\ntls_enabled=true\nauth_token=abcdefghijklmnopqrstuvwxyz\nsource_allowlist=127.0.0.1/32\nwebhook_enabled=false\nwebhook_public_key=\napproval_preview_consistent=true\ncommand_allowlist_normalized=true\ndebug_enabled=false\nskills_status_exposes_secrets=false\nsuspicious_skills=\ninstaller_origin=official",
    )
    .expect("config should be written");
    fs::write(root.join(".env"), "OPENCLAW_AUTH_TOKEN=super-secret\n")
        .expect("env file should be written");
    fs::write(logs_dir.join("openclaw.log"), "auth_token=super-secret\n")
        .expect("log file should be written");

    let scan = Command::new(&bin)
        .args(["scan-profile", "--path"])
        .arg(&root)
        .output()
        .expect("scan-profile should run");
    assert!(scan.status.success());

    let stdout = String::from_utf8(scan.stdout).expect("stdout should be valid utf-8");
    assert!(stdout.contains("\"SECR-0002\""));
    assert!(stdout.contains("\"SECR-0003\""));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn localized_text_scan_uses_requested_locale() {
    let bin = cli_bin_path();
    let config_path = temp_path("localized.conf");

    fs::write(
        &config_path,
        "profile_name=edge\nbind_address=0.0.0.0\nport=18789\ntls_enabled=false\nauth_token=changeme\nsource_allowlist=\nwebhook_enabled=false\nwebhook_public_key=\napproval_preview_consistent=true\ncommand_allowlist_normalized=true\ndebug_enabled=false\nskills_status_exposes_secrets=false\nsuspicious_skills=\ninstaller_origin=official",
    )
    .expect("config should be written");

    let scan = Command::new(&bin)
        .args(["scan", "--config"])
        .arg(&config_path)
        .args(["--format", "text", "--locale", "zh-CN"])
        .output()
        .expect("localized scan should run");
    assert!(scan.status.success());

    let stdout = String::from_utf8(scan.stdout).expect("stdout should be valid utf-8");
    assert!(stdout.contains("风险评分"));
    assert!(stdout.contains("发现项"));

    let _ = fs::remove_file(config_path);
}
