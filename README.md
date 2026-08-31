# tunnel-rs

`tunnel-rs` is a modern, simple TCP tunnel in Rust that exposes local ports to a remote server, bypassing standard NAT connection firewalls.

Similar to [localtunnel](https://github.com/localtunnel) and [ngrok](https://ngrok.io/), except `tunnel-rs` is intended to be a highly efficient, unopinionated tool for forwarding TCP traffic that is simple to install and easy to self-host, with no frills attached.

## Installation

`cargo install` is the easiest way to build `tunnel-rs` from source:

```shell
cargo install --path .
```

## Usage

### Local Forwarding

On your local machine, run:

```shell
tunnel local 8000 --to example.com
```

This exposes your local port `8000` to the public internet at `example.com:<PORT>`, where the port number is assigned randomly.

### Self-Hosting

Run the server:

```shell
tunnel server
```

By default it listens on `0.0.0.0:7836`. Point the client here using `--to`.

### Authentication

Protect a self-hosted server with a secret:

```shell
# on the server
tunnel server --secret my_secret_string

# on the client
tunnel local <LOCAL_PORT> --to <TO> --secret my_secret_string
```

## Protocol

There is an implicit *control port* at `7836`, used for creating new connections on demand. At initialization, the client sends a "Hello" message to the server on the TCP control port, asking to proxy a selected remote port. The server responds with an acknowledgement and begins listening for external TCP connections.

Whenever the server obtains a connection on the remote port, it generates a secure [UUID](https://en.wikipedia.org/wiki/Universally_unique_identifier) and sends it back to the client. The client opens a separate TCP stream and sends an "Accept" message. The server then proxies the two connections together.

Incoming connections are discarded if the client does not accept them within 10 seconds.

## License

MIT