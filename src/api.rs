//! Bear Web Gateway API client.
//! Handles communication with https://bear-way.ai.studio

use serde::{Deserialize, Serialize};
use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

/// Request payload for creating an invite/tunnel
#[derive(Debug, Serialize)]
pub struct InviteRequest {
    /// Local port to forward
    pub local_port: u16,
    /// Protocol type (rdp, vnc, web, ssh)
    pub protocol: String,
    /// Name/label for this tunnel
    pub name: String,
    /// Permission mode
    pub permission_mode: String,
    /// Maximum number of guests
    pub max_guests: u32,
}

/// Response from the API when creating an invite
#[derive(Debug, Deserialize)]
pub struct InviteResponse {
    /// Status message from server
    pub status: String,
    /// Public address for the tunnel (host:port)
    pub public_address: String,
    /// Web link with join token
    pub web_link: String,
    /// Security PIN code
    pub pin: String,
    /// Permission mode confirmation
    pub permission_mode: String,
    /// Maximum guests allowed
    pub max_guests: u32,
}

/// Health check response from gateway
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub online_tunnels: u32,
    pub traffic_up: u64,
    pub traffic_down: u64,
}

/// Check gateway health
pub async fn check_health(gateway: &str) -> Result<HealthResponse> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let url = format!("https://{}/api/health", gateway);
    let resp = client.get(&url).send().await?;
    let body: HealthResponse = resp.json().await?;
    Ok(body)
}

/// Send invite request to create a tunnel
pub async fn create_invite(
    gateway: &str,
    local_port: u16,
    protocol: &str,
    name: &str,
    permission_mode: &str,
    max_guests: u32,
) -> Result<InviteResponse> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let url = format!("https://{}/api/cli/invite", gateway);
    
    let req = InviteRequest {
        local_port,
        protocol: protocol.to_string(),
        name: name.to_string(),
        permission_mode: permission_mode.to_string(),
        max_guests,
    };
    
    let resp = client.post(&url).json(&req).send().await?;
    let body: InviteResponse = resp.json().await?;
    Ok(body)
}

/// List active tunnels
pub async fn list_tunnels(gateway: &str) -> Result<Vec<TunnelInfo>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let url = format!("https://{}/api/tunnels", gateway);
    let resp = client.get(&url).send().await?;
    let tunnels: Vec<TunnelInfo> = resp.json().await?;
    Ok(tunnels)
}

/// Tunnel info from status API
#[derive(Debug, Deserialize, Clone)]
pub struct TunnelInfo {
    pub id: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: String,
    pub status: String,
    pub connected_at: String,
    pub traffic_up: u64,
    pub traffic_down: u64,
}