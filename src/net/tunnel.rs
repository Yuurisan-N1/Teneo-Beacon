use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};

use super::proxy::Proxy;

pub fn config() -> Result<Arc<ClientConfig>> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    Ok(Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    ))
}

pub async fn direct(host: &str, port: u16) -> Result<TcpStream> {
    Ok(TcpStream::connect((host, port)).await?)
}

pub async fn connect(proxy: &Proxy, host: &str, port: u16) -> Result<TcpStream> {
    let mut tcp = TcpStream::connect((proxy.host.as_str(), proxy.port)).await?;
    match proxy.scheme.as_str() {
        "http" | "https" => http_tunnel(&mut tcp, proxy, host, port).await?,
        _ => socks5(&mut tcp, proxy, host, port).await?,
    }
    Ok(tcp)
}

async fn http_tunnel(
    tcp: &mut TcpStream,
    proxy: &Proxy,
    host: &str,
    port: u16,
) -> Result<()> {
    let mut head = format!(
        "CONNECT {host}:{port} HTTP/1.1\r\nHost: {host}:{port}\r\nProxy-Connection: keep-alive\r\n"
    );
    if let (Some(user), Some(pass)) = (proxy.user.as_ref(), proxy.pass.as_ref()) {
        let token = base64(format!("{user}:{pass}").as_bytes());
        head.push_str(&format!("Proxy-Authorization: Basic {token}\r\n"));
    }
    head.push_str("\r\n");
    tcp.write_all(head.as_bytes()).await?;
    let reply = read_head(tcp).await?;
    let line = reply.lines().next().unwrap_or_default();
    if !line.contains(" 200") {
        return Err(anyhow!("proxy CONNECT refused the tunnel"));
    }
    Ok(())
}

async fn socks5(tcp: &mut TcpStream, proxy: &Proxy, host: &str, port: u16) -> Result<()> {
    let auth = proxy.user.is_some() && proxy.pass.is_some();
    let greeting: [u8; 4] = if auth {
        [0x05, 0x02, 0x00, 0x02]
    } else {
        [0x05, 0x01, 0x00, 0x00]
    };
    tcp.write_all(&greeting).await?;
    let mut picked = [0u8; 2];
    tcp.read_exact(&mut picked).await?;
    if picked[0] != 0x05 || picked[1] == 0xFF {
        return Err(anyhow!("proxy SOCKS5 refused the greeting"));
    }
    if picked[1] == 0x02 {
        let user = proxy.user.clone().unwrap_or_default().into_bytes();
        let pass = proxy.pass.clone().unwrap_or_default().into_bytes();
        let mut frame = vec![0x01, user.len() as u8];
        frame.extend_from_slice(&user);
        frame.push(pass.len() as u8);
        frame.extend_from_slice(&pass);
        tcp.write_all(&frame).await?;
        let mut answer = [0u8; 2];
        tcp.read_exact(&mut answer).await?;
        if answer[1] != 0x00 {
            return Err(anyhow!("proxy SOCKS5 refused the credentials"));
        }
    }

    let name = host.as_bytes();
    let mut frame = vec![0x05, 0x01, 0x00, 0x03, name.len() as u8];
    frame.extend_from_slice(name);
    frame.extend_from_slice(&port.to_be_bytes());
    tcp.write_all(&frame).await?;

    let mut head = [0u8; 4];
    tcp.read_exact(&mut head).await?;
    if head[1] != 0x00 {
        return Err(anyhow!("proxy SOCKS5 refused the tunnel"));
    }
    let skip = match head[3] {
        0x01 => 4,
        0x04 => 16,
        0x03 => {
            let mut len = [0u8; 1];
            tcp.read_exact(&mut len).await?;
            len[0] as usize
        }
        _ => return Err(anyhow!("proxy SOCKS5 returned an unknown address")),
    };
    let mut scratch = vec![0u8; skip + 2];
    tcp.read_exact(&mut scratch).await?;
    Ok(())
}

async fn read_head(tcp: &mut TcpStream) -> Result<String> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        let size = tcp.read(&mut chunk).await?;
        if size == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..size]);
        if buf.windows(4).any(|pass| pass == b"\r\n\r\n") || buf.len() > 16_384 {
            break;
        }
    }
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn base64(raw: &[u8]) -> String {
    const SET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((raw.len() + 2) / 3 * 4);
    for block in raw.chunks(3) {
        let first = block[0] as u32;
        let second = *block.get(1).unwrap_or(&0) as u32;
        let third = *block.get(2).unwrap_or(&0) as u32;
        let joined = (first << 16) | (second << 8) | third;
        out.push(SET[((joined >> 18) & 0x3F) as usize] as char);
        out.push(SET[((joined >> 12) & 0x3F) as usize] as char);
        out.push(if block.len() > 1 {
            SET[((joined >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if block.len() > 2 {
            SET[(joined & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}