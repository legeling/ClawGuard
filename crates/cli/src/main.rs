use clawguard_core::{
    activate_rules_pack, create_signed_rules_pack, default_rules_store_dir,
    default_ruleset_text, generate_signing_keypair_hex, harden_config_file, import_rules_pack,
    load_active_ruleset, load_config, load_ruleset, render_report_html_with_locale,
    render_report_json, render_report_text_with_locale, rollback_rules_pack, rules_store_status,
    sample_config, scan_config, scan_config_with_rules, scan_profile_dir, scan_profile_with_rules,
    write_rules_pack, Locale, OpenClawConfig, Ruleset, ScanReport,
};
use dialoguer::{
    console::{style, Style},
    theme::ColorfulTheme,
    Confirm, Input, MultiSelect,
};
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
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
    let command_args = strip_global_flags(&args);
    let Some(command) = command_args.first().map(String::as_str) else {
        return run_interactive(locale);
    };

    match command {
        "interactive" => run_interactive(locale),
        "check" => run_check(&command_args[1..], locale),
        "fix" => run_fix(&command_args[1..], locale),
        "remove" => run_remove(&command_args[1..], locale),
        "scan" => run_scan(&command_args[1..], locale),
        "scan-profile" => run_scan_profile(&command_args[1..], locale),
        "harden" => run_harden(&command_args[1..], locale),
        "uninstall" => run_uninstall(&command_args[1..], locale),
        "sample-config" => run_sample_config(&command_args[1..], locale),
        "sample-rules" => run_sample_rules(&command_args[1..], locale),
        "generate-signing-keypair" => run_generate_signing_keypair(&command_args[1..]),
        "sign-rules-pack" => run_sign_rules_pack(&command_args[1..]),
        "import-rules-pack" => run_import_rules_pack(&command_args[1..]),
        "activate-rules" => run_activate_rules(&command_args[1..]),
        "rollback-rules" => run_rollback_rules(&command_args[1..]),
        "rules-status" => run_rules_status(&command_args[1..]),
        "help" | "--help" | "-h" => {
            print_usage(locale);
            Ok(())
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

fn strip_global_flags(args: &[String]) -> Vec<String> {
    let mut filtered = Vec::new();
    let mut index = 0;

    while index < args.len() {
        if args[index] == "--locale" {
            index += 1;
            if index < args.len() {
                index += 1;
            }
            continue;
        }

        filtered.push(args[index].clone());
        index += 1;
    }

    filtered
}

enum CheckTarget {
    Config(PathBuf),
    Profile(PathBuf),
}

fn run_scan(args: &[String], locale: Locale) -> Result<(), String> {
    let config_path = required_flag(args, "--config")?;
    let format = optional_flag(args, "--format").unwrap_or_else(|| "json".to_string());
    let output = optional_flag(args, "--output");

    let config = load_config(&PathBuf::from(config_path))?;
    let report = if let Some(rules) = resolve_ruleset_for_scan(args)? {
        scan_config_with_rules(&config, &rules)
    } else {
        scan_config(&config)
    };
    let rendered = render_report(&report, &format, locale)?;

    if let Some(path) = output {
        fs::write(&path, rendered).map_err(|error| format!("failed to write report {path}: {error}"))?;
        println!("{}", locale_text(locale).report_written(&path));
    } else {
        println!("{rendered}");
    }

    Ok(())
}

fn run_check(args: &[String], locale: Locale) -> Result<(), String> {
    let format = optional_flag(args, "--format").unwrap_or_else(|| "text".to_string());
    let output = optional_flag(args, "--output");
    let rules = resolve_ruleset_for_scan(args)?;
    let target = resolve_check_target(args)?;
    execute_check(target, rules, format, output, locale)
}

fn run_scan_profile(args: &[String], locale: Locale) -> Result<(), String> {
    let profile_path = required_flag(args, "--path")?;
    let format = optional_flag(args, "--format").unwrap_or_else(|| "json".to_string());
    let output = optional_flag(args, "--output");

    let report = if let Some(rules) = resolve_ruleset_for_scan(args)? {
        scan_profile_with_rules(&PathBuf::from(profile_path), &rules)?
    } else {
        scan_profile_dir(&PathBuf::from(profile_path))?
    };

    let rendered = render_report(&report, &format, locale)?;

    if let Some(path) = output {
        fs::write(&path, rendered).map_err(|error| format!("failed to write report {path}: {error}"))?;
        println!("{}", locale_text(locale).report_written(&path));
    } else {
        println!("{rendered}");
    }

    Ok(())
}

fn run_fix(args: &[String], locale: Locale) -> Result<(), String> {
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

fn run_harden(args: &[String], locale: Locale) -> Result<(), String> {
    let config_path = required_flag(args, "--config")?;
    let output = optional_flag(args, "--output");
    let in_place = args.iter().any(|arg| arg == "--in-place");

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

fn run_sample_config(args: &[String], locale: Locale) -> Result<(), String> {
    let output = required_flag(args, "--output")?;
    fs::write(&output, sample_config())
        .map_err(|error| format!("failed to write sample config {output}: {error}"))?;
    println!("{}", locale_text(locale).sample_config_written(&output));
    Ok(())
}

fn run_sample_rules(args: &[String], locale: Locale) -> Result<(), String> {
    let output = required_flag(args, "--output")?;
    fs::write(&output, default_ruleset_text())
        .map_err(|error| format!("failed to write sample rules {output}: {error}"))?;
    println!("{}", locale_text(locale).sample_rules_written(&output));
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

fn run_remove(args: &[String], locale: Locale) -> Result<(), String> {
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

fn run_interactive(locale: Locale) -> Result<(), String> {
    if supports_rich_interaction() {
        return run_rich_interactive(locale);
    }

    run_text_interactive(locale)
}

fn run_rich_interactive(locale: Locale) -> Result<(), String> {
    let text = locale_text(locale);
    println!("{}", style(banner()).color256(208));
    println!();
    println!(
        "{}",
        style(text.interactive_mode_title)
            .bold()
            .color256(208)
    );
    println!("{}", style(text.tagline).bold());
    println!(
        "{}",
        style(text.interactive_instructions).color256(214)
    );
    println!();

    loop {
        let selections = MultiSelect::with_theme(&interactive_theme())
            .with_prompt(text.quick_actions)
            .items([
                text.action_check,
                text.action_fix,
                text.action_remove,
                text.action_sample,
                text.action_exit,
            ])
            .interact()
            .map_err(|error| format!("interactive menu failed: {error}"))?;

        if selections.is_empty() {
            if Confirm::with_theme(&interactive_theme())
                .with_prompt(text.no_action_selected_prompt)
                .default(true)
                .interact()
                .map_err(|error| format!("interactive confirmation failed: {error}"))?
            {
                return Ok(());
            }
            continue;
        }

        let exit_after_run = selections.contains(&4);
        for selection in selections {
            match selection {
                0 => interactive_check(locale)?,
                1 => interactive_fix(locale)?,
                2 => interactive_remove(locale)?,
                3 => interactive_sample_config(locale)?,
                4 => {}
                _ => {}
            }
        }

        if exit_after_run {
            return Ok(());
        }

        println!("\n{}\n", text.completed_selected_actions);
    }
}

fn run_text_interactive(locale: Locale) -> Result<(), String> {
    let text = locale_text(locale);
    println!("{}", banner());
    println!();
    println!("{}", text.interactive_mode_plain_title);
    println!("{}", text.tagline);
    println!();

    loop {
        println!("1. {}", text.action_check_plain);
        println!("2. {}", text.action_fix_plain);
        println!("3. {}", text.action_remove_plain);
        println!("4. {}", text.action_sample_plain);
        println!("5. {}", text.action_exit_plain);

        match prompt_line(text.choose_action_prompt)?.trim() {
            "1" => interactive_check(locale)?,
            "2" => interactive_fix(locale)?,
            "3" => interactive_remove(locale)?,
            "4" => interactive_sample_config(locale)?,
            "5" | "" => return Ok(()),
            _ => println!("{}", text.unknown_choice),
        }

        println!();
    }
}

fn interactive_check(locale: Locale) -> Result<(), String> {
    let target = match resolve_check_target(&[]) {
        Ok(target) => target,
        Err(_) => match interactive_target_prompt(locale)? {
            Some(target) => target,
            None => return Ok(()),
        },
    };

    execute_check(target, None, "text".to_string(), None, locale)
}

fn interactive_fix(locale: Locale) -> Result<(), String> {
    let config_path = match resolve_config_path(&[]) {
        Ok(path) => path,
        Err(_) => match interactive_target_prompt(locale)? {
            Some(CheckTarget::Config(path)) => path,
            Some(CheckTarget::Profile(path)) => path.join("openclaw.conf"),
            None => return Ok(()),
        },
    };

    let outcome = execute_fix(&config_path, None, true, locale, false)?;
    println!("{}", locale_text(locale).fixed_path(&outcome.display().to_string()));
    Ok(())
}

fn interactive_remove(locale: Locale) -> Result<(), String> {
    let install_dir = match resolve_install_dir(&[]) {
        Ok(path) => path,
        Err(_) => match prompt_optional_input(
            locale_text(locale).remove_install_dir_prompt,
            locale,
        )? {
            Some(value) => PathBuf::from(value),
            None => return Ok(()),
        },
    };

    execute_remove(&install_dir, locale, false)
}

fn interactive_sample_config(locale: Locale) -> Result<(), String> {
    let output = match prompt_optional_input(locale_text(locale).sample_config_prompt, locale)? {
        None => env::current_dir()
            .map_err(|error| format!("failed to resolve current directory: {error}"))?
            .join("openclaw.conf"),
        Some(value) => PathBuf::from(value),
    };

    fs::write(&output, sample_config())
        .map_err(|error| format!("failed to write sample config {}: {error}", output.display()))?;
    println!(
        "{}",
        locale_text(locale).sample_config_written(&output.display().to_string())
    );
    Ok(())
}

fn run_uninstall(args: &[String], locale: Locale) -> Result<(), String> {
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

fn resolve_check_target(args: &[String]) -> Result<CheckTarget, String> {
    if let Some(config_path) = optional_flag(args, "--config") {
        return Ok(CheckTarget::Config(PathBuf::from(config_path)));
    }

    resolve_profile_dir(args).map(CheckTarget::Profile)
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

fn execute_check(
    target: CheckTarget,
    rules: Option<Ruleset>,
    format: String,
    output: Option<String>,
    locale: Locale,
) -> Result<(), String> {
    let (profile_path, config, report) = load_check_execution(target, rules.as_ref())?;
    let rendered = render_report(&report, &format, locale)?;

    if let Some(path) = output {
        fs::write(&path, rendered)
            .map_err(|error| format!("failed to write report {path}: {error}"))?;
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

fn load_check_execution(
    target: CheckTarget,
    rules: Option<&Ruleset>,
) -> Result<(PathBuf, OpenClawConfig, ScanReport), String> {
    match target {
        CheckTarget::Config(config_path) => {
            let config = load_config(&config_path)?;
            let report = if let Some(rules) = rules {
                scan_config_with_rules(&config, rules)
            } else {
                scan_config(&config)
            };
            Ok((config_path, config, report))
        }
        CheckTarget::Profile(profile_dir) => {
            let config = load_config(&profile_dir.join("openclaw.conf"))?;
            let report = if let Some(rules) = rules {
                scan_profile_with_rules(&profile_dir, rules)?
            } else {
                scan_profile_dir(&profile_dir)?
            };
            Ok((profile_dir, config, report))
        }
    }
}

fn execute_fix(
    config_path: &Path,
    output: Option<&Path>,
    in_place: bool,
    locale: Locale,
    assume_yes: bool,
) -> Result<PathBuf, String> {
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

    let outcome = harden_config_file(config_path, output, in_place)?;
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

    Ok(outcome.output_path)
}

fn execute_remove(install_dir: &Path, locale: Locale, assume_yes: bool) -> Result<(), String> {
    let text = locale_text(locale);
    let binary_path = install_dir.join(binary_name());

    if !binary_path.exists() {
        return Err(format!("{} {}", text.not_found_prefix, binary_path.display()));
    }

    if !assume_yes {
        prompt_for_confirmation(&format!("Remove {}", binary_path.display()))?;
    }

    fs::remove_file(&binary_path)
        .map_err(|error| format!("failed to remove {}: {error}", binary_path.display()))?;
    println!("{}={}", text.removed_path, binary_path.display());
    Ok(())
}

fn interactive_target_prompt(locale: Locale) -> Result<Option<CheckTarget>, String> {
    let text = locale_text(locale);
    println!("{}", text.auto_discovery_failed);
    if supports_rich_interaction() {
        let choice = dialoguer::Select::with_theme(&interactive_theme())
            .with_prompt(text.continue_prompt)
            .items([
                text.target_option_path,
                text.target_option_sample,
                text.target_option_back,
            ])
            .default(0)
            .interact()
            .map_err(|error| format!("interactive selection failed: {error}"))?;

        match choice {
            0 => {
                let path = prompt_required_input(text.target_path_prompt, locale)?;
                parse_interactive_target_path(&path).map(Some)
            }
            1 => create_sample_target_here(locale).map(Some),
            _ => Ok(None),
        }
    } else {
        let input = prompt_line(text.target_path_inline_prompt)?;
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        if trimmed.eq_ignore_ascii_case("sample") {
            return create_sample_target_here(locale).map(Some);
        }

        parse_interactive_target_path(trimmed).map(Some)
    }
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
        "could not auto-discover an OpenClaw profile. If OpenClaw is installed elsewhere, rerun with --path <dir> or --config <path>. For a local demo, run `clawguard sample-config --output openclaw.conf` first."
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
    if supports_rich_interaction() {
        return Confirm::with_theme(&interactive_theme())
            .with_prompt(prompt)
            .default(false)
            .interact()
            .map_err(|error| format!("interactive confirmation failed: {error}"))
            .and_then(|confirmed| {
                if confirmed {
                    Ok(())
                } else {
                    Err("operation cancelled".to_string())
                }
            });
    }

    let input = prompt_line(&format!("{prompt}? [y/N]"))?;

    if matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
        Ok(())
    } else {
        Err("operation cancelled".to_string())
    }
}

fn prompt_line(prompt: &str) -> Result<String, String> {
    print!("{prompt} ");
    io::stdout()
        .flush()
        .map_err(|error| format!("failed to flush stdout: {error}"))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|error| format!("failed to read input: {error}"))?;
    Ok(input)
}

fn prompt_optional_input(prompt: &str, _locale: Locale) -> Result<Option<String>, String> {
    if supports_rich_interaction() {
        let value = Input::<String>::with_theme(&interactive_theme())
            .with_prompt(prompt)
            .allow_empty(true)
            .interact_text()
            .map_err(|error| format!("interactive input failed: {error}"))?;
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed))
        }
    } else {
        let value = prompt_line(prompt)?;
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed))
        }
    }
}

fn prompt_required_input(prompt: &str, locale: Locale) -> Result<String, String> {
    if supports_rich_interaction() {
        Input::<String>::with_theme(&interactive_theme())
            .with_prompt(prompt)
            .validate_with(|value: &String| {
                if value.trim().is_empty() {
                    Err(locale_text(locale).path_required)
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map(|value| value.trim().to_string())
            .map_err(|error| format!("interactive input failed: {error}"))
    } else {
        let value = prompt_line(prompt)?;
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            Err("operation cancelled".to_string())
        } else {
            Ok(trimmed)
        }
    }
}

fn parse_interactive_target_path(input: &str) -> Result<CheckTarget, String> {
    let path = PathBuf::from(input.trim());
    if path.is_dir() {
        if path.join("openclaw.conf").exists() {
            Ok(CheckTarget::Profile(path))
        } else {
            Err(format!(
                "no openclaw.conf was found under {}",
                path.display()
            ))
        }
    } else {
        Ok(CheckTarget::Config(path))
    }
}

fn create_sample_target_here(locale: Locale) -> Result<CheckTarget, String> {
    let config_path = env::current_dir()
        .map_err(|error| format!("failed to resolve current directory: {error}"))?
        .join("openclaw.conf");
    fs::write(&config_path, sample_config()).map_err(|error| {
        format!(
            "failed to write sample config {}: {error}",
            config_path.display()
        )
    })?;
    println!(
        "{}",
        locale_text(locale).sample_config_written(&config_path.display().to_string())
    );
    Ok(CheckTarget::Config(config_path))
}

fn interactive_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_style: Style::new().for_stderr().bold().color256(208),
        prompt_prefix: style("?".to_string()).for_stderr().color256(208),
        prompt_suffix: style(">".to_string()).black().bright(),
        success_prefix: style("OK".to_string()).for_stderr().color256(208),
        success_suffix: style(">".to_string()).black().bright(),
        error_prefix: style("ERR".to_string()).red(),
        active_item_style: Style::new().for_stderr().bold().color256(208),
        active_item_prefix: style(">".to_string()).for_stderr().color256(208),
        inactive_item_prefix: style(" ".to_string()),
        checked_item_prefix: style("[x]".to_string()).for_stderr().color256(208),
        unchecked_item_prefix: style("[ ]".to_string()).magenta(),
        picked_item_prefix: style(">".to_string()).for_stderr().color256(208),
        unpicked_item_prefix: style(" ".to_string()),
        ..Default::default()
    }
}

fn supports_rich_interaction() -> bool {
    if matches!(env::var("CLAWGUARD_FORCE_TEXT_UI").as_deref(), Ok("1")) {
        return false;
    }

    io::stdin().is_terminal()
        && io::stdout().is_terminal()
        && !matches!(env::var("TERM").as_deref(), Ok("dumb"))
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
    println!("{}", text.usage_intro);
    println!();
    println!("{}", text.commands_label);
    println!("  interactive");
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
    is_chinese: bool,
    tagline: &'static str,
    interactive_mode_title: &'static str,
    interactive_mode_plain_title: &'static str,
    interactive_instructions: &'static str,
    quick_actions: &'static str,
    action_check: &'static str,
    action_fix: &'static str,
    action_remove: &'static str,
    action_sample: &'static str,
    action_exit: &'static str,
    action_check_plain: &'static str,
    action_fix_plain: &'static str,
    action_remove_plain: &'static str,
    action_sample_plain: &'static str,
    action_exit_plain: &'static str,
    no_action_selected_prompt: &'static str,
    completed_selected_actions: &'static str,
    choose_action_prompt: &'static str,
    unknown_choice: &'static str,
    auto_discovery_failed: &'static str,
    continue_prompt: &'static str,
    target_option_path: &'static str,
    target_option_sample: &'static str,
    target_option_back: &'static str,
    target_path_prompt: &'static str,
    target_path_inline_prompt: &'static str,
    path_required: &'static str,
    remove_install_dir_prompt: &'static str,
    sample_config_prompt: &'static str,
    usage_intro: &'static str,
    commands_label: &'static str,
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

impl CliLocaleText {
    fn report_written(&self, path: &str) -> String {
        if self.is_chinese {
            format!("报告已写入 {path}")
        } else {
            format!("report written to {path}")
        }
    }

    fn sample_config_written(&self, path: &str) -> String {
        if self.is_chinese {
            format!("示例配置已写入 {path}")
        } else {
            format!("sample config written to {path}")
        }
    }

    fn sample_rules_written(&self, path: &str) -> String {
        if self.is_chinese {
            format!("示例规则已写入 {path}")
        } else {
            format!("sample rules written to {path}")
        }
    }

    fn fixed_path(&self, path: &str) -> String {
        if self.is_chinese {
            format!("已完成修复 {path}")
        } else {
            format!("Fixed {path}")
        }
    }
}

fn locale_text(locale: Locale) -> CliLocaleText {
    match locale {
        Locale::En => CliLocaleText {
            is_chinese: false,
            tagline: "XiaoLongXia Guard | OpenClaw Security Audit and Hardening CLI",
            interactive_mode_title: "== ClawGuard Interactive Mode ==",
            interactive_mode_plain_title: "Interactive Mode",
            interactive_instructions: "Use arrow keys to move, Space to toggle, and Enter to run.",
            quick_actions: "Quick Actions",
            action_check: "[Check] Check this machine",
            action_fix: "[Fix] Fix local OpenClaw config",
            action_remove: "[Remove] Remove installed ClawGuard binary",
            action_sample: "[Sample] Generate sample config here",
            action_exit: "[Exit] Exit interactive mode",
            action_check_plain: "Check this machine",
            action_fix_plain: "Fix local OpenClaw config",
            action_remove_plain: "Remove installed ClawGuard binary",
            action_sample_plain: "Generate sample config here",
            action_exit_plain: "Exit",
            no_action_selected_prompt: "No action selected. Exit ClawGuard?",
            completed_selected_actions: "Completed selected action(s).",
            choose_action_prompt: "Choose an action",
            unknown_choice: "Unknown choice. Enter 1, 2, 3, 4, or 5.",
            auto_discovery_failed: "No OpenClaw profile was auto-discovered.",
            continue_prompt: "How should ClawGuard continue?",
            target_option_path: "[Path] Enter a profile directory or config path",
            target_option_sample: "[Sample] Create ./openclaw.conf from the sample template",
            target_option_back: "[Back] Return to the main menu",
            target_path_prompt: "Enter a profile directory or config path",
            target_path_inline_prompt: "Enter a profile directory or config path, type 'sample' to create ./openclaw.conf, or press Enter to cancel",
            path_required: "Please enter a path.",
            remove_install_dir_prompt: "Enter an install directory, or leave blank to cancel removal",
            sample_config_prompt: "Enter a config path, or leave blank to create ./openclaw.conf",
            usage_intro: "Run `clawguard` without arguments to start interactive mode.",
            commands_label: "Commands:",
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
            is_chinese: true,
            tagline: "小龙虾卫士 | OpenClaw 安全审计与加固 CLI",
            interactive_mode_title: "== ClawGuard 交互模式 ==",
            interactive_mode_plain_title: "交互模式",
            interactive_instructions: "使用方向键移动，空格选择，回车执行。",
            quick_actions: "快速操作",
            action_check: "[检查] 扫描本机",
            action_fix: "[修复] 加固本地 OpenClaw 配置",
            action_remove: "[卸载] 移除已安装的 ClawGuard 二进制",
            action_sample: "[示例] 在当前目录生成示例配置",
            action_exit: "[退出] 退出交互模式",
            action_check_plain: "扫描本机",
            action_fix_plain: "加固本地 OpenClaw 配置",
            action_remove_plain: "移除已安装的 ClawGuard 二进制",
            action_sample_plain: "在当前目录生成示例配置",
            action_exit_plain: "退出",
            no_action_selected_prompt: "未选择任何操作。要退出 ClawGuard 吗？",
            completed_selected_actions: "已完成所选操作。",
            choose_action_prompt: "请选择操作",
            unknown_choice: "无效选项。请输入 1、2、3、4 或 5。",
            auto_discovery_failed: "未能自动发现 OpenClaw profile。",
            continue_prompt: "ClawGuard 应如何继续？",
            target_option_path: "[路径] 输入 profile 目录或配置文件路径",
            target_option_sample: "[示例] 从模板创建 ./openclaw.conf",
            target_option_back: "[返回] 返回主菜单",
            target_path_prompt: "请输入 profile 目录或配置文件路径",
            target_path_inline_prompt: "请输入 profile 目录或配置文件路径，输入 'sample' 可创建 ./openclaw.conf，直接回车可取消",
            path_required: "请输入路径。",
            remove_install_dir_prompt: "请输入安装目录，或直接回车取消卸载",
            sample_config_prompt: "请输入配置文件路径，或直接回车创建 ./openclaw.conf",
            usage_intro: "直接运行 `clawguard` 可进入交互模式。",
            commands_label: "命令：",
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
