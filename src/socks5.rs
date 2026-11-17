//! SOCKS5 CONNECT over one authenticated TLS connection per local client.
use anyhow::{Result, bail};
use rustls::{ClientConfig, pki_types::ServerName};
use std::{sync::Arc, time::Duration};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn listen(
    port: u16,
    name: ServerName<'static>,
    server_port: u16,
    config: ClientConfig,
) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    tracing::info!(port, "SOCKS5 listening on loopback");
    let limit = Arc::new(tokio::sync::Semaphore::new(128));
    loop {
        let permit = limit.clone().acquire_owned().await?;
        let (mut local, _) = listener.accept().await?;
        let name = name.clone();
        let config = config.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let result = async {
                let mut tls = crate::transport::connect(name, server_port, config).await?;
                tokio::io::copy_bidirectional(&mut local, &mut tls).await?;
                Ok::<_, anyhow::Error>(())
            }
            .await;
            if let Err(error) = result {
                tracing::warn!(%error, "SOCKS5 session ended");
            }
        });
    }
}

async fn reply<S: AsyncWrite + Unpin>(stream: &mut S, code: u8) -> Result<()> {
    stream.write_all(&[5, code, 0, 1, 0, 0, 0, 0, 0, 0]).await?;
    Ok(())
}

/// Development relay. Only loopback destinations are permitted; not a production gateway.
pub async fn serve_loopback<S: AsyncRead + AsyncWrite + Unpin>(stream: &mut S) -> Result<()> {
    let mut target = tokio::time::timeout(Duration::from_secs(10), negotiate(stream)).await??;
    tokio::io::copy_bidirectional(stream, &mut target).await?;
    Ok(())
}

async fn negotiate<S: AsyncRead + AsyncWrite + Unpin>(s: &mut S) -> Result<TcpStream> {
    if s.read_u8().await? != 5 {
        bail!("unsupported SOCKS version");
    }
    let n = s.read_u8().await? as usize;
    let mut methods = vec![0; n];
    s.read_exact(&mut methods).await?;
    if !methods.contains(&0) {
        s.write_all(&[5, 255]).await?;
        bail!("no supported authentication method");
    }
    s.write_all(&[5, 0]).await?;
    let version = s.read_u8().await?;
    let command = s.read_u8().await?;
    let reserved = s.read_u8().await?;
    let atyp = s.read_u8().await?;
    if version != 5 || reserved != 0 {
        reply(s, 1).await?;
        bail!("invalid request");
    }
    if command != 1 {
        reply(s, 7).await?;
        bail!("only CONNECT is supported");
    }
    let host = match atyp {
        1 => {
            let mut a = [0; 4];
            s.read_exact(&mut a).await?;
            std::net::Ipv4Addr::from(a).to_string()
        }
        4 => {
            let mut a = [0; 16];
            s.read_exact(&mut a).await?;
            std::net::Ipv6Addr::from(a).to_string()
        }
        3 => {
            let n = s.read_u8().await? as usize;
            let mut a = vec![0; n];
            s.read_exact(&mut a).await?;
            String::from_utf8(a)?
        }
        _ => {
            reply(s, 8).await?;
            bail!("unsupported address type");
        }
    };
    let port = s.read_u16().await?;
    // Resolve once on the server, then enforce policy on the exact addresses used.
    let addresses: Vec<_> = match tokio::net::lookup_host((host.as_str(), port)).await {
        Ok(a) => a.collect(),
        Err(e) => {
            reply(s, 4).await?;
            return Err(e.into());
        }
    };
    if addresses.is_empty() || addresses.iter().any(|a| !a.ip().is_loopback()) {
        reply(s, 2).await?;
        bail!("development relay only allows loopback destinations");
    }
    match TcpStream::connect(addresses.as_slice()).await {
        Ok(t) => {
            reply(s, 0).await?;
            Ok(t)
        }
        Err(e) => {
            reply(s, 5).await?;
            Err(e.into())
        }
    }
}
