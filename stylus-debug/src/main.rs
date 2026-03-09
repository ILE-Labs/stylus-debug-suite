use clap::{Parser, Subcommand};
use anyhow::Result;

mod migrate;
mod adapter;
mod demo;

#[derive(Parser)]
#[command(name = "stylus-debug", version, about = "Stylus Debug Suite CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
        Commands::Migrate(args) => migrate::run(args).await?,
        Commands::Adapter(args) => adapter::run(args).await?,
        Commands::Demo(args) => demo::run(args).await?,
    }

    Ok(())
}
