//! `bear status` command - check tunnel status

use anyhow::Result;
use clap::Args;
use colored::*;

use crate::api;
use crate::config;

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Gateway URL
    #[clap(short, long, env = "BEAR_GATEWAY")]
    pub gateway: Option<String>,
}

pub async fn run(args: StatusArgs) -> Result<()> {
    let cfg = config::load()?;
    let gateway = args.gateway.unwrap_or(cfg.gateway);

    println!("{}", "🔍 Checking Bear Gateway status...".cyan().bold());
    println!();

    // Check gateway health
    match api::check_health(&gateway).await {
        Ok(health) => {
            println!("{}", "📊 Gateway Status".cyan().bold());
            println!("{}", "==================".cyan());
            println!("🌐 Gateway: {}", gateway);
            println!("📈 Status: {}", health.status.green());
            println!("🔄 Online Tunnels: {}", health.online_tunnels.to_string().yellow());
            println!("📤 Traffic Up: {} bytes", health.traffic_up.to_string().white());
            println!("📥 Traffic Down: {} bytes", health.traffic_down.to_string().white());
            println!();
        }
        Err(e) => {
            println!("{} {}", "❌ Failed to check gateway health:".red().bold(), e);
            println!();
        }
    }

    // List active tunnels
    println!("{}", "🔗 Active Tunnels".cyan().bold());
    println!("{}", "==================".cyan());

    match api::list_tunnels(&gateway).await {
        Ok(tunnels) => {
            if tunnels.is_empty() {
                println!("{}", "  (no active tunnels)".bright_black());
            } else {
                for tunnel in tunnels {
                    print_tunnel(&tunnel);
                }
            }
        }
        Err(e) => {
            println!("{} {}", "❌ Failed to list tunnels:".red().bold(), e);
        }
    }

    println!();
    Ok(())
}

fn print_tunnel(tunnel: &api::TunnelInfo) {
    let status_color = if tunnel.status == "online" { "green" } else { "red" };
    let status = match status_color {
        "green" => tunnel.status.green(),
        _ => tunnel.status.red(),
    };

    println!();
    println!("  🆔 ID: {}", tunnel.id.white());
    println!("  📍 Local Port: {}", tunnel.local_port.to_string().yellow());
    println!("  🌐 Remote Port: {}", tunnel.remote_port.to_string().yellow());
    println!("  📡 Protocol: {}", tunnel.protocol.cyan());
    println!("  📊 Status: {}", status);
    println!("  🕐 Connected: {}", tunnel.connected_at.white());
    println!("  📤 Traffic Up: {} bytes", tunnel.traffic_up.to_string().white());
    println!("  📥 Traffic Down: {} bytes", tunnel.traffic_down.to_string().white());
}