use clawguard_core::{
    render_report_html_with_locale, render_report_text_with_locale, scan_config, Locale,
    OpenClawConfig,
};

#[test]
fn chinese_locale_renders_localized_report_text() {
    let config = OpenClawConfig::parse(
        "profile_name=edge\nbind_address=0.0.0.0\nport=18789\ntls_enabled=false\nauth_token=changeme\nsource_allowlist=\nwebhook_enabled=true\nwebhook_public_key=\napproval_preview_consistent=false\ncommand_allowlist_normalized=false\ndebug_enabled=true\nskills_status_exposes_secrets=true\nsuspicious_skills=wallet-sync\ninstaller_origin=community-fork",
    )
    .expect("config should parse");
    let report = scan_config(&config);

    let text = render_report_text_with_locale(&report, Locale::ZhCn);
    let html = render_report_html_with_locale(&report, Locale::ZhCn);

    assert!(text.contains("风险评分"));
    assert!(text.contains("发现项"));
    assert!(html.contains("Clawguard 安全报告"));
    assert!(html.contains("风险评分"));
}
