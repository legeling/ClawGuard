use clawguard_core::{
    activate_rules_pack, create_signed_rules_pack, generate_signing_keypair_hex,
    import_rules_pack, load_active_ruleset, rollback_rules_pack, rules_store_status,
    scan_config_with_rules, write_rules_pack, OpenClawConfig, Ruleset,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!("clawguard-{name}-{unique}"))
}

#[test]
fn rules_pack_store_supports_import_activation_and_rollback() {
    let root = temp_path("rules-store");
    let pack_v1_path = root.join("pack-v1.json");
    let pack_v2_path = root.join("pack-v2.json");
    let store_dir = root.join("store");
    let (private_key, public_key) = generate_signing_keypair_hex();

    fs::create_dir_all(&root).expect("temp root should exist");

    let default_pack = create_signed_rules_pack(Ruleset::default(), "0.1.0", "ci", &private_key)
        .expect("default pack should sign");
    write_rules_pack(&pack_v1_path, &default_pack).expect("default pack should be written");
    import_rules_pack(&pack_v1_path, &public_key, &store_dir).expect("default pack should import");
    activate_rules_pack(&store_dir, "0.1.0").expect("default pack should activate");

    let relaxed_rules = Ruleset::parse(
        "trusted_origins=official,enterprise-mirror\nsuspicious_skill_patterns=wallet,exfil\nweak_tokens=default-token,changeme",
    )
    .expect("custom rules should parse");
    let relaxed_pack = create_signed_rules_pack(relaxed_rules, "0.2.0", "ci", &private_key)
        .expect("relaxed pack should sign");
    write_rules_pack(&pack_v2_path, &relaxed_pack).expect("relaxed pack should be written");
    import_rules_pack(&pack_v2_path, &public_key, &store_dir).expect("relaxed pack should import");
    activate_rules_pack(&store_dir, "0.2.0").expect("relaxed pack should activate");

    let status = rules_store_status(&store_dir).expect("rules status should load");
    assert_eq!(status.active_version.as_deref(), Some("0.2.0"));
    assert_eq!(status.installed_versions, vec!["0.1.0", "0.2.0"]);

    let config = OpenClawConfig::parse(
        "profile_name=edge\nbind_address=127.0.0.1\nport=18789\ntls_enabled=true\nauth_token=abcdefghijklmnopqrstuvwxyz\nsource_allowlist=127.0.0.1/32\nwebhook_enabled=false\nwebhook_public_key=\napproval_preview_consistent=true\ncommand_allowlist_normalized=true\ndebug_enabled=false\nskills_status_exposes_secrets=false\nsuspicious_skills=ops-helper\ninstaller_origin=enterprise-mirror",
    )
    .expect("config should parse");

    let active_rules = load_active_ruleset(&store_dir)
        .expect("active rules should load")
        .expect("active rules should exist");
    let relaxed_report = scan_config_with_rules(&config, &active_rules);
    assert!(relaxed_report
        .findings
        .iter()
        .all(|finding| finding.id != "SUP-0002"));

    rollback_rules_pack(&store_dir).expect("rollback should succeed");

    let rolled_back_rules = load_active_ruleset(&store_dir)
        .expect("rolled back rules should load")
        .expect("rolled back rules should exist");
    let rolled_back_report = scan_config_with_rules(&config, &rolled_back_rules);
    assert!(rolled_back_report
        .findings
        .iter()
        .any(|finding| finding.id == "SUP-0002"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn tampered_rules_pack_is_rejected() {
    let root = temp_path("rules-store-invalid");
    let pack_path = root.join("pack.json");
    let store_dir = root.join("store");
    let (private_key, public_key) = generate_signing_keypair_hex();

    fs::create_dir_all(&root).expect("temp root should exist");

    let mut pack = create_signed_rules_pack(Ruleset::default(), "0.1.0", "ci", &private_key)
        .expect("pack should sign");
    pack.signature_hex.replace_range(..2, "00");
    write_rules_pack(&pack_path, &pack).expect("tampered pack should be written");

    let error = import_rules_pack(&pack_path, &public_key, &store_dir)
        .expect_err("tampered pack should be rejected");
    assert!(error.contains("signature verification failed"));

    let _ = fs::remove_dir_all(root);
}
