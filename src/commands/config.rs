//! `bear config` command - manage default configuration

use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::config;

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    action: ConfigAction,
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Set default gateway
    Set {
        /// Gateway URL
        #[clap(short, long, env = "BEAR_GATEWAY")]
        gateway: String,
    },
    /// Show current configuration
    Get,
}

pub async fn run(args: ConfigArgs) -> Result<()> {
    match args.action {
        ConfigAction::Set { gateway } => {
            println!("{}", "⚙️  Cập nhật cấu hình Bear...".cyan().bold());
            println!();

            let mut cfg = config::load()?;
            cfg.gateway = gateway.clone();

            match config::save(&cfg) {
                Ok(_) => {
                    println!("  ✅ Gateway đã được đặt thành: {}", gateway.green().bold());
                    println!("  📁 File cấu hình đã được lưu");
                }
                Err(e) => {
                    println!("  ❌ Không thể lưu cấu hình: {}", e.to_string().red());
                }
            }
        }
        ConfigAction::Get => {
            println!("{}", "⚙️  Cấu hình Bear hiện tại".cyan().bold());
            println!("{}", "==========================".cyan());
            println!();

            match config::load() {
                Ok(cfg) => {
                    println!("  🌐 Gateway: {}", cfg.gateway.cyan().bold());
                    println!("  📁 File cấu hình: ~/.bear/config.toml");
                }
                Err(e) => {
                    println!("  ❌ Không thể tải cấu hình: {}", e.to_string().red());
                }
            }
        }
    }

    Ok(())
}
