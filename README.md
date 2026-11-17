# Post-Quantum VPN

A secure Virtual Private Network (VPN) implementation using **post-quantum cryptographic algorithms** designed to resist attacks from both classical and future quantum computers.

**By [MDQuantum](https://mdquantum.xyz)**

## Why Post-Quantum?

Current encryption methods rely on mathematical problems that quantum computers could solve efficiently using Shor's algorithm. This project uses **NIST-standardized post-quantum algorithms** to ensure your data remains secure even in a post-quantum world.

## How It Works

The VPN is built on **TLS 1.3** with native post-quantum key exchange via the `rustls` library and its `aws-lc-rs` cryptographic backend. This approach provides quantum resistance directly at the transport layer, without requiring a custom protocol.

### Cryptographic Approach

The hybrid key exchange **X25519MLKEM768** combines:

- **X25519** — A well-proven classical key exchange algorithm
- **ML-KEM-768** (CRYSTALS-Kyber) — NIST's post-quantum key encapsulation mechanism (FIPS 203)

This hybrid approach provides **defense in depth**: even if one algorithm is broken, the other still protects your connection.

> **Note**: The `rustls` default backend is `aws-lc-rs`, which natively supports X25519MLKEM768. No separate post-quantum crates are needed.

### Security Guarantees

- ✅ **Quantum resistance** — ML-KEM-768 protects against future quantum adversaries using Shor's algorithm
- ✅ **Classical security** — X25519 provides proven security against classical attackers
- ✅ **NIST compliant** — ML-KEM-768 aligns with FIPS 203 (NIST post-quantum standard)
- ✅ **Forward secrecy** — Each session uses unique keys that cannot be compromised retroactively

### Future Cryptographic Extensions

For additional crypto-agility, the project may integrate:

- **ML-DSA (CRYSTALS-Dilithium)** — Post-quantum digital signatures (FIPS 204)
- **SLH-DSA (SPHINCS+)** — Hash-based signatures (FIPS 205)

## Architecture

```
┌─────────────┐         TLS 1.3 + X25519MLKEM768          ┌─────────────┐
│  VPN Client  │ ◄────────────────────────────────────────► │  VPN Server  │
│              │                                            │              │
│  • Rust      │       Encrypted Tunnel (Post-Quantum)      │  • Rust      │
│  • SOCKS5    │                                            │  • Docker    │
│  • TUN iface │                                            │  • Proxmox   │
└─────────────┘                                            └─────────────┘
```

### Client Features

- **SOCKS5 Proxy** — Route specific application traffic through the VPN
- **TUN Interface** — Full IP-layer tunneling for complete network redirection
- **Custom Certificates** — Support for self-signed and private CA certificates
- **Cross-platform** — Built in Rust for Windows, macOS, and Linux support

### Server Features

- **Docker** container deployment
- **Proxmox VM** deployment
- **Scripted installation** for Debian and Ubuntu

## Project Structure

```
vpn/
├── vpn-client/        VPN client application (Rust)
│   ├── Cargo.toml     Dependencies and configuration
│   └── src/
│       └── main.rs    Client entry point
├── vpn-server/        VPN server application (Rust)
│   ├── Cargo.toml     Server dependencies
│   └── src/
│       └── main.rs    Server entry point
├── .gitignore         Git ignore rules
├── CONTEXT.md         Detailed technical documentation
└── README.md          This file
```

## Protocol Design

### Transport Layer: TLS 1.3 with X25519MLKEM768

The VPN tunnel is built on top of TLS 1.3, which natively provides post-quantum key exchange via the X25519MLKEM768 hybrid group. No custom handshake protocol is needed at the transport layer.

**Connection flow**:
1. **TCP Connection** — Client connects to server on configured port (default 443)
2. **TLS 1.3 Handshake** — Uses X25519MLKEM768 as the first key exchange group, providing hybrid classical + post-quantum security
3. **Certificate Verification** — Server certificate is verified against trusted roots (Mozilla roots or custom `--server-cert`)
4. **Encrypted Channel** — The TLS 1.3 record layer provides authenticated encryption (AES-256-GCM or ChaCha20-Poly1305)

### Application Layer: VPN Tunnel Protocol

Over the encrypted TLS channel, a custom application-layer protocol manages the VPN tunnel:

**Planned message format**:
```
[Message Type (1 byte)] [Sequence Number (4 bytes)] [Payload (variable)] [MAC (16 bytes)]
```

**Planned message types**:
- `0x01`: Handshake request (tunnel initialization)
- `0x02`: Handshake response (tunnel established)
- `0x03`: Data packet (encapsulated IP traffic)
- `0x04`: Keepalive (connection health check)
- `0x05`: Control message (configuration changes, disconnect)
- `0xFF`: Error

## Client Modules

| Module | Purpose | Status |
|--------|---------|--------|
| `server.rs` | TLS 1.3 connection to the VPN server with native X25519MLKEM768 key exchange | ❌ Not started |
| `socks5.rs` | SOCKS5 proxy server for application-level traffic forwarding through the VPN tunnel | ❌ Not started |
| `tun.rs` | TUN interface creation and management for full IP-layer tunneling | ❌ Not started |
| `types.rs` | Protocol message formats, packet headers, and serialization structures | ❌ Not started |
| `util.rs` | Utility functions for packet encoding/decoding and helpers | ❌ Not started |

> **Note**: A separate `crypto.rs` module is **not needed** because post-quantum key exchange (X25519MLKEM768) is handled natively by the TLS 1.3 layer via rustls + aws-lc-rs.

## Server Modules

| Module | Purpose | Status |
|--------|---------|--------|
| `handler.rs` | Connection handler for managing multiple concurrent client sessions | ❌ Not started |
| `auth.rs` | Authentication integration with LDAP/Active Directory and OpenID Connect | ❌ Not started |
| `routing.rs` | Traffic routing between VPN clients and university network resources | ❌ Not started |
| `config.rs` | Server configuration management | ❌ Not started |

## Dependencies

### Current Dependencies (`vpn-client/Cargo.toml`)

| Dependency | Version | Purpose |
|------------|---------|---------|
| `tokio` | 1 | Async runtime (net, io-util, macros, rt-multi-thread, time, sync) |
| `rustls` | 0.23 | TLS 1.3 with native X25519MLKEM768 (default aws-lc-rs backend) |
| `webpki-roots` | 1.0 | Mozilla root CA trust anchors |
| `clap` | 4 | CLI argument parsing with derive macros |
| `anyhow` | 1 | Error handling with error chains |
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | Logging with ENV filter support |
| `rcgen` | 0.14 | Self-signed certificate generation (dev-dependency for tests) |

### Future Dependencies (to be added)

| Dependency | Purpose |
|------------|---------|
| `tokio-rustls` | TLS 1.3 integration with Tokio async runtime |
| `fast-socks5` | SOCKS5 proxy server for application-level traffic |
| `tun` | TUN/TAP interface for full IP-layer tunneling |
| `postcard` + `bytes` | Binary serialization for application-layer protocol |

> **Important**: The `rustls` crate must use its **default backend** (`aws-lc-rs`). Do not enable the `ring` feature, as it does not support X25519MLKEM768.

## Building

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (edition 2024)
- Linux system with TUN device support (`/dev/net/tun`)

### Compile the Client

```bash
cd vpn-client
cargo build --release
```

### Run the Client

```bash
# Basic usage
./target/release/vpn-client --port 1080 --server-domain vpn.example.com

# With custom server port
./target/release/vpn-client --port 1080 --server-domain vpn.example.com --server-port 8443

# With custom server certificate
./target/release/vpn-client --port 1080 --server-domain vpn.example.com --server-cert server.pem
```

### Run Tests

```bash
cd vpn-client
cargo test
```

## Cryptographic Standards

This project follows the NIST post-quantum cryptography standards:

| Standard | Algorithm | Purpose |
|----------|-----------|---------|
| [FIPS 203](https://csrc.nist.gov/pubs/fips/203/final) | ML-KEM (CRYSTALS-Kyber) | Post-quantum key exchange |
| [FIPS 204](https://csrc.nist.gov/pubs/fips/204/final) | ML-DSA (CRYSTALS-Dilithium) | Post-quantum digital signatures |
| [FIPS 205](https://csrc.nist.org/pubs/fips/205/final) | SLH-DSA (SPHINCS+) | Hash-based signatures |

## Network Compatibility

- **IPv4/IPv6** dual-stack support
- **NAT traversal** — Works behind home routers
- **CGNAT support** — Functions behind carrier-grade NAT
- **TCP fallback** — TLS over TCP works when UDP is blocked

## Development Status

This project is currently in **early development**.

### Completed

- ✅ `vpn-client/Cargo.toml` configured with rustls (aws-lc-rs backend) for native X25519MLKEM768
- ✅ `vpn-client/src/main.rs` with CLI argument parsing, TLS configuration skeleton, and certificate loading
- ✅ Unit test: `post_quantum_key_exchange_is_offered_first` — verifies X25519MLKEM768 is the first key exchange group
- ✅ Unit test: `server_cert_replaces_the_public_roots` — verifies custom certificate loading behavior
- ✅ Project structure with separate client and server directories
- ✅ `.gitignore` configured for build artifacts and sensitive files

### In Progress

- 🔄 Server connection implementation (`server.rs`)
- 🔄 Tunneling module design

### Planned

- ❌ `vpn-server` — Convert to VPN server application with rustls
- ❌ TUN interface implementation (`tun.rs`)
- ❌ SOCKS5 proxy implementation (`socks5.rs`)
- ❌ Application-layer protocol messages (`types.rs`)
- ❌ Authentication system (LDAP/OIDC)
- ❌ Docker/Proxmox deployment
- ❌ Multi-platform clients

### Next Steps (Priority Order)

1. **`vpn-server`** — Convert to VPN server application with rustls
2. **`server.rs`** — Implement server connection with TLS 1.3 (X25519MLKEM768)
3. **`types.rs`** — Define application-layer protocol message types
4. **`tun.rs`** — Implement TUN interface packet capture and forwarding
5. **`socks5.rs`** — Implement SOCKS5 proxy forwarding
6. **Integration test** — End-to-end test with self-signed certificate and local TLS server

## References

- **NIST Post-Quantum Cryptography**: https://csrc.nist.gov/projects/post-quantum-cryptography
- **NIST ML-KEM (FIPS 203)**: https://csrc.nist.gov/pubs/fips/203/final
- **NIST ML-DSA (FIPS 204)**: https://csrc.nist.gov/pubs/fips/204/final
- **NIST SLH-DSA (FIPS 205)**: https://csrc.nist.gov/pubs/fips/205/final
- **CRYSTALS-Kyber**: https://pq-crystals.org/kyber/
- **CRYSTALS-Dilithium**: https://pq-crystals.org/dilithium/
- **rustls**: https://github.com/rustls/rustls
- **aws-lc-rs**: https://github.com/aws/aws-lc-rs (cryptographic backend with X25519MLKEM768 support)
- **X25519MLKEM768**: Hybrid key exchange combining X25519 (classical) with ML-KEM-768 (post-quantum)

## License

This project is licensed under the **Business Source License 1.1 (BUSL 1.1)** by **MDQuantum**.

See the [LICENSE](LICENSE) file for details.

On the Change Date (2029-11-17), the license will automatically change to **Apache-2.0**.

## Contact

- **Website**: [mdquantum.xyz](https://mdquantum.xyz)
