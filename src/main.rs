mod commands;
mod config;
mod models;
mod utils;

#[cfg(test)]
mod tests;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = commands::Cli::parse();
    commands::handlers::handle_command(cli.command).await
}
