//! `bear invite` command - create a tunnel invitation

use anyhow::Result;
use clap::Args;
use colored::*;
use std::time::Duration;

use crate::api;
use crate::config;
use crate::tunnel;

#[derive(Args, Debug)]
pub struct InviteArgs {
    /// Local port to expose
    #[clap(short, long, env = "BEAR_LOCAL_PORT", default_value_t = 3389)]
    pub port: u16,

    /// Gateway URL
    #[clap(short, long, env = "BEAR_GATEWAY")]
    pub gateway: Option<String>,

    /// Protocol to use
    #[clap(short, long, env = "BEAR_PROTOCOL", default_value = "rdp")]
    pub protocol: String,

    /// Name/label for this tunnel
    #[clap(short, long, env = "BEAR_NAME")]
    pub name: Option<String>,

    /// Security PIN code
    #[clap(short, long, env = "BEAR_PIN")]
    pub pin: Option<String>,

    /// Permission mode (approval_required, open_access, locked_down)
    #[clap(short, long, env = "BEAR_PERMISSION", default_value = "approval_required")]
    pub permission: String,

    /// Max number of guests
    #[clap(long, env = "BEAR_MAX_GUESTS", default_value_t = 5)]
    pub max_guests: u32,
}

pub async fn run(args: InviteArgs) -> Result<()> {
    let cfg = config::load()?;
    let gateway = args.gateway.unwrap_or(cfg.gateway);

    let name = args.name.unwrap_or_else(|| {
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "Bear PC".to_string())
    });

    print_banner_header();

    println!("{}", "📡 Sending request to Bear Gateway...".cyan());
    println!();

    let response = api::create_invite(
        &gateway,
        args.port,
        &args.protocol,
        &name,
        &args.permission,
        args.max_guests,
    )
    .await?;

    print_invite_result(&response, &gateway);
    print_usage_instructions(&response);
    print_tunnel_info(args.port, &response);

    // Start the reverse tunnel in the background
    let remote_host = response.public_address.split(':').next().unwrap_or(&gateway).to_string();
    let remote_port: u16 = response
        .public_address
        .split(':')
        .nth(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);

    if remote_port > 0 {
        println!();
        println!("{}", "⚡ Starting reverse TCP tunnel...".green().bold());
        println!(
            "{} {} <-> {}:{}",
            "🔄".yellow(),
            format!("127.0.0.1:{}", args.port).cyan(),
            remote_host.green(),
            remote_port.to_string().green()
        );
        println!("{}", "(Nhấn Ctrl+C để dừng phiên kết nối)".bright_black());
        println!();

        // Start tunnel with ctrl-c handling
        let tunnel_handle = tokio::spawn(async move {
            if let Err(e) = tunnel::run_reverse_tunnel(args.port, &remote_host, remote_port).await {
                eprintln!("{} {}", "Tunnel error:".red().bold(), e);
            }
        });

        // Wait for Ctrl+C
        tokio::signal::ctrl_c().await?;
        println!();
        println!("{}", "🛑 Stopping tunnel...".yellow());
        tunnel_handle.abort();
    }

    Ok(())
}

fn print_banner_header() {
    println!();
    println!(
        "{}",
        "================================================================".bright_cyan()
    );
    println!(
        "{}",
        "🐻 BEAR - Remote Desktop Tunneling CLI".bright_cyan().bold()
    );
    println!(
        "{}",
        "================================================================".bright_cyan()
    );
    println!();
}

fn print_invite_result(response: &api::InviteResponse, gateway: &str) {
    println!(
        "{}",
        "✅ BEAR REVERSE TUNNEL ALLOCATED SUCCESSFULLY!"
            .green()
            .bold()
    );
    println!();
    println!(
        "🔗 {} : {}",
        "Web Remote Desktop Link".cyan(),
        response.web_link.yellow().underline()
    );
    println!(
        "🖥️  {} : {}",
        "Native RDP / Host Address".cyan(),
        response.public_address.green().bold()
    );
    println!(
        "🔑 {} : {}",
        "Security PIN Code".cyan(),
        response.pin.yellow().bold()
    );
    println!(
        "🛡️  {} : {}",
        "Permission Mode".cyan(),
        response.permission_mode.white()
    );
    println!(
        "👥 {} : {}",
        "Max Guests".cyan(),
        response.max_guests.to_string().white()
    );
    println!(
        "🌐 {} : {}",
        "Gateway".cyan(),
        gateway.white()
    );
}

fn print_usage_instructions(response: &api::InviteResponse) {
    println!();
    println!(
        "{}",
        "----------------------------------------------------------------".bright_black()
    );
    println!(
        "📱 {}: Nhập \"{}\" vào ô PC Name",
        "Windows App (iOS/Android)".bright_blue(),
        response.public_address
    );
    println!(
        "💻 {}: Chạy \"mstsc.exe /v:{}\"",
        "Windows PC (mstsc.exe)".bright_blue(),
        response.public_address
    );
    println!(
        "🌐 {}: Gửi link trên cho khách điều khiển trực tiếp",
        "Trình duyệt web".bright_blue()
    );
    println!(
        "{}",
        "----------------------------------------------------------------".bright_black()
    );
}

fn print_tunnel_info(local_port: u16, response: &api::InviteResponse) {
    let remote = &response.public_address;
    println!(
        "{} {} {} {}",
        "⚡".yellow(),
        "Đang chuyển tiếp dữ liệu TCP:".green(),
        format!("127.0.0.1:{}", local_port).cyan(),
        format!("<-> {}", remote).green()
    );
}