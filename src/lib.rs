//! A lightweight TCP tunnel written in Rust.
//!
//! `tunnel` is a simple CLI tool for forwarding a local TCP port to a remote
//! server, allowing you to expose services running behind a NAT firewall.
//! It is designed to be minimal, efficient, and easy to self-host.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod auth;
pub mod client;
pub mod server;
pub mod shared;
