# Bear - Remote Desktop Tunneling CLI

Bear is a modern CLI tool for creating secure reverse tunnels to the Bear Web Gateway, enabling remote access to local services (RDP, VNC, SSH, web apps) through firewalls and NATs.

## 🚀 Features

- **Instant Remote Access**: Create secure tunnels with a single command
- **Multi-Protocol Support**: RDP, VNC, SSH, and generic TCP services
- **Web-Based Control**: Access via browser at https://bear-way.ai.studio
- **Secure Authentication**: HMAC-based challenge-response protocol
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Zero Configuration**: Automatic setup with sensible defaults
- **Professional Output**: Colorful terminal interface with clear instructions

## 📦 Installation

### From Source (requires Rust)

```bash
git clone https://github.com/example/bear
cd bear
cargo install --path .
```

### Pre-built Binaries

Download from [Releases](https://github.com/example/bear/releases) page.

### Homebrew (macOS/Linux)

```bash
brew tap bear-io/bear
brew install bear
```

## 🔧 Usage

### 1. Create an Invitation (Main Command)

```bash
bear invite [--port 3389] [--gateway bear-way.ai.studio] [--protocol rdp] [--name "My PC"] [--pin 1234] [--permission approval_required]
```

Example:
```bash
bear invite --port 3389 --name "Office PC"
```

This will output something like:
```
===============================================================
🐻 BEAR - Remote Desktop Tunneling CLI
===============================================================

📡 Sending request to Bear Gateway...

✅ BEAR REVERSE TUNNEL ALLOCATED SUCCESSFULLY!

🔗 Web Remote Desktop Link : https://bear-way.ai.studio?join=bear-rdp-xxxx&pin=1234
🖥️  Native RDP / Host Address: bear-way.ai.studio:48922
🔑 Security PIN Code        : 1234
🛡️  Permission Mode          : approval_required
👥 Max Guests               : 5
🌐 Gateway                  : bear-way.ai.studio

----------------------------------------------------------------
📱 Windows App (iOS/Android): Nhập "bear-way.ai.studio:48922" vào ô PC Name
💻 Windows PC (mstsc.exe)   : Chạy "mstsc.exe /v:bear-way.ai.studio:48922"
🌐 Trình duyệt web          : Gửi link trên cho khách điều khiển trực tiếp
----------------------------------------------------------------

⚡ Đang chuyển tiếp dữ liệu TCP: 127.0.0.1:3389 <-> bear-way.ai.studio:48922
(Nhấn Ctrl+C để dừng phiên kết nối)
```

### 2. Manual Client Connection

```bash
bear client --local-port 3389 --to bear-way.ai.studio:7836 --secret my_secret
```

### 3. Run Your Own Gateway Node

```bash
bear server --port 7835 --min-port 20000 --max-port 60000 --secret my_secret
```

### 4. Check Tunnel Status

```bash
bear status [--gateway bear-way.ai.studio]
```

### 5. Network Diagnostics

```bash
bear test [--gateway bear-way.ai.studio] [--port 3389]
```

### 6. Manage Configuration

```bash
# Set default gateway
bear config set --gateway bear-way.ai.studio

# View current config
bear config get
```

## 🛠️ Configuration

Bear automatically saves your preferences to `~/.bear/config.toml`:

```toml
gateway = "bear-way.ai.studio"
```

Environment variables can override config:
- `BEAR_GATEWAY` - Default gateway URL
- `BEAR_LOCAL_PORT` - Default local port to expose
- `BEAR_PROTOCOL` - Default protocol (rdp, vnc, web, ssh)
- `BEAR_NAME` - Default tunnel name
- `BEAR_PERMISSION` - Default permission mode
- `BEAR_SECRET` - Authentication secret
- `BEAR_MIN_PORT` / `BEAR_MAX_PORT` - Gateway port range
- `BEAR_PORT` - Gateway control port

## 🔐 Security

- All tunnel allocations require authentication with the Bear Gateway
- Optional HMAC-based secrets for additional client-server authentication
- Traffic is NOT encrypted by default - use TLS/RDP encryption for sensitive data
- Connections automatically expire after inactivity
- PIN codes provide additional layer of access control

## 🐻 How It Works

1. **Invitation**: `bear invite` sends a request to `https://bear-way.ai.studio/api/cli/invite`
2. **Allocation**: Gateway allocates a public port and returns connection details
3. **Tunneling**: Bear establishes a reverse TCP tunnel from your machine to the gateway
4. **Connection**: Clients connect to the public address, which forwards to your local service
5. **Management**: Use `bear status` to monitor active tunnels and traffic

## 📋 Protocol Details

Bear uses a simple text-based protocol over TCP for gateway communication:
- `HELLO <port>` - Request a tunnel (port 0 for auto-assign)
- `ACCEPT <uuid>` - Accept an incoming connection
- `BYE` - Close connection
- `STATUS` - Get tunnel statistics

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by [bore](https://github.com/ekzhang/bore)
- Built with [Tokio](https://tokio.rs/) for async I/O
- Uses [Clap](https://crates.io/crates/clap) for CLI argument parsing
- Powered by [Reqwest](https://crates.io/crates/reqwest) for HTTP requests