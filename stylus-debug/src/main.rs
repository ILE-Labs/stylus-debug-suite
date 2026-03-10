use clap::{Parser, Subcommand, CommandFactory};
use anyhow::Result;

mod migrate;
mod adapter;
mod demo;

#[derive(Parser)]
#[command(name = "stylus-debug", version, about = "Stylus Debug Suite CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze Solidity contracts for Stylus migration.
    Migrate(migrate::MigrateArgs),
    /// Start the DAP-compatible debug adapter.
    Adapter(adapter::AdapterArgs),
    /// Run the Stylus VM demo.
    Demo(demo::DemoArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Migrate(args)) => migrate::run(args).await?,
        Some(Commands::Adapter(args)) => adapter::run(args).await?,
        Some(Commands::Demo(args)) => demo::run(args).await?,
        None => {
            Cli::command().print_help()?;
            println!();
        }
    }

    Ok(())
}
