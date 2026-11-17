# MDQuantum VPN client

Rust client under development for a VPN using hybrid post-quantum TLS.

## Implemented

- Command-line configuration and PEM trust roots (custom roots replace public roots).
- Actual TCP/TLS connection with a 10-second deadline covering DNS, TCP and TLS.
- TLS 1.3 only, with X25519MLKEM768 as the only allowed key exchange group.
- Certificate chain and hostname verification through rustls.
- Negotiated TLS version and key exchange logged after connection.
- Local development echo server and tests of encrypted data exchange and rejection cases.

The CLI listens for SOCKS5 on `127.0.0.1:1080` (configurable with `--port`).
Each connection uses a separate hybrid TLS session to the relay; DNS resolution
happens at the relay. Up to 128 sessions are admitted concurrently.

The development relay supports CONNECT with IPv4, IPv6 and domain addresses,
and restricts destinations to loopback. UDP ASSOCIATE and BIND are rejected.
It has no client authentication and must remain loopback-only. It is sequential
and limits each development session to 10 seconds. Production authentication,
destination authorization, idle timeouts and a production gateway remain pending.
There is no TUN interface or system-wide routing. Hybrid key exchange does not
imply post-quantum certificate signatures.

## Try locally

Requires a Rust toolchain supporting edition 2024 and the native build tools
needed by aws-lc-rs (including a C compiler and CMake).

From this repository root, start the loopback-only test server:

```sh
cargo run --locked --example socks-server
```

It listens on `127.0.0.1:8443` and prints the path of an ephemeral public
certificate. Use that path in another terminal:

```sh
cargo run --locked --bin vpn-client -- \
  --server-domain localhost --server-port 8443 \
  --server-cert /tmp/mdquantum-dev-PID.pem
```

Replace `PID` with the value printed by the server. The private key remains in
memory. Stop the example with Ctrl+C; its public PEM can then be deleted.
For a local HTTP demonstration, start `python3 -m http.server 18080 --bind 127.0.0.1`
in a directory containing only test files, then run:

```sh
curl --noproxy "" --socks5-hostname 127.0.0.1:1080 http://localhost:18080/
```

The original `echo-server` example remains available for raw TLS experiments.

## Verify

```sh
cargo build --locked --all-targets
python3 tests/socks_e2e.py
cargo fmt --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
```

Tests cover hybrid TLS 1.3 data exchange, rejection of classical-only key
exchange, TLS 1.2, untrusted certificates and hostname mismatch, plus certificate
loading and configured key exchange preference.

## Next milestones

1. Agree on server authentication, destination permissions and a bounded message format.
2. Extend the development relay into an authenticated, concurrent production gateway.
3. Add TUN, routing, DNS, IPv6 and recovery according to product requirements.
4. Validate the server endpoint with the separate `mdq` scanner.

TLS encryption and integrity are delegated to rustls and aws-lc-rs.

## License

See [LICENSE](LICENSE) for the project's license terms.
