use clawguard_core::{scan_config_with_rules, OpenClawConfig, Ruleset};

#[test]
fn custom_ruleset_can_change_trust_and_supply_chain_detection() {
    let config = OpenClawConfig::parse(
        "profile_name=edge\nbind_address=127.0.0.1\nport=18789\ntls_enabled=true\nauth_token=abcdefghijklmnopqrstuvwxyz\nsource_allowlist=127.0.0.1/32\nwebhook_enabled=false\nwebhook_public_key=\napproval_preview_consistent=true\ncommand_allowlist_normalized=true\ndebug_enabled=false\nskills_status_exposes_secrets=false\nsuspicious_skills=ops-helper\ninstaller_origin=enterprise-mirror",
    )
    .expect("config should parse");

    let rules = Ruleset::parse(
        "trusted_origins=official,enterprise-mirror\nsuspicious_skill_patterns=wallet,exfil\nweak_tokens=default-token,changeme",
    )
    .expect("rules should parse");

    let report = scan_config_with_rules(&config, &rules);

    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.id != "SUP-0002")
    );
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.id != "SUP-0001")
    );
}
