# vpn-client

Client of a post-quantum VPN for university networks (ARIU call). The design notes are in `CONTEXT.md`.

**Status: skeleton.** It checks its arguments and the TLS trust setup, then exits with "not implemented".
No tunnel is started yet.

```sh
cargo run -- --server-domain vpn.example.com                         # trusts the built-in Mozilla root list
cargo run -- --server-domain vpn.example.com --server-cert ca.pem    # trusts only the certificates in ca.pem
cargo test
```

Logs are at info level; set `RUST_LOG=debug` for more.

Post-quantum security comes from TLS 1.3 itself: the default rustls backend (aws-lc-rs) offers the hybrid key exchange
`X25519MLKEM768` (X25519 + ML-KEM-768, FIPS 203) first, and a test checks it. Do not turn on the rustls `ring` feature:
it has no ML-KEM.

To do, in this order: `types.rs` (messages between client and server), `server.rs` (TLS connection to the server),
`tun.rs` (TUN interface), `socks5.rs` (SOCKS5 proxy).
