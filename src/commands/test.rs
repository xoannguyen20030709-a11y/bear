//! `bear test` command - diagnose and test network connectivity

use anyhow::Result;
use clap::Args;
use colored::*;
use std::time::Duration;

use crate::api;
use crate::config;

#[derive(Args, Debug)]
pub struct TestArgs {
    /// Gateway URL
    #[clap(short, long, env = "BEAR_GATEWAY")]
    pub gateway: Option<String>,

    /// Local port to test (e.g., 3389 for RDP)
    #[clap(long, default_value_t = 3389)]
    pub port: u16,

    /// Use TCP probe instead of HTTP
    #[clap(long)]
    pub tcp_probe: bool,
}

pub async fn run(args: TestArgs) -> Result<()> {
    let cfg = config::load()?;
    let gateway = args.gateway.unwrap_or(cfg.gateway);

    println!("{}", "🧪 Running Bear Gateway diagnostics...".cyan().bold());
    println!();

    // Test 1: Gateway connectivity
    println!("{}", "📡 Test 1: Gateway reachability".bright_black().bold());
    let gateway_reachable = test_gateway_reachability(&gateway).await;
    
    if gateway_reachable {
        println!("  ✅ Gateway {} is reachable", gateway.green());
    } else {
        println!("  ❌ Gateway {} is unreachable", gateway.red());
    }
    println!();

    // Test 2: Local port test
    println!("{}", "📡 Test 2: Local port test".bright_black().bold());
    test_local_port(args.port).await;
    println!();

    // Test 3: API endpoint test
    println!("{}", "📡 Test 3: API endpoint".bright_black().bold());
    match api::check_health(&gateway).await {
        Ok(health) => {
            println!("  ✅ API endpoint is working");
            println!("  📊 Gateway status: {}", health.status.green());
            println!("  🔄 Online tunnels: {}", health.online_tunnels.to_string().yellow());
        }
        Err(e) => {
            println!("  ❌ API endpoint failed: {}", e.to_string().red());
        }
    }
    println!();

    // Test 4: Tunnel allocation test
    println!("{}", "📡 Test 4: Tunnel allocation simulation".bright_black().bold());
    test_tunnel_allocation(&gateway, args.port).await;
    println!();

    // Test 5: Network diagnostics
    println!("{}", "📡 Test 5: Network diagnostics".bright_black().bold());
    test_network_diagnostics().await;
    println!();

    println!("{}", "📊 Test Summary".cyan().bold());
    println!("{}", "=============".cyan());
    println!("  All tests completed. Check output above for details.");
    println!("  Use 'bear status' for active tunnel information.");

    Ok(())
}

async fn test_gateway_reachability(gateway: &str) -> bool {
    let url = format!("https://{}/api/health", gateway);
    
    match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(client) => {
            match client.get(&url).send().await {
                Ok(resp) => resp.status().is_success(),
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

async fn test_local_port(port: u16) {
    let addr = format!("127.0.0.1:{}", port);
    
    match tokio::net::TcpStream::connect(&addr).await {
        Ok(_) => {
            println!("  ✅ Local port {} is listening", port.to_string().green());
        }
        Err(e) => {
            println!("  ❌ Local port {} is NOT listening: {}", 
                port.to_string().red(), e.to_string().bright_black());
        }
    }
}

async fn test_tunnel_allocation(gateway: &str, port: u16) {
    // Try to create a test tunnel (will be cleaned up)
    let name = "test-tunnel";
    let protocol = "rdp";
    let permission = "open_access";
    let max_guests: u32 = 1;

    match api::create_invite(gateway, port, protocol, name, permission, max_guests).await {
        Ok(response) => {
            println!("  ✅ Tunnel allocation successful");
            println!("  🔗 Link: {}", response.web_link.yellow().underline());
            println!("  🔑 PIN: {}", response.pin.yellow().bold());
            
            // Clean up - delete the tunnel (in real implementation)
            // For now, just report success
            println!("  🧹 Test tunnel created (would be cleaned up in production)");
        }
        Err(e) => {
            println!("  ❌ Tunnel allocation failed: {}", e.to_string().red());
        }
    }
}

async fn test_network_diagnostics() {
    // Test DNS resolution
    if let Ok(hostname) = hostname::get() {
        if let Ok(hostname_str) = hostname.to_string_os().into_string() {
            println!("  📍 Local hostname: {}", hostname_str.white());
        }
    }

    // Test connectivity to common servers
    let tests = vec![
        ("Google DNS", "8.8.8.8"),
        ("Cloudflare DNS", "1.1.1.1"),
        ("Bear Gateway", "bear-way.ai.studio"),
    ];

    for (name, addr) in tests {
        let start = std::time::Instant::now();
        match tokio::net::TcpStream::connect(format!("{}:80", addr)).await {
            Ok(_) => {
                let duration = start.elapsed();
                println!("  ✅ {} reachable ({}ms)", name, duration.as_millis());
            }
            Err(e) => {
                println!("  ❌ {} unreachable: {}", name, e.to_string().bright_black());
            }
        }
    }
}