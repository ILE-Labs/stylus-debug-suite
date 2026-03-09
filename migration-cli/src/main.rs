mod patterns;


use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use colored::*;

use patterns::detect_patterns;

/// Stylus Migration Assistant — analyze Solidity contracts using AST-based
/// transformation rules for Arbitrum Stylus (Rust).
#[derive(Parser)]
#[command(name = "stylus-migrate", version, about)]
struct Cli {
    /// Path to the Solidity source file to analyze.
    #[arg(default_value = "examples/demo-contracts/Demo.sol")]
    input: PathBuf,

    /// Output format: text or json.
    #[arg(long, default_value = "text")]
    format: String,

    /// Show verbose output including matched source lines.
    #[arg(long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Pattern detection

    let detected = detect_patterns(&cli.input);

    match cli.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&detected)?;
            println!("{json}");
        }
        _ => print_text_report(&cli, &detected),
    }

    Ok(())
}

fn print_text_report(cli: &Cli, detected: &[patterns::DetectedPattern]) {
    println!();
    println!("{}", "  ╔══════════════════════════════════════════════════╗".cyan());
    println!("  ║      {}                 ║", "Stylus Migration Assistant".bold().white());
    println!("{}", "  ╠══════════════════════════════════════════════════╣".cyan());
    println!("  ║  {}              ║", "AST-Based Solidity → Stylus Transformation".italic().bright_black());
    println!("{}", "  ╚══════════════════════════════════════════════════╝".cyan());
    println!();
    println!("  {} {}", "Note:".bold().yellow(), "This POC supports a subset of Solidity (ERC-20 scope).".yellow());
    println!("  Input: {}", cli.input.display().to_string().white().bold());
    println!("  Detected {} migration patterns\n", detected.len().to_string().cyan().bold());

    // Group by unique pattern_id for a cleaner summary.
    let mut seen_ids: Vec<&str> = Vec::new();
    let mut grouped: Vec<(&str, Vec<&patterns::DetectedPattern>)> = Vec::new();

    for p in detected {
        if let Some(pos) = seen_ids.iter().position(|id| **id == p.pattern_id) {
            grouped[pos].1.push(p);
        } else {
            seen_ids.push(&p.pattern_id);
            grouped.push((&p.pattern_id, vec![p]));
        }
    }

    for (i, (pattern_id, instances)) in grouped.iter().enumerate() {
        let first = instances[0];
        println!("{}", "  ┌─────────────────────────────────────────────────".bright_black());
        println!("  {} {}. {} ({}x detected)", "│".bright_black(), (i + 1).to_string().bold(), pattern_id.bold().yellow(), instances.len().to_string().cyan());
        println!("{}", "  ├─────────────────────────────────────────────────".bright_black());
        println!("  {} {}", "│".bright_black(), first.description.bright_white());
        println!("  {} ", "│".bright_black());
        println!("  {} {}", "│".bright_black(), "Stylus equivalent:".bold().green());
        for line in first.stylus_equivalent.lines() {
            println!("  {}   {}", "│".bright_black(), line.green());
        }

        if cli.verbose {
            println!("  {} ", "│".bright_black());
            println!("  {} {}", "│".bright_black(), "Found at:".italic().bright_black());
            for inst in instances {
                if let Some(ln) = inst.line_number {
                    println!("  {}   {} {}: {}", "│".bright_black(), "line".bright_black(), ln.to_string().cyan(), inst.matched_text.white());
                }
            }
        }

        println!("{}", "  └─────────────────────────────────────────────────".bright_black());
        println!();
    }

    println!("  Summary: {} unique pattern type(s), {} total instances", grouped.len().to_string().bold().white(), detected.len().to_string().bold().cyan());
    println!("  Use {} for machine-readable output.", "--format json".cyan());
    println!("  Use {} to see matched source lines.", "--verbose".cyan());
}
