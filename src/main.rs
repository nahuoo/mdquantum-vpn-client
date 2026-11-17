//! Post-Quantum VPN Client
//!
//! A secure VPN client that establishes encrypted tunnels using
//! quantum-resistant cryptographic primitives designed to defend
//! against adversaries with quantum computing capabilities.
//!
//! # Cryptographic Algorithms
//!
//! - **CRYSTALS-Kyber (ML-KEM)**: Post-quantum key encapsulation mechanism
//!   for establishing shared secrets, replacing classical ECDH.
//! - **CRYSTALS-Dilithium (ML-DSA)**: Post-quantum digital signatures
//!   for authentication, replacing RSA/ECDSA.
//! - **AES-256-GCM**: Symmetric encryption for tunnel data.
//! - **SHA-3**: Hash-based key derivation and integrity verification.
//!
//! # Operational Modes
//!
//! - **SOCKS5 Proxy**: Application-level traffic redirection for selective
//!   tunneling of specific applications.
//! - **TUN Interface**: Full IP-layer tunneling redirecting all network
//!   traffic through the encrypted VPN tunnel.
//!
//! # Usage
//!
//! ```text
//! vpn-client --port 1080 --server-domain vpn.example.com
//! vpn-client --port 1080 --server-domain vpn.example.com --server-port 8443
//! vpn-client --port 1080 --server-domain vpn.example.com --server-cert server.pem
//! ```

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use rustls::{
    ClientConfig, RootCertStore,
    pki_types::{CertificateDer, pem::PemObject},
};
use tracing::info;
use tracing_forest::ForestLayer;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

// ---------------------------------------------------------------------------
// Crypto provider initialization for rustls 0.23+
// ---------------------------------------------------------------------------
// rustls 0.23 requires a CryptoProvider to be installed before creating
// ClientConfig. We use the `ring` provider, which supports the same
// algorithms as the default aws-lc-rs provider but without FIPS mode.
fn init_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

// ---------------------------------------------------------------------------
// Post-quantum cryptographic module placeholders
// ---------------------------------------------------------------------------
// These modules will implement the core post-quantum protocol logic.
// They are currently stubs to allow compilation while the implementation
// is developed incrementally.

/// Post-quantum key exchange using CRYSTALS-Kyber.
///
/// Implements the client-side of the Kyber key encapsulation mechanism
/// for establishing shared secrets with the VPN server. Supports Kyber512,
/// Kyber768, and Kyber1024 security levels.
// TODO: Implement Kyber-based key exchange with server
// mod crypto;

/// VPN server connection handler.
///
/// Manages the TLS + post-quantum hybrid connection to the VPN server,
/// including handshake, key negotiation, and encrypted data tunneling.
// TODO: Implement server connection with post-quantum handshake
// mod server;

/// SOCKS5 proxy server for application-level traffic.
///
/// Listens for incoming SOCKS5 connections and forwards traffic through
/// the encrypted VPN tunnel.
// TODO: Implement SOCKS5 proxy forwarding through VPN tunnel
// mod socks5;

/// TUN interface for full IP-layer tunneling.
///
/// Creates and manages a TUN network interface to capture and forward
/// all IP traffic through the encrypted VPN tunnel.
// TODO: Implement TUN interface packet handling and forwarding
// mod tun;

/// Shared types and data structures for the VPN protocol.
///
/// Defines message formats, packet headers, and protocol constants
/// used across client and server communication.
// TODO: Define protocol message types and serialization
// mod types;

/// Utility functions for cryptographic operations and packet parsing.
///
/// Helper functions for key derivation, packet encoding/decoding,
/// and other common operations.
// TODO: Implement crypto helpers and packet utilities
// mod util;

// use crate::{server::Server, socks5::Socks5Server, tun::Tun, types::ProxySender};

/// Command-line arguments for the VPN client.
///
/// Provides configuration options for connecting to a VPN server
/// and setting up local proxy interfaces.
#[derive(Parser, Debug)]
#[command(name = "vpn-client")]
#[command(about = "A post-quantum VPN client using CRYSTALS-Kyber and CRYSTALS-Dilithium")]
#[command(long_about = None)]
struct CliArgs {
    /// SOCKS5 proxy port to listen on for incoming connections.
    ///
    /// Applications configured to use this proxy will have their traffic
    /// routed through the encrypted VPN tunnel.
    #[arg(short, long, default_value = "1080")]
    port: u16,

    /// Path to a .pem encoded public certificate for the server.
    ///
    /// If not provided, uses system root certificates for TLS verification.
    /// Use this option when connecting to a server with a self-signed
    /// or custom CA certificate.
    #[arg(long)]
    server_cert: Option<PathBuf>,

    /// Server domain name to connect to.
    ///
    /// This is used for TLS SNI (Server Name Indication) and hostname
    /// verification during the TLS handshake.
    #[arg(short = 'd', long)]
    server_domain: String,

    /// Server port to connect to.
    ///
    /// Defaults to 443 (standard HTTPS port) for TLS-based connections.
    #[arg(long, default_value = "443")]
    server_port: u16,
}

/// Entry point for the post-quantum VPN client.
///
/// Initializes the logging system, parses command-line arguments,
/// configures the TLS client with post-quantum capabilities, and
/// prepares to start the SOCKS5 proxy and TUN interface tasks.
///
/// # Post-Quantum Security
///
/// The client uses a hybrid key exchange combining classical ECDH
/// with CRYSTALS-Kyber to ensure forward secrecy and quantum
/// resistance. Digital signatures use CRYSTALS-Dilithium for
/// server authentication.
///
/// # Returns
///
/// * `Ok(())` - Client initialized successfully (modules pending implementation).
/// * `Err(e)` - Failed to initialize (e.g., invalid certificate, config error).
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the rustls crypto provider before any TLS configuration.
    init_crypto_provider();

    // Initialize structured logging with forest visualization.
    // Set RUST_LOG=debug for verbose output, or RUST_LOG=trace for maximum detail.
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(ForestLayer::default())
        .init();

    let args = CliArgs::parse();

    info!("Initializing Post-Quantum VPN Client");
    info!("Target server: {}:{}", args.server_domain, args.server_port);
    info!("SOCKS5 proxy port: {}", args.port);

    // -----------------------------------------------------------------------
    // TLS Configuration
    // -----------------------------------------------------------------------
    // Set up the root certificate store for TLS 1.3 verification.
    // Start with Mozilla's trusted root CAs, then optionally add custom certs.
    let mut root_cert_store = RootCertStore::empty();
    for cert in webpki_roots::TLS_SERVER_ROOTS.iter() {
        root_cert_store.add_trust_anchor(cert.clone()).expect("valid trust anchor");
    }

    // Optionally add a custom server certificate for self-signed or private CA setups.
    if let Some(server_cert_path) = &args.server_cert {
        info!("Loading custom server certificate: {:?}", server_cert_path);
        root_cert_store.add(CertificateDer::from_pem_file(server_cert_path)?)?;
    }

    // Build the TLS client configuration.
    // TODO: Integrate post-quantum key exchange (Kyber) into TLS handshake
    // via custom TLS extensions or application-layer hybrid protocol.
    let _config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    // -----------------------------------------------------------------------
    // Post-Quantum Key Exchange Setup
    // -----------------------------------------------------------------------
    // TODO: Initialize CRYSTALS-Kyber key pair for encapsulation
    // let kyber_keypair = pqcrypto::kyber::keypair(&mut rand::thread_rng());
    // let (kyber_public, kyber_secret) = (kyber_keypair.pk, kyber_keypair.sk);

    // TODO: Initialize CRYSTALS-Dilithium key pair for signatures
    // let dilithium_keypair = pqcrypto::dilithium::keypair(&mut rand::thread_rng());

    // -----------------------------------------------------------------------
    // Server Connection
    // -----------------------------------------------------------------------
    // TODO: Establish TLS connection with post-quantum hybrid handshake
    // let server = Arc::new(Server::new(
    //     args.server_domain.clone(),
    //     args.server_port,
    //     _config,
    //     kyber_public,
    // )?);

    // -----------------------------------------------------------------------
    // SOCKS5 Proxy Server
    // -----------------------------------------------------------------------
    // TODO: Start SOCKS5 proxy to forward application traffic through VPN
    // let s = server.clone();
    // let h_socks5 = async move {
    //     let socks5 = Socks5Server::new(s, "127.0.0.1".to_string(), args.port);
    //     socks5.server_start().await?;
    //     Ok::<(), anyhow::Error>(())
    // };

    // -----------------------------------------------------------------------
    // TUN Interface
    // -----------------------------------------------------------------------
    // TODO: Create and configure TUN interface for full IP tunneling
    // let tun = Tun {
    //     server: server.clone(),
    //     addr: (10, 8, 0, 2),
    //     netmask: (255, 255, 255, 0),
    //     mtu: 1400,
    // };
    // let h_tun = tun.server_start();

    // -----------------------------------------------------------------------
    // Run Concurrent Tasks
    // -----------------------------------------------------------------------
    // TODO: Launch both SOCKS5 proxy and TUN interface concurrently
    // if let Err(e) = tokio::try_join!(h_socks5, h_tun) {
    //     tracing::error!(?e, "Fatal error in VPN tasks");
    //     return Err(e);
    // }

    // -----------------------------------------------------------------------
    // Status Output
    // -----------------------------------------------------------------------
    println!("\n=== Post-Quantum VPN Client ===");
    println!("Server:        {}:{}", args.server_domain, args.server_port);
    println!("SOCKS5 Proxy:  127.0.0.1:{}", args.port);
    println!("Crypto:        CRYSTALS-Kyber (KEM) + CRYSTALS-Dilithium (Signatures)");
    println!("Status:        Initialized (modules pending implementation)");
    println!("===============================\n");
    println!("TODO: Implement post-quantum tunnel modules:");
    println!("  - crypto.rs  : Kyber/Dilithium key exchange and signatures");
    println!("  - server.rs  : TLS + post-quantum hybrid connection");
    println!("  - socks5.rs  : SOCKS5 proxy forwarding");
    println!("  - tun.rs     : TUN interface packet handling");
    println!("  - types.rs   : Protocol message definitions");
    println!("  - util.rs    : Crypto helpers and packet utilities");

    Ok(())
}
