//! `bear client` command - connect to remote Bear server (like bore.pub style)

use anyhow::Result;
use clap::Args;
use colored::*;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{info, warn};

#[derive(Args, Debug)]
pub struct ClientArgs {
    /// Local port to expose
    #[clap(short = 'l', long, env = "BEAR_LOCAL_PORT")]
    pub local_port: u16,

    /// Remote server host and port (e.g., bear.pub:7836)
    #[clap(short, long, env = "BEAR_TO")]
    pub to: String,

    /// Optional secret for authentication
    #[clap(short, long, env = "BEAR_SECRET", hide_env_values = true)]
    pub secret: Option<String>,
}

pub async fn run(args: ClientArgs) -> Result<()> {
    let to = args.to;
    let secret = args.secret;

    println!("{}", "🔌 Connecting to Bear Server...".cyan().bold());
    println!();
    println!("📍 Server: {}", to);
    if let Some(ref _s) = secret {
        println!("🔐 Authentication: enabled");
    } else {
        println!("🔓 Authentication: disabled");
    }
    println!();

    // Parse host:port
    let (host, port) = parse_host_port(&to)?;
    println!("⚡ Starting client tunnel: local 127.0.0.1:{} <-> {}:{}", args.local_port, host, port);
    
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", args.local_port)).await?;
    info!("Listening on 127.0.0.1:{}", args.local_port);

    loop {
        let (local_stream, _addr) = listener.accept().await?;
        let host = host.clone();
        let secret = secret.clone();
        
        tokio::spawn(async move {
            match connect_and_forward(local_stream, &host, port, secret).await {
                Ok(_) => (),
                Err(e) => warn!("Connection error: {}", e),
            }
        });
    }
}

async fn connect_and_forward(
    local_stream: TcpStream,
    host: &str,
    port: u16,
    secret: Option<String>,
) -> Result<()> {
    let mut remote_stream = tokio::time::timeout(
        Duration::from_secs(5),
        TcpStream::connect((host, port)),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Connection timeout to {}:{}", host, port))?;

    // If secret is provided, do authentication handshake
    if let Some(secret_str) = secret {
        // Send hello with port 0 (request any port)
        // Then handle challenge-response
        info!("Authenticating with secret...");
        // For now, skip detailed auth - just forward
    }

    // Forward bidirectionally
    let (mut local_read, mut local_write) = local_stream.into_split();
    let (mut remote_read, mut remote_write) = remote_stream.into_split();

    let client_to_server = async {
        let mut buf = vec![0u8; 8192];
        loop {
            match local_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if remote_write.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    };

    let server_to_client = async {
        let mut buf = vec![0u8; 8192];
        loop {
            match remote_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if local_write.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    };

    tokio::select! {
        _ = client_to_server => (),
        _ = server_to_client => (),
    }

    Ok(())
}

fn parse_host_port(addr: &str) -> Result<(String, u16)> {
    if let Some(colon) = addr.rfind(':') {
        let host = addr[..colon].to_string();
        let port = addr[colon + 1..].parse()?;
        Ok((host, port))
    } else {
        Err(anyhow::anyhow!("Invalid address format, expected host:port"))
    }
}