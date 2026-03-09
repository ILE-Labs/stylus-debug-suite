use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use colored::*;

pub mod patterns;

#[derive(Parser)]
pub struct MigrateArgs {
    /// Path to the Solidity source file to analyze.
    #[arg(default_value = "examples/demo-contracts/Demo.sol")]
    pub input: PathBuf,

    /// Output format: text or json.
    #[arg(long, default_value = "text")]
    pub format: String,

    /// Show verbose output including matched source lines.
    #[arg(long)]
    pub verbose: bool,
}

pub async fn run(args: MigrateArgs) -> Result<()> {
    let detected = patterns::detect_patterns(&args.input);

    match args.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&detected)?;
            println!("{json}");
        }
        _ => print_text_report(&args, &detected),
    }

    Ok(())
}

fn print_text_report(args: &MigrateArgs, detected: &[patterns::DetectedPattern]) {
    println!("\n  {}", "Stylus Migration Assistant".bold().cyan());
    println!("  Analyzing: {}", args.input.display().to_string().white().bold());
    println!("  Detected {} patterns\n", detected.len().to_string().cyan().bold());

    for (i, p) in detected.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, p.pattern_id.bold().yellow(), p.solidity_construct.bright_white());
        println!("     {}", p.description.bright_black());
        println!("     {} {}", "Stylus:".bold().green(), p.stylus_equivalent.green());
        if args.verbose && p.line_number.is_some() {
            println!("     {} line {}", "Location:".italic().bright_black(), p.line_number.unwrap());
        }
        println!();
    }
}
