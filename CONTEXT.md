# Post-Quantum VPN Project

## Overview

This project implements a secure Virtual Private Network (VPN) solution using **post-quantum cryptographic algorithms** designed to resist attacks from quantum computers. It is developed in response to the **ARIU (Asociación de Redes de Interconexión Universitaria)** call for proposals, which seeks a production-ready quantum-resistant VPN for Argentine national universities.

The solution uses a **hybrid cryptographic approach**, combining classical algorithms with NIST-standardized post-quantum primitives to ensure security against both current and future quantum adversaries.

---

## Cryptographic Algorithms

The project follows the NIST post-quantum standards published in 2024:

| Algorithm | Standard | Type | Purpose |
|-----------|----------|------|---------|
| **CRYSTALS-Kyber (ML-KEM)** | FIPS 203 | Key Encapsulation Mechanism (KEM) | Quantum-resistant key exchange, replacing classical ECDH |
| **CRYSTALS-Dilithium (ML-DSA)** | FIPS 204 | Digital Signatures | Quantum-resistant authentication, replacing RSA/ECDSA |
| **SPHINCS+** | FIPS 205 | Hash-based Signatures | Backup signature algorithm for crypto-agility |
| **AES-256-GCM** | NIST AES | Symmetric Encryption | Tunnel data encryption |
| **SHA-3** | FIPS 202 | Hash Function | Used inside ML-KEM and ML-DSA (TLS 1.3 derives its keys with HKDF over SHA-2) |

### Hybrid Approach

The protocol uses **hybrid key exchange** combining classical ECDH with CRYSTALS-Kyber. This ensures:
- **Forward compatibility**: Security even if classical algorithms are broken by quantum computers
- **Backward compatibility**: Security even if unknown vulnerabilities exist in post-quantum algorithms
- **Transition safety**: A smooth migration path from classical to post-quantum cryptography

---

## Project Structure

```
vpn/
├── vpn-client/           # VPN client application (Rust console app)
│   ├── Cargo.toml        # Dependencies including post-quantum crypto libraries
│   └── src/
│       └── main.rs       # Client entry point with CLI argument parsing
├── vpn-server/           # VPN server application (to be implemented)
│   ├── Cargo.toml        # Server dependencies
│   └── src/
│       └── main.rs       # Server entry point
├── .gitignore            # Git ignore rules (PDFs, build artifacts, IDE files)
└── CONTEXT.md            # This file
```

---

## Architecture

### Client-Side (`vpn-client`)

The VPN client is a Rust console application that establishes encrypted tunnels to the VPN server. It supports two operational modes:

1. **SOCKS5 Proxy**: Application-level traffic redirection. Users configure specific applications to route traffic through the proxy, which forwards it through the encrypted VPN tunnel.

2. **TUN Interface**: Full IP-layer tunneling. The client creates a virtual network interface (TUN) that captures all IP traffic and routes it through the encrypted tunnel, providing complete network redirection.

#### Client Modules (Planned)

| Module | Purpose | Status |
|--------|---------|--------|
| `server.rs` | TLS + post-quantum hybrid connection to the VPN server, including handshake and key negotiation | ❌ Not started |
| `socks5.rs` | SOCKS5 proxy server for application-level traffic forwarding through the VPN tunnel | ❌ Not started |
| `tun.rs` | TUN interface creation and management for full IP-layer tunneling | ❌ Not started |
| `types.rs` | Protocol message formats, packet headers, and serialization structures | ❌ Not started |

There is no `crypto.rs` or `util.rs`: key exchange, authentication, key derivation and encryption are done by the TLS library (see Protocol Design).

### Server-Side (`vpn-server`)

The VPN server manages incoming client connections, performs authentication, and routes traffic to the university network. It will support:

- **Docker container** deployment
- **Proxmox VM** deployment
- **Scripted installation** on Debian and Ubuntu

#### Server Modules (Planned)

| Module | Purpose | Status |
|--------|---------|--------|
| `handler.rs` | Connection handler for managing multiple concurrent client sessions | ❌ Not started |
| `auth.rs` | Authentication integration with LDAP/Active Directory and OpenID Connect | ❌ Not started |
| `routing.rs` | Traffic routing between VPN clients and university network resources | ❌ Not started |
| `config.rs` | Server configuration management | ❌ Not started |

---

## Protocol Design

### Handshake Flow

The handshake is plain TLS 1.3. Its post-quantum parts are standard features of rustls with its default backend (aws-lc-rs). There is no second, home-made key exchange or encryption layer on top:

1. **TLS 1.3 Connection**: Client establishes a TLS 1.3 connection with the server
2. **Post-Quantum Key Exchange**: The TLS handshake itself uses the hybrid group `X25519MLKEM768` (X25519 + ML-KEM-768, FIPS 203), which rustls offers first
3. **Authentication**: Server authenticates with its TLS certificate; rustls can verify ML-DSA (FIPS 204) certificates
4. **Key Derivation**: Done by TLS 1.3 from the hybrid shared secret
5. **Tunnel Establishment**: Tunnel data travels inside the TLS connection, encrypted by TLS (rustls prefers AES-256-GCM)

### Message Format

Protocol messages use **postcard** (compact binary serialization) for efficient encoding:

```
[Message Type (1 byte)] [Payload (variable)]
```

TLS already orders and authenticates the data, so messages carry no sequence number or MAC of their own.

Message types include:
- `0x01`: Handshake request
- `0x02`: Handshake response
- `0x03`: Data packet
- `0x04`: Keepalive
- `0x05`: Control message
- `0xFF`: Error

---

## ARIU Requirements Compliance

### Phase 1 — Design ✅ (In Progress)

| Requirement | Status | Notes |
|-------------|--------|-------|
| Protocol VPN analysis and selection | ✅ Done | TLS 1.3 with its standard hybrid PQC key exchange (`X25519MLKEM768`); no custom crypto |
| PQC support justification | ✅ Done | ML-KEM + ML-DSA hybrid approach |
| Authentication architecture | ❌ Not started | LDAP/OIDC integration planned |
| Client strategy | ✅ Done | Rust client with SOCKS5 + TUN modes |
| Provisioning strategy | ❌ Not started | Web portal planned |

### Phase 2 — Server Implementation ❌

| Requirement | Status | Notes |
|-------------|--------|-------|
| Functional VPN server | ❌ Not started | `vpn-server` module needs implementation |
| Authentication system | ❌ Not started | LDAP/OIDC integration |
| Test directory integration | ❌ Not started | |
| At least one deployment platform | ❌ Not started | Docker, Proxmox, or scripted install |

### Phase 3 — Client & Provisioning ❌

| Requirement | Status | Notes |
|-------------|--------|-------|
| Multi-platform clients | ❌ Not started | Windows, macOS, Linux, Android, iOS |
| Self-provisioning system | ❌ Not started | Web-based credential portal |

### Phase 4 — Hardening & Final Delivery ❌

| Requirement | Status | Notes |
|-------------|--------|-------|
| Security review | ❌ Not started | |
| Technical report | ❌ Not started | |
| Deployment repository | ❌ Not started | |

---

## Key Technical Requirements

### Network Compatibility
- **Dual-stack IPv4/IPv6**: Server accepts connections on both address families; tunnel transports both simultaneously
- **NAT traversal**: Works behind standard home NAT routers
- **CGNAT support**: Functions correctly when client is behind carrier-grade NAT
- **UDP blockage fallback**: TLS over TCP provides fallback when UDP is blocked by institutional networks, hotel networks, or captive portals

### Authentication
- **Centralized authentication**: Decoupled from VPN server
- **Directory compatibility**: LDAP / Active Directory
- **Federated identity**: SAML 2.0 or OpenID Connect
- **Multi-factor authentication (MFA)**: Configurable by administrator
- **User lifecycle management**: User creation, deletion, and modification without manual VPN server intervention

### High Availability
- The design must address primary node failure scenarios
- Redundancy or recovery mechanisms should be documented

### Credential Rotation
- Procedures for cryptographic key updates without service interruption

---

## Current Development Status

### Completed
- ✅ `vpn-client/Cargo.toml` with only the dependencies the code uses (the post-quantum key exchange comes with rustls)
- ✅ `vpn-client/src/main.rs` with CLI argument parsing and TLS configuration skeleton
- ✅ Project structure with separate client and server directories
- ✅ `.gitignore` configured for build artifacts and PDF files

### In Progress
- 🔄 Tunneling module design and implementation planning

### Next Steps (Priority Order)
1. **`types.rs`** — Define protocol message types and serialization formats
2. **`server.rs`** — Implement server connection with post-quantum handshake
3. **`tun.rs`** — Implement TUN interface packet capture and forwarding
4. **`socks5.rs`** — Implement SOCKS5 proxy forwarding
5. **`vpn-server`** — Convert from NSH template to VPN server application

---

## Building and Running

### Prerequisites
- Rust toolchain (edition 2024)
- Linux system with TUN device support (`/dev/net/tun`)

### Build
```bash
cd vpn-client
cargo build
```

### Run (Current Skeleton)
```bash
# Basic usage
./target/debug/vpn-client --port 1080 --server-domain vpn.example.com

# With custom server port
./target/debug/vpn-client --port 1080 --server-domain vpn.example.com --server-port 8443

# With custom server certificate
./target/debug/vpn-client --port 1080 --server-domain vpn.example.com --server-cert server.pem
```

---

## References

- **ARIU Call for Proposals**: `convocatoria_proyecto_VPN.pdf`
- **NIST ML-KEM (FIPS 203)**: https://csrc.nist.gov/pubs/fips/203/final
- **NIST ML-DSA (FIPS 204)**: https://csrc.nist.gov/pubs/fips/204/final
- **NIST SLH-DSA (FIPS 205)**: https://csrc.nist.gov/pubs/fips/205/final
- **CRYSTALS-Kyber**: https://pq-crystals.org/kyber/
- **CRYSTALS-Dilithium**: https://pq-crystals.org/dilithium/
