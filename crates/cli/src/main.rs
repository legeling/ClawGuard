use clawguard_core::{
    activate_rules_pack, create_signed_rules_pack, default_rules_store_dir,
    default_ruleset_text, generate_signing_keypair_hex, harden_config_file, import_rules_pack,
    load_active_ruleset, load_config, load_ruleset, render_report_html_with_locale,
    render_report_json, render_report_text_with_locale, rollback_rules_pack, rules_store_status,
    sample_config, scan_config, scan_config_with_rules, scan_profile_dir, scan_profile_with_rules,
    write_rules_pack, Locale, Ruleset,
};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let locale = parse_locale(&args)?;
    let Some(command) = args.first().map(String::as_str) else {
        print_usage(locale);
        return Ok(());
    };

    match command {
        "check" => run_check(&args[1..]),
        "fix" => run_fix(&args[1..]),
        "remove" => run_remove(&args[1..]),
        "scan" => run_scan(&args[1..]),
        "scan-profile" => run_scan_profile(&args[1..]),
        "harden" => run_harden(&args[1..]),
        "uninstall" => run_uninstall(&args[1..]),
        "sample-config" => run_sample_config(&args[1..]),
        "sample-rules" => run_sample_rules(&args[1..]),
        "generate-signing-keypair" => run_generate_signing_keypair(&args[1..]),
        "sign-rules-pack" => run_sign_rules_pack(&args[1..]),
        "import-rules-pack" => run_import_rules_pack(&args[1..]),
        "activate-rules" => run_activate_rules(&args[1..]),
        "rollback-rules" => run_rollback_rules(&args[1..]),
        "rules-status" => run_rules_status(&args[1..]),
        "help" | "--help" | "-h" => {
            print_usage(locale);
            Ok(())
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

fn run_scan(args: &[String]) -> Result<(), String> {
    let config_path = required_flag(args, "--config")?;
    let format = optional_flag(args, "--format").unwrap_or_else(|| "json".to_string());
    let output = optional_flag(args, "--output");
    let locale = parse_locale(args)?;

    let config = load_config(&PathBuf::from(config_path))?;
    let report = if let Some(rules) = resolve_ruleset_for_scan(args)? {
        scan_config_with_rules(&config, &rules)
    } else {
        scan_config(&config)
    };
    let rendered = render_report(&report, &format, locale)?;

    if let Some(path) = output {
        fs::write(&path, rendered).map_err(|error| format!("failed to write report {path}: {error}"))?;
        println!("report written to {path}");
    } else {
        println!("{rendered}");
    }

    Ok(())
}

fn run_check(args: &[String]) -> Result<(), String> {
    let format = optional_flag(args, "--format").unwrap_or_else(|| "text".to_string());
    let output = optional_flag(args, "--output");
    let locale = parse_locale(args)?;
    let rules = resolve_ruleset_for_scan(args)?;
    let (profile_path, config, report) = if let Some(config_path) = optional_flag(args, "--config") {
        let config_path = PathBuf::from(config_path);
        let config = load_config(&config_path)?;
        let report = if let Some(rules) = rules.as_ref() {
            scan_config_with_rules(&config, rules)
        } else {
            scan_config(&config)
        };
        (config_path, config, report)
    } else {
        let profile_dir = resolve_profile_dir(args)?;
        let config = load_config(&profile_dir.join("openclaw.conf"))?;
        let report = if let Some(rules) = rules.as_ref() {
            scan_profile_with_rules(&profile_dir, rules)?
        } else {
            scan_profile_dir(&profile_dir)?
        };
        (profile_dir, config, report)
    };

    let rendered = render_report(&report, &format, locale)?;
    if let Some(path) = output {
        fs::write(&path, rendered).map_err(|error| format!("failed to write report {path}: {error}"))?;
        println!("report written to {path}");
    } else {
        println!("profile_path={}", profile_path.display());
        println!(
            "local_probe={}",
            probe_local_service(&config.bind_address, config.port)
        );
        println!("{rendered}");
    }

    Ok(())
}

fn run_scan_profile(args: &[String]) -> Result<(), String> {
    let profile_path = required_flag(args, "--path")?;
    let format = optional_flag(args, "--format").unwrap_or_else(|| "json".to_string());
    let output = optional_flag(args, "--output");
    let locale = parse_locale(args)?;

    let report = if let Some(rules) = resolve_ruleset_for_scan(args)? {
        scan_profile_with_rules(&PathBuf::from(profile_path), &rules)?
    } else {
        scan_profile_dir(&PathBuf::from(profile_path))?
    };

    let rendered = render_report(&report, &format, locale)?;

    if let Some(path) = output {
        fs::write(&path, rendered).map_err(|error| format!("failed to write report {path}: {error}"))?;
        println!("report written to {path}");
    } else {
        println!("{rendered}");
    }

    Ok(())
}

fn run_fix(args: &[String]) -> Result<(), String> {
    let locale = parse_locale(args)?;
    let config_path = resolve_config_path(args)?;
    let output = optional_flag(args, "--output").map(PathBuf::from);
    let in_place = output.is_none() || args.iter().any(|arg| arg == "--in-place");
    let assume_yes = args.iter().any(|arg| arg == "--yes");

    if !assume_yes {
        prompt_for_confirmation(&format!(
            "Apply hardening to {}{}",
            config_path.display(),
            if in_place {
                " in place with a backup"
            } else {
                " and write the result to a separate file"
            }
        ))?;
    }

    let outcome = harden_config_file(&config_path, output.as_deref(), in_place)?;
    let text = locale_text(locale);
    println!("{}", text.hardening_completed);
    println!("{}={}", text.before_score, outcome.before_score);
    println!("{}={}", text.after_score, outcome.after_score);
    println!("{}={}", text.output_path, outcome.output_path.display());

    if let Some(backup_path) = outcome.backup_path {
        println!("{}={}", text.backup_path, backup_path.display());
    }

    for action in outcome.applied_actions {
        println!("{}={action}", text.applied);
    }
    for action in outcome.manual_actions {
        println!("{}={action}", text.manual);
    }

    Ok(())
}

fn run_harden(args: &[String]) -> Result<(), String> {
    let config_path = required_flag(args, "--config")?;
    let output = optional_flag(args, "--output");
    let in_place = args.iter().any(|arg| arg == "--in-place");
    let locale = parse_locale(args)?;

    if output.is_none() && !in_place {
        return Err("either --output or --in-place must be supplied".to_string());
    }

    let outcome = harden_config_file(
        &PathBuf::from(config_path),
        output.as_ref().map(PathBuf::from).as_deref(),
        in_place,
    )?;

    let text = locale_text(locale);
    println!("{}", text.hardening_completed);
    println!("{}={}", text.before_score, outcome.before_score);
    println!("{}={}", text.after_score, outcome.after_score);
    println!("{}={}", text.output_path, outcome.output_path.display());

    if let Some(backup_path) = outcome.backup_path {
        println!("{}={}", text.backup_path, backup_path.display());
    }

    for action in outcome.applied_actions {
        println!("{}={action}", text.applied);
    }
    for action in outcome.manual_actions {
        println!("{}={action}", text.manual);
    }

    Ok(())
}

fn run_sample_config(args: &[String]) -> Result<(), String> {
    let output = required_flag(args, "--output")?;
    fs::write(&output, sample_config())
        .map_err(|error| format!("failed to write sample config {output}: {error}"))?;
    println!("sample config written to {output}");
    Ok(())
}

fn run_sample_rules(args: &[String]) -> Result<(), String> {
    let output = required_flag(args, "--output")?;
    fs::write(&output, default_ruleset_text())
        .map_err(|error| format!("failed to write sample rules {output}: {error}"))?;
    println!("sample rules written to {output}");
    Ok(())
}

fn run_generate_signing_keypair(args: &[String]) -> Result<(), String> {
    let output_dir = PathBuf::from(required_flag(args, "--output-dir")?);
    let prefix = optional_flag(args, "--prefix").unwrap_or_else(|| "clawguard-rules".to_string());
    let private_key_path = output_dir.join(format!("{prefix}.private.key"));
    let public_key_path = output_dir.join(format!("{prefix}.public.key"));
    let (private_key, public_key) = generate_signing_keypair_hex();

    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;
    fs::write(&private_key_path, format!("{private_key}\n"))
        .map_err(|error| format!("failed to write {}: {error}", private_key_path.display()))?;
    fs::write(&public_key_path, format!("{public_key}\n"))
        .map_err(|error| format!("failed to write {}: {error}", public_key_path.display()))?;

    println!("private_key_path={}", private_key_path.display());
    println!("public_key_path={}", public_key_path.display());
    Ok(())
}

fn run_sign_rules_pack(args: &[String]) -> Result<(), String> {
    let output_path = PathBuf::from(required_flag(args, "--output")?);
    let pack_version = required_flag(args, "--version")?;
    let private_key_path = PathBuf::from(required_flag(args, "--private-key")?);
    let key_id = optional_flag(args, "--key-id").unwrap_or_else(|| "local-dev".to_string());
    let rules = if let Some(path) = optional_flag(args, "--rules") {
        load_ruleset(&PathBuf::from(path))?
    } else {
        Ruleset::default()
    };
    let private_key = read_trimmed_file(&private_key_path)?;
    let pack = create_signed_rules_pack(rules, &pack_version, &key_id, &private_key)?;

    write_rules_pack(&output_path, &pack)?;
    println!("rules_pack_path={}", output_path.display());
    println!("pack_version={}", pack.payload.pack_version);
    println!("key_id={}", pack.key_id);
    Ok(())
}

fn run_import_rules_pack(args: &[String]) -> Result<(), String> {
    let pack_path = PathBuf::from(required_flag(args, "--pack")?);
    let public_key_path = PathBuf::from(required_flag(args, "--public-key")?);
    let store_dir = optional_flag(args, "--store")
        .map(PathBuf::from)
        .unwrap_or_else(default_rules_store_dir);
    let activate_after_import = args.iter().any(|arg| arg == "--activate");
    let public_key = read_trimmed_file(&public_key_path)?;
    let imported = import_rules_pack(&pack_path, &public_key, &store_dir)?;

    println!("rules_store={}", store_dir.display());
    println!("imported_version={}", imported.version);
    println!("key_id={}", imported.key_id);
    println!("stored_pack_path={}", imported.path.display());

    if activate_after_import {
        let outcome = activate_rules_pack(&store_dir, &imported.version)?;
        println!("active_version={}", outcome.active_version);
        if let Some(previous_version) = outcome.previous_version {
            println!("previous_version={previous_version}");
        }
    }

    Ok(())
}

fn run_activate_rules(args: &[String]) -> Result<(), String> {
    let version = required_flag(args, "--version")?;
    let store_dir = optional_flag(args, "--store")
        .map(PathBuf::from)
        .unwrap_or_else(default_rules_store_dir);
    let outcome = activate_rules_pack(&store_dir, &version)?;

    println!("rules_store={}", store_dir.display());
    println!("active_version={}", outcome.active_version);
    if let Some(previous_version) = outcome.previous_version {
        println!("previous_version={previous_version}");
    }
    Ok(())
}

fn run_rollback_rules(args: &[String]) -> Result<(), String> {
    let store_dir = optional_flag(args, "--store")
        .map(PathBuf::from)
        .unwrap_or_else(default_rules_store_dir);
    let outcome = rollback_rules_pack(&store_dir)?;

    println!("rules_store={}", store_dir.display());
    println!("active_version={}", outcome.active_version);
    if let Some(previous_version) = outcome.previous_version {
        println!("previous_version={previous_version}");
    }
    Ok(())
}

fn run_rules_status(args: &[String]) -> Result<(), String> {
    let store_dir = optional_flag(args, "--store")
        .map(PathBuf::from)
        .unwrap_or_else(default_rules_store_dir);
    let status = rules_store_status(&store_dir)?;

    println!("rules_store={}", store_dir.display());
    println!(
        "active_version={}",
        status.active_version.unwrap_or_else(|| "none".to_string())
    );
    println!(
        "installed_versions={}",
        join_or_none(&status.installed_versions)
    );
    println!(
        "activation_history={}",
        join_or_none(&status.activation_history)
    );
    Ok(())
}

fn run_remove(args: &[String]) -> Result<(), String> {
    let locale = parse_locale(args)?;
    let text = locale_text(locale);
    let assume_yes = args.iter().any(|arg| arg == "--yes");
    let install_dir = resolve_install_dir(args)?;
    let binary_path = install_dir.join(binary_name());

    if !assume_yes {
        prompt_for_confirmation(&format!("Remove {}", binary_path.display()))?;
    }

    fs::remove_file(&binary_path)
        .map_err(|error| format!("failed to remove {}: {error}", binary_path.display()))?;
    println!("{}={}", text.removed_path, binary_path.display());
    Ok(())
}

fn run_uninstall(args: &[String]) -> Result<(), String> {
    let locale = parse_locale(args)?;
    let text = locale_text(locale);
    let install_dir = optional_flag(args, "--install-dir")
        .map(PathBuf::from)
        .unwrap_or_else(default_install_dir);
    let binary_path = install_dir.join(binary_name());

    if !binary_path.exists() {
        return Err(format!("{} {}", text.not_found_prefix, binary_path.display()));
    }

    fs::remove_file(&binary_path)
        .map_err(|error| format!("failed to remove {}: {error}", binary_path.display()))?;
    println!("{}={}", text.removed_path, binary_path.display());
    Ok(())
}

fn resolve_profile_dir(args: &[String]) -> Result<PathBuf, String> {
    if let Some(path) = optional_flag(args, "--path") {
        let profile_dir = PathBuf::from(path);
        if profile_dir.join("openclaw.conf").exists() {
            return Ok(profile_dir);
        }

        return Err(format!(
            "no openclaw.conf was found under {}",
            profile_dir.display()
        ));
    }

    discover_profile_dir()
}

fn resolve_config_path(args: &[String]) -> Result<PathBuf, String> {
    if let Some(path) = optional_flag(args, "--config") {
        return Ok(PathBuf::from(path));
    }

    if let Some(path) = optional_flag(args, "--path") {
        let config_path = PathBuf::from(path).join("openclaw.conf");
        if config_path.exists() {
            return Ok(config_path);
        }
    }

    Ok(discover_profile_dir()?.join("openclaw.conf"))
}

fn resolve_ruleset_for_scan(args: &[String]) -> Result<Option<Ruleset>, String> {
    if let Some(path) = optional_flag(args, "--rules") {
        return load_ruleset(&PathBuf::from(path)).map(Some);
    }

    if let Some(path) = optional_flag(args, "--rules-store") {
        let store_dir = PathBuf::from(path);
        let Some(rules) = load_active_ruleset(&store_dir)? else {
            return Err(format!(
                "no active rules pack is available in {}",
                store_dir.display()
            ));
        };
        return Ok(Some(rules));
    }

    Ok(None)
}

fn discover_profile_dir() -> Result<PathBuf, String> {
    discover_profile_under_roots(profile_search_candidates(), 3)
}

fn profile_search_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(current_dir) = env::current_dir() {
        candidates.push(current_dir);
    }

    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        candidates.push(home.join(".openclaw"));
        candidates.push(home.join("openclaw"));
    }

    if let Some(home) = env::var_os("USERPROFILE").map(PathBuf::from) {
        candidates.push(home.join(".openclaw"));
        candidates.push(home.join("openclaw"));
    }

    unique_paths(candidates)
}

fn probe_local_service(bind_address: &str, port: u16) -> String {
    if let Ok(value) = env::var("CLAWGUARD_TEST_PROBE_RESULT") {
        return value;
    }

    let timeout = Duration::from_millis(250);
    for address in local_probe_candidates(bind_address, port) {
        if TcpStream::connect_timeout(&address, timeout).is_ok() {
            return "reachable".to_string();
        }
    }

    "unreachable".to_string()
}

fn local_probe_candidates(bind_address: &str, port: u16) -> Vec<SocketAddr> {
    let mut candidates = Vec::new();

    for candidate in probe_address_strings(bind_address, port) {
        if let Ok(addresses) = candidate.to_socket_addrs() {
            for address in addresses {
                if !candidates.iter().any(|existing| existing == &address) {
                    candidates.push(address);
                }
            }
        }
    }

    candidates
}

fn probe_address_strings(bind_address: &str, port: u16) -> Vec<String> {
    match bind_address {
        "127.0.0.1" | "localhost" => vec![format!("127.0.0.1:{port}")],
        "::1" => vec![format!("[::1]:{port}")],
        "0.0.0.0" => vec![format!("127.0.0.1:{port}")],
        "::" => vec![format!("[::1]:{port}"), format!("127.0.0.1:{port}")],
        other if other.contains(':') => vec![format!("[{other}]:{port}")],
        other => vec![format!("{other}:{port}")],
    }
}

fn resolve_install_dir(args: &[String]) -> Result<PathBuf, String> {
    if let Some(path) = optional_flag(args, "--install-dir") {
        let install_dir = PathBuf::from(path);
        if install_dir.join(binary_name()).exists() {
            return Ok(install_dir);
        }

        return Err(format!(
            "binary not found at {}",
            install_dir.join(binary_name()).display()
        ));
    }

    for candidate in install_search_candidates() {
        if candidate.join(binary_name()).exists() {
            return Ok(candidate);
        }
    }

    Err("could not auto-detect an installed clawguard binary; use --install-dir <path>".to_string())
}

fn install_search_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = env::var_os("CLAWGUARD_INSTALL_DIR").map(PathBuf::from) {
        candidates.push(path);
    }

    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        candidates.push(home.join(".local/bin"));
        candidates.push(home.join(".cargo/bin"));
    }

    if let Some(home) = env::var_os("USERPROFILE").map(PathBuf::from) {
        candidates.push(home.join(".local/bin"));
        candidates.push(home.join(".cargo/bin"));
    }

    unique_paths(candidates)
}

fn unique_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique = Vec::new();
    for path in paths {
        if !unique.iter().any(|existing| existing == &path) {
            unique.push(path);
        }
    }
    unique
}

fn discover_profile_under_roots(roots: Vec<PathBuf>, max_depth: usize) -> Result<PathBuf, String> {
    for root in unique_paths(roots) {
        if let Some(path) = search_profile_dir(&root, max_depth)? {
            return Ok(path);
        }
    }

    Err(
        "could not auto-discover an OpenClaw profile; use --path <dir> or --config <path>"
            .to_string(),
    )
}

fn search_profile_dir(root: &Path, depth: usize) -> Result<Option<PathBuf>, String> {
    if root.join("openclaw.conf").exists() {
        return Ok(Some(root.to_path_buf()));
    }

    if depth == 0 || !root.is_dir() {
        return Ok(None);
    }

    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return Ok(None),
    };

    let mut directories = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|file_type| file_type.is_dir())
                .map(|_| entry.path())
        })
        .collect::<Vec<_>>();
    directories.sort();

    for directory in directories {
        if let Some(path) = search_profile_dir(&directory, depth - 1)? {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn prompt_for_confirmation(prompt: &str) -> Result<(), String> {
    print!("{prompt}? [y/N] ");
    io::stdout()
        .flush()
        .map_err(|error| format!("failed to flush stdout: {error}"))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|error| format!("failed to read confirmation: {error}"))?;

    if matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
        Ok(())
    } else {
        Err("operation cancelled".to_string())
    }
}

fn render_report(
    report: &clawguard_core::ScanReport,
    format: &str,
    locale: Locale,
) -> Result<String, String> {
    match format {
        "json" => Ok(render_report_json(report)),
        "html" => Ok(render_report_html_with_locale(report, locale)),
        "text" => Ok(render_report_text_with_locale(report, locale)),
        _ => Err(format!("unsupported format: {format}")),
    }
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(",")
    }
}

fn read_trimmed_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))
        .map(|content| content.trim().to_string())
}

fn required_flag(args: &[String], flag: &str) -> Result<String, String> {
    optional_flag(args, flag).ok_or_else(|| format!("missing required flag {flag}"))
}

fn optional_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1).cloned())
}

fn print_usage(locale: Locale) {
    let text = locale_text(locale);
    println!("{}", banner());
    println!();
    println!("{}", text.tagline);
    println!();
    println!("ClawGuard CLI");
    println!();
    println!("Commands:");
    println!("  check [--path <dir> | --config <path>] [--rules <path> | --rules-store <path>] [--format json|html|text] [--locale en|zh-CN] [--output <path>]");
    println!("  fix [--path <dir> | --config <path>] [--locale en|zh-CN] [--output <path> | --in-place] [--yes]");
    println!("  remove [--locale en|zh-CN] [--install-dir <path>] [--yes]");
    println!("  scan --config <path> [--rules <path> | --rules-store <path>] [--format json|html|text] [--locale en|zh-CN] [--output <path>]");
    println!("  scan-profile --path <dir> [--rules <path> | --rules-store <path>] [--format json|html|text] [--locale en|zh-CN] [--output <path>]");
    println!("  harden --config <path> [--locale en|zh-CN] (--output <path> | --in-place)");
    println!("  sample-config --output <path>");
    println!("  sample-rules --output <path>");
    println!("  generate-signing-keypair --output-dir <path> [--prefix <name>]");
    println!("  sign-rules-pack --output <path> --version <version> --private-key <path> [--rules <path>] [--key-id <id>]");
    println!("  import-rules-pack --pack <path> --public-key <path> [--store <path>] [--activate]");
    println!("  activate-rules --version <version> [--store <path>]");
    println!("  rollback-rules [--store <path>]");
    println!("  rules-status [--store <path>]");
    println!("  uninstall [--locale en|zh-CN] [--install-dir <path>]");
}

fn parse_locale(args: &[String]) -> Result<Locale, String> {
    match optional_flag(args, "--locale") {
        Some(value) => {
            Locale::parse(&value).ok_or_else(|| format!("unsupported locale: {value}"))
        }
        None => Ok(detect_locale_from_env()),
    }
}

struct CliLocaleText {
    tagline: &'static str,
    hardening_completed: &'static str,
    before_score: &'static str,
    after_score: &'static str,
    output_path: &'static str,
    backup_path: &'static str,
    applied: &'static str,
    manual: &'static str,
    removed_path: &'static str,
    not_found_prefix: &'static str,
}

fn locale_text(locale: Locale) -> CliLocaleText {
    match locale {
        Locale::En => CliLocaleText {
            tagline: "XiaoLongXia Guard | OpenClaw Security Audit and Hardening CLI",
            hardening_completed: "hardening completed",
            before_score: "before_score",
            after_score: "after_score",
            output_path: "output_path",
            backup_path: "backup_path",
            applied: "applied",
            manual: "manual",
            removed_path: "removed_path",
            not_found_prefix: "binary not found at",
        },
        Locale::ZhCn => CliLocaleText {
            tagline: "小龙虾卫士 | OpenClaw 安全审计与加固 CLI",
            hardening_completed: "加固完成",
            before_score: "加固前评分",
            after_score: "加固后评分",
            output_path: "输出路径",
            backup_path: "备份路径",
            applied: "已执行",
            manual: "需人工处理",
            removed_path: "已移除路径",
            not_found_prefix: "未找到可卸载的二进制文件：",
        },
    }
}

fn detect_locale_from_env() -> Locale {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|key| env::var(key).ok())
        .find_map(|value| {
            let normalized = value.to_ascii_lowercase();
            if normalized.starts_with("zh") {
                Some(Locale::ZhCn)
            } else if normalized.starts_with("en") {
                Some(Locale::En)
            } else {
                None
            }
        })
        .unwrap_or(Locale::En)
}

fn default_install_dir() -> PathBuf {
    env::var_os("CLAWGUARD_INSTALL_DIR")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/bin")))
        .or_else(|| {
            env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join(".local/bin"))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}

fn binary_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "clawguard.exe"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "clawguard"
    }
}

fn banner() -> &'static str {
    r"  ______ _                 _____                     _ 
 / ____| |               / ____|                   | |
| |    | | __ ___      _| |  __ _   _  __ _ _ __ __| |
| |    | |/ _` \ \ /\ / / | |_ | | | |/ _` | '__/ _` |
| |____| | (_| |\ V  V /| |__| | |_| | (_| | | | (_| |
 \_____|_|\__,_| \_/\_/  \_____|\__,_|\__,_|_|  \__,_|"
}
