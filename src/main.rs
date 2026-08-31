use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod api;
mod commands;
mod config;
mod tunnel;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create an invitation for remote access
    Invite(commands::invite::InviteArgs),
    /// Connect to a remote server as a client
    Client(commands::client::ClientArgs),
    /// Run the Bear Gateway Node server
    Server(commands::server::ServerArgs),
    /// Check tunnel status and active connections
    Status(commands::status::StatusArgs),
    /// Test network connectivity and latency
    Test(commands::test::TestArgs),
    /// Manage configuration
    Config(commands::config::ConfigArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Invite(args) => commands::invite::run(args).await,
        Commands::Client(args) => commands::client::run(args).await,
        Commands::Server(args) => commands::server::run(args).await,
        Commands::Status(args) => commands::status::run(args).await,
        Commands::Test(args) => commands::test::run(args).await,
        Commands::Config(args) => commands::config::run(args).await,
    }
}