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

#[test]
fn localized_text_scan_uses_system_locale_by_default() {
    let bin = cli_bin_path();
    let config_path = temp_path("system-locale.conf");

    fs::write(
        &config_path,
        "profile_name=edge\nbind_address=0.0.0.0\nport=18789\ntls_enabled=false\nauth_token=changeme\nsource_allowlist=\nwebhook_enabled=false\nwebhook_public_key=\napproval_preview_consistent=true\ncommand_allowlist_normalized=true\ndebug_enabled=false\nskills_status_exposes_secrets=false\nsuspicious_skills=\ninstaller_origin=official",
    )
    .expect("config should be written");

    let scan = Command::new(&bin)
        .args(["scan", "--config"])
        .arg(&config_path)
        .args(["--format", "text"])
        .env("LANG", "zh_CN.UTF-8")
        .output()
        .expect("localized scan should run");
    assert!(scan.status.success());

    let stdout = String::from_utf8(scan.stdout).expect("stdout should be valid utf-8");
    assert!(stdout.contains("风险评分"));
    assert!(stdout.contains("发现项"));

    let _ = fs::remove_file(config_path);
}

#[test]
fn help_shows_banner_and_localized_tagline() {
    let bin = cli_bin_path();

    let help = Command::new(&bin)
        .arg("--help")
        .env("LANG", "zh_CN.UTF-8")
        .output()
        .expect("help should run");
    assert!(help.status.success());

    let stdout = String::from_utf8(help.stdout).expect("stdout should be valid utf-8");
    assert!(stdout.contains("ClawGuard"));
    assert!(stdout.contains("小龙虾卫士"));
    assert!(stdout.contains("OpenClaw 安全审计与加固 CLI"));
}

#[test]
fn uninstall_command_removes_installed_binary() {
    let bin = cli_bin_path();
    let install_dir = temp_path("install-dir");
    fs::create_dir_all(&install_dir).expect("install dir should exist");

    #[cfg(target_os = "windows")]
    let binary_name = "clawguard.exe";
    #[cfg(not(target_os = "windows"))]
    let binary_name = "clawguard";

    let installed_binary = install_dir.join(binary_name);
    fs::write(&installed_binary, "dummy").expect("dummy install should be written");

    let uninstall = Command::new(&bin)
        .args(["uninstall", "--install-dir"])
        .arg(&install_dir)
        .output()
        .expect("uninstall should run");
    assert!(uninstall.status.success());
    assert!(!installed_binary.exists());

    let _ = fs::remove_dir_all(install_dir);
}

#[test]
fn rules_pack_commands_manage_store_and_drive_scan_behavior() {
    let bin = cli_bin_path();
    let root = temp_path("rules-pack-flow");
    let keys_dir = root.join("keys");
    let store_dir = root.join("store");
    let rules_path = root.join("custom.rules");
    let pack_path = root.join("custom-pack.json");
    let config_path = root.join("custom.conf");

    fs::create_dir_all(&root).expect("temp root should exist");
    fs::write(
        &rules_path,
        "trusted_origins=official,enterprise-mirror\nsuspicious_skill_patterns=wallet,exfil\nweak_tokens=default-token,changeme",
    )
    .expect("rules should be written");
    fs::write(
        &config_path,
        "profile_name=edge\nbind_address=127.0.0.1\nport=18789\ntls_enabled=true\nauth_token=abcdefghijklmnopqrstuvwxyz\nsource_allowlist=127.0.0.1/32\nwebhook_enabled=false\nwebhook_public_key=\napproval_preview_consistent=true\ncommand_allowlist_normalized=true\ndebug_enabled=false\nskills_status_exposes_secrets=false\nsuspicious_skills=ops-helper\ninstaller_origin=enterprise-mirror",
    )
    .expect("config should be written");

    let keygen = Command::new(&bin)
        .args(["generate-signing-keypair", "--output-dir"])
        .arg(&keys_dir)
        .args(["--prefix", "test-rules"])
        .output()
        .expect("key generation should run");
    assert!(keygen.status.success());

    let private_key_path = keys_dir.join("test-rules.private.key");
    let public_key_path = keys_dir.join("test-rules.public.key");
    assert!(private_key_path.exists());
    assert!(public_key_path.exists());

    let sign = Command::new(&bin)
        .args(["sign-rules-pack", "--output"])
        .arg(&pack_path)
        .args(["--version", "1.2.3", "--private-key"])
        .arg(&private_key_path)
        .args(["--rules"])
        .arg(&rules_path)
        .args(["--key-id", "ci"])
        .output()
        .expect("sign should run");
    assert!(sign.status.success());

    let import = Command::new(&bin)
        .args(["import-rules-pack", "--pack"])
        .arg(&pack_path)
        .args(["--public-key"])
        .arg(&public_key_path)
        .args(["--store"])
        .arg(&store_dir)
        .arg("--activate")
        .output()
        .expect("import should run");
    assert!(import.status.success());

    let status = Command::new(&bin)
        .args(["rules-status", "--store"])
        .arg(&store_dir)
        .output()
        .expect("rules status should run");
    assert!(status.status.success());
    let status_stdout = String::from_utf8(status.stdout).expect("stdout should be utf-8");
    assert!(status_stdout.contains("active_version=1.2.3"));

    let scan = Command::new(&bin)
        .args(["scan", "--config"])
        .arg(&config_path)
        .args(["--rules-store"])
        .arg(&store_dir)
        .output()
        .expect("scan should run");
    assert!(scan.status.success());
    let scan_stdout = String::from_utf8(scan.stdout).expect("stdout should be utf-8");
    assert!(!scan_stdout.contains("\"SUP-0002\""));
    assert!(!scan_stdout.contains("\"SUP-0001\""));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn rollback_rules_restores_previous_active_version() {
    let bin = cli_bin_path();
    let root = temp_path("rules-rollback");
    let keys_dir = root.join("keys");
    let store_dir = root.join("store");
    let pack_v1_path = root.join("pack-v1.json");
    let pack_v2_path = root.join("pack-v2.json");
    let rules_path = root.join("custom.rules");

    fs::create_dir_all(&root).expect("temp root should exist");
    fs::write(
        &rules_path,
        "trusted_origins=official,enterprise-mirror\nsuspicious_skill_patterns=wallet,exfil\nweak_tokens=default-token,changeme",
    )
    .expect("rules should be written");

    let keygen = Command::new(&bin)
        .args(["generate-signing-keypair", "--output-dir"])
        .arg(&keys_dir)
        .args(["--prefix", "rollback"])
        .output()
        .expect("key generation should run");
    assert!(keygen.status.success());

    let private_key_path = keys_dir.join("rollback.private.key");
    let public_key_path = keys_dir.join("rollback.public.key");

    let sign_v1 = Command::new(&bin)
        .args(["sign-rules-pack", "--output"])
        .arg(&pack_v1_path)
        .args(["--version", "1.0.0", "--private-key"])
        .arg(&private_key_path)
        .output()
        .expect("sign v1 should run");
    assert!(sign_v1.status.success());

    let sign_v2 = Command::new(&bin)
        .args(["sign-rules-pack", "--output"])
        .arg(&pack_v2_path)
        .args(["--version", "2.0.0", "--private-key"])
        .arg(&private_key_path)
        .args(["--rules"])
        .arg(&rules_path)
        .output()
        .expect("sign v2 should run");
    assert!(sign_v2.status.success());

    let import_v1 = Command::new(&bin)
        .args(["import-rules-pack", "--pack"])
        .arg(&pack_v1_path)
        .args(["--public-key"])
        .arg(&public_key_path)
        .args(["--store"])
        .arg(&store_dir)
        .arg("--activate")
        .output()
        .expect("import v1 should run");
    assert!(import_v1.status.success());

    let import_v2 = Command::new(&bin)
        .args(["import-rules-pack", "--pack"])
        .arg(&pack_v2_path)
        .args(["--public-key"])
        .arg(&public_key_path)
        .args(["--store"])
        .arg(&store_dir)
        .arg("--activate")
        .output()
        .expect("import v2 should run");
    assert!(import_v2.status.success());

    let rollback = Command::new(&bin)
        .args(["rollback-rules", "--store"])
        .arg(&store_dir)
        .output()
        .expect("rollback should run");
    assert!(rollback.status.success());
    let rollback_stdout = String::from_utf8(rollback.stdout).expect("stdout should be utf-8");
    assert!(rollback_stdout.contains("active_version=1.0.0"));

    let _ = fs::remove_dir_all(root);
}
