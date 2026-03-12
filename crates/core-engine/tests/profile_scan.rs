use clawguard_core::{scan_profile_with_rules, Ruleset};
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn profile_scan_detects_secret_artifacts() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let root = env::temp_dir().join(format!("openclaw-profile-{unique}"));
    let logs_dir = root.join("logs");
    let skills_dir = root.join("skills");

    fs::create_dir_all(&logs_dir).expect("logs directory should exist");
    fs::create_dir_all(&skills_dir).expect("skills directory should exist");
    fs::write(
        root.join("openclaw.conf"),
        "profile_name=edge\nbind_address=127.0.0.1\nport=18789\ntls_enabled=true\nauth_token=abcdefghijklmnopqrstuvwxyz\nsource_allowlist=127.0.0.1/32\nwebhook_enabled=false\nwebhook_public_key=\napproval_preview_consistent=true\ncommand_allowlist_normalized=true\ndebug_enabled=false\nskills_status_exposes_secrets=false\nsuspicious_skills=\ninstaller_origin=official",
    )
    .expect("config should be written");
    fs::write(root.join(".env"), "OPENCLAW_AUTH_TOKEN=super-secret\n")
        .expect("env file should be written");
    fs::write(logs_dir.join("openclaw.log"), "auth_token=super-secret\n")
        .expect("log file should be written");
    fs::write(skills_dir.join("installed.txt"), "wallet-sync\nops-helper\n")
        .expect("skills file should be written");

    let report = scan_profile_with_rules(&root, &Ruleset::default()).expect("profile scan should work");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "SECR-0002")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "SECR-0003")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.id == "SUP-0003")
    );

    let _ = fs::remove_dir_all(root);
}
