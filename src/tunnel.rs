//! TCP tunneling core - handles port forwarding
use anyhow::Result;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::{info, warn};

/// Forward data between two TCP streams bidirectionally
pub async fn proxy_stream(local: TcpStream, remote: TcpStream) -> Result<()> {
    let (mut local_read, mut local_write) = local.into_split();
    let (mut remote_read, mut remote_write) = remote.into_split();
    
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

/// Connect to a remote TCP server with timeout
pub async fn connect_remote(host: &str, port: u16) -> Result<TcpStream> {
    let addr = format!("{}:{}", host, port);
    let stream = tokio::time::timeout(
        Duration::from_secs(5),
        TcpStream::connect(&addr),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Connection timeout to {}", addr))??;
    
    info!("Connected to remote: {}", addr);
    Ok(stream)
}

/// Run a simple client tunnel that forwards local port to remote server
pub async fn run_simple_tunnel(
    local_port: u16,
    remote_host: &str,
    remote_port: u16,
) -> Result<()> {
    info!("Starting tunnel: local 127.0.0.1:{} <-> remote {}:{}", local_port, remote_host, remote_port);
    
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", local_port)).await?;
    info!("Listening on 127.0.0.1:{}", local_port);
    
    loop {
        let (local_stream, _addr) = listener.accept().await?;
        let remote_host = remote_host.to_string();
        
        tokio::spawn(async move {
            match connect_remote(&remote_host, remote_port).await {
                Ok(remote_stream) => {
                    if let Err(e) = proxy_stream(local_stream, remote_stream).await {
                        warn!("Tunnel error: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Failed to connect to remote: {}", e);
                }
            }
        });
    }
}

/// Reverse tunnel: forward from remote to local
pub async fn run_reverse_tunnel(
    local_port: u16,
    remote_host: &str,
    remote_port: u16,
) -> Result<()> {
    use tokio::time::sleep;
    
    info!("Starting reverse tunnel: local 127.0.0.1:{} <-> remote {}:{}", local_port, remote_host, remote_port);
    
    loop {
        match connect_remote(remote_host, remote_port).await {
            Ok(remote_stream) => {
                info!("Connected to remote, awaiting traffic...");
                
                // For each remote connection, forward to local
                let (mut remote_read, mut remote_write) = remote_stream.into_split();
                let local_addr = format!("127.0.0.1:{}", local_port);
                
                let local_stream = match tokio::time::timeout(
                    Duration::from_secs(5),
                    TcpStream::connect(&local_addr),
                ).await {
                    Ok(Ok(s)) => s,
                    _ => {
                        warn!("Failed to connect to local {}", local_addr);
                        continue;
                    }
                };
                
                let (mut local_read, mut local_write) = local_stream.into_split();
                
                let up = async {
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
                
                let down = async {
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
                
                tokio::select! {
                    _ = up => (),
                    _ = down => (),
                }
            }
            Err(e) => {
                warn!("Connection failed: {}. Retrying in 3s...", e);
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}