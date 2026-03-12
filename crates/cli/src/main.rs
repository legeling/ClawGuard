use openclaw_guard_core::{
    default_ruleset_text, harden_config_file, load_config, load_ruleset, render_report_html,
    render_report_json, sample_config, scan_config, scan_config_with_rules, scan_profile_dir,
    scan_profile_with_rules,
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

    let config = load_config(&PathBuf::from(config_path))?;
    let report = if let Some(path) = rules_path {
        let rules = load_ruleset(&PathBuf::from(path))?;
        scan_config_with_rules(&config, &rules)
    } else {
        scan_config(&config)
    };
    let rendered = match format.as_str() {
        "json" => render_report_json(&report),
        "html" => render_report_html(&report),
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

    let report = if let Some(path) = rules_path {
        let rules = load_ruleset(&PathBuf::from(path))?;
        scan_profile_with_rules(&PathBuf::from(profile_path), &rules)?
    } else {
        scan_profile_dir(&PathBuf::from(profile_path))?
    };

    let rendered = match format.as_str() {
        "json" => render_report_json(&report),
        "html" => render_report_html(&report),
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

    if output.is_none() && !in_place {
        return Err("either --output or --in-place must be supplied".to_string());
    }

    let outcome = harden_config_file(
        &PathBuf::from(config_path),
        output.as_ref().map(PathBuf::from).as_deref(),
        in_place,
    )?;

    println!("hardening completed");
    println!("before_score={}", outcome.before_score);
    println!("after_score={}", outcome.after_score);
    println!("output_path={}", outcome.output_path.display());

    if let Some(backup_path) = outcome.backup_path {
        println!("backup_path={}", backup_path.display());
    }

    for action in outcome.applied_actions {
        println!("applied={action}");
    }
    for action in outcome.manual_actions {
        println!("manual={action}");
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
    println!("OpenClaw Guard CLI");
    println!();
    println!("Commands:");
    println!("  scan --config <path> [--format json|html] [--output <path>]");
    println!("  scan-profile --path <dir> [--format json|html] [--output <path>]");
    println!("  harden --config <path> (--output <path> | --in-place)");
    println!("  sample-config --output <path>");
    println!("  sample-rules --output <path>");
}
