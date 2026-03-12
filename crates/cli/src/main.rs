use clawguard_core::{
    default_ruleset_text, harden_config_file, load_config, load_ruleset,
    render_report_html_with_locale, render_report_json, render_report_text_with_locale,
    sample_config, scan_config, scan_config_with_rules, scan_profile_dir,
    scan_profile_with_rules, Locale,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first().map(String::as_str) else {
        print_usage();
        return Ok(());
    };

    match command {
        "scan" => run_scan(&args[1..]),
        "scan-profile" => run_scan_profile(&args[1..]),
        "harden" => run_harden(&args[1..]),
        "sample-config" => run_sample_config(&args[1..]),
        "sample-rules" => run_sample_rules(&args[1..]),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

fn run_scan(args: &[String]) -> Result<(), String> {
    let config_path = required_flag(args, "--config")?;
    let format = optional_flag(args, "--format").unwrap_or_else(|| "json".to_string());
    let output = optional_flag(args, "--output");
    let rules_path = optional_flag(args, "--rules");
    let locale = parse_locale(args)?;

    let config = load_config(&PathBuf::from(config_path))?;
    let report = if let Some(path) = rules_path {
        let rules = load_ruleset(&PathBuf::from(path))?;
        scan_config_with_rules(&config, &rules)
    } else {
        scan_config(&config)
    };
    let rendered = match format.as_str() {
        "json" => render_report_json(&report),
        "html" => render_report_html_with_locale(&report, locale),
        "text" => render_report_text_with_locale(&report, locale),
        _ => return Err(format!("unsupported format: {format}")),
    };

    if let Some(path) = output {
        fs::write(&path, rendered).map_err(|error| format!("failed to write report {path}: {error}"))?;
        println!("report written to {path}");
    } else {
        println!("{rendered}");
    }

    Ok(())
}

fn run_scan_profile(args: &[String]) -> Result<(), String> {
    let profile_path = required_flag(args, "--path")?;
    let format = optional_flag(args, "--format").unwrap_or_else(|| "json".to_string());
    let output = optional_flag(args, "--output");
    let rules_path = optional_flag(args, "--rules");
    let locale = parse_locale(args)?;

    let report = if let Some(path) = rules_path {
        let rules = load_ruleset(&PathBuf::from(path))?;
        scan_profile_with_rules(&PathBuf::from(profile_path), &rules)?
    } else {
        scan_profile_dir(&PathBuf::from(profile_path))?
    };

    let rendered = match format.as_str() {
        "json" => render_report_json(&report),
        "html" => render_report_html_with_locale(&report, locale),
        "text" => render_report_text_with_locale(&report, locale),
        _ => return Err(format!("unsupported format: {format}")),
    };

    if let Some(path) = output {
        fs::write(&path, rendered).map_err(|error| format!("failed to write report {path}: {error}"))?;
        println!("report written to {path}");
    } else {
        println!("{rendered}");
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

fn required_flag(args: &[String], flag: &str) -> Result<String, String> {
    optional_flag(args, flag).ok_or_else(|| format!("missing required flag {flag}"))
}

fn optional_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1).cloned())
}

fn print_usage() {
    println!("Clawguard CLI");
    println!();
    println!("Commands:");
    println!("  scan --config <path> [--format json|html|text] [--locale en|zh-CN] [--output <path>]");
    println!("  scan-profile --path <dir> [--format json|html|text] [--locale en|zh-CN] [--output <path>]");
    println!("  harden --config <path> [--locale en|zh-CN] (--output <path> | --in-place)");
    println!("  sample-config --output <path>");
    println!("  sample-rules --output <path>");
}

fn parse_locale(args: &[String]) -> Result<Locale, String> {
    match optional_flag(args, "--locale") {
        Some(value) => {
            Locale::parse(&value).ok_or_else(|| format!("unsupported locale: {value}"))
        }
        None => Ok(Locale::En),
    }
}

struct CliLocaleText {
    hardening_completed: &'static str,
    before_score: &'static str,
    after_score: &'static str,
    output_path: &'static str,
    backup_path: &'static str,
    applied: &'static str,
    manual: &'static str,
}

fn locale_text(locale: Locale) -> CliLocaleText {
    match locale {
        Locale::En => CliLocaleText {
            hardening_completed: "hardening completed",
            before_score: "before_score",
            after_score: "after_score",
            output_path: "output_path",
            backup_path: "backup_path",
            applied: "applied",
            manual: "manual",
        },
        Locale::ZhCn => CliLocaleText {
            hardening_completed: "加固完成",
            before_score: "加固前评分",
            after_score: "加固后评分",
            output_path: "输出路径",
            backup_path: "备份路径",
            applied: "已执行",
            manual: "需人工处理",
        },
    }
}
