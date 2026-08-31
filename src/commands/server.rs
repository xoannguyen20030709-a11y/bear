//! `bear server` command - run Bear Gateway Node

use anyhow::Result;
use clap::Args;
use colored::*;
use fastrand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

const CONTROL_PORT: u16 = 7835;
const DEFAULT_MIN_PORT: u16 = 20000;
const DEFAULT_MAX_PORT: u16 = 60000;

#[derive(Args, Debug)]
pub struct ServerArgs {
    /// Port for control connections
    #[clap(short = 'p', long, env = "BEAR_PORT", default_value_t = CONTROL_PORT)]
    pub port: u16,

    /// Minimum port for tunnel allocation
    #[clap(long, env = "BEAR_MIN_PORT", default_value_t = DEFAULT_MIN_PORT)]
    pub min_port: u16,

    /// Maximum port for tunnel allocation
    #[clap(long, env = "BEAR_MAX_PORT", default_value_t = DEFAULT_MAX_PORT)]
    pub max_port: u16,

    /// Optional secret for authentication
    #[clap(short, long, env = "BEAR_SECRET", hide_env_values = true)]
    pub secret: Option<String>,

    /// IP address to bind to
    #[clap(long, env = "BEAR_BIND", default_value = "0.0.0.0")]
    pub bind: String,
}

pub async fn run(args: ServerArgs) -> Result<()> {
    println!();
    println!("{}", "🐻 BEAR Gateway Server".cyan().bold());
    println!("{}", "====================".cyan());
    println!();
    println!("🌐 Listening on: {}:{}", args.bind, args.port);
    println!("🔌 Tunnel ports: {} - {}", args.min_port, args.max_port);
    if let Some(ref _s) = args.secret {
        println!("🔐 Authentication: enabled (secret configured)");
    } else {
        println!("🔓 Authentication: disabled");
    }
    println!();

    let listener = TcpListener::bind((args.bind.as_str(), args.port)).await?;
    info!("Bear server listening on {}:{}", args.bind, args.port);

    let state = Arc::new(RwLock::new(ServerState::new(args.min_port, args.max_port)));
    let state_clone = Arc::clone(&state);

    println!("{}", "✅ Server is running. Press Ctrl+C to stop.".green());
    println!();

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("New connection from {}", addr);
                let state = Arc::clone(&state_clone);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, state).await {
                        error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                warn!("Failed to accept connection: {}", e);
            }
        }
    }
}

struct ServerState {
    min_port: u16,
    max_port: u16,
    tunnels: HashMap<u16, Tunnel>,
    connections: HashMap<Uuid, TcpStream>,
}

struct Tunnel {
    local_port: u16,
    listener: TcpListener,
}

impl ServerState {
    fn new(min_port: u16, max_port: u16) -> Self {
        Self {
            min_port,
            max_port,
            tunnels: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    fn allocate_port(&self) -> Option<u16> {
        let mut rng = Rng::new();
        for _ in 0..100 {
            let port = rng.u16(self.min_port..=self.max_port);
            if !self.tunnels.contains_key(&port) {
                return Some(port);
            }
        }
        None
    }
}

async fn handle_connection(stream: TcpStream, state: Arc<RwLock<ServerState>>) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, BufStream};
    use std::io::Write;

    let mut stream = BufStream::new(stream);
    let mut line = String::new();

    // Read command
    if stream.read_line(&mut line).await? == 0 {
        return Ok(());
    }

    let line = line.trim();
    info!("Received command: {}", line);

    // Parse command (simple text protocol)
    // HELLO <port>\n - request tunnel
    // ACCEPT <uuid>\n - accept connection
    // BYE\n - close

    if line.starts_with("HELLO ") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let requested_port: u16 = parts[1].parse().unwrap_or(0);
            let mut state = state.write().await;

            let port = if requested_port > 0 {
                if state.tunnels.contains_key(&requested_port) {
                    None
                } else {
                    Some(requested_port)
                }
            } else {
                state.allocate_port()
            };

            if let Some(port) = port {
                let bind_addr = "0.0.0.0";
                let listener = TcpListener::bind((bind_addr, port)).await?;
                info!("Allocated tunnel port: {}", port);

                let response = format!("OK {}\n", port);
                stream.write_all(response.as_bytes()).await?;
                stream.flush().await?;

                state.tunnels.insert(port, Tunnel {
                    local_port: 0, // Would need to parse from client
                    listener,
                });
            } else {
                let response = "ERROR no ports available\n";
                stream.write_all(response.as_bytes()).await?;
                stream.flush().await?;
            }
        }
    } else if line.starts_with("ACCEPT ") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let id = parts[1];
            let mut state = state.write().await;
            if let Some(mut conn) = state.connections.remove(&Uuid::parse_str(id).unwrap_or_default()) {
                let response = "OK\n";
                stream.write_all(response.as_bytes()).await?;
                stream.flush().await?;
                info!("Accepted connection: {}", id);
            } else {
                let response = "ERROR connection not found\n";
                stream.write_all(response.as_bytes()).await?;
                stream.flush().await?;
            }
        }
    } else if line == "STATUS" {
        let state = state.read().await;
        let response = format!("TUNNELS {} CONNECTIONS {}\n", 
            state.tunnels.len(), 
            state.connections.len()
        );
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
    } else if line == "BYE" {
        let response = "OK\n";
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
    }

    Ok(())
}