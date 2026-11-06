use std::net::IpAddr;

use anyhow::{anyhow, Result};
use tokio::net::TcpStream;

fn blocked(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.is_multicast()
        }
        IpAddr::V6(v6) => {
            let first = v6.segments()[0];
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
        }
    }
}

pub async fn dial(host: &str, port: u16) -> Result<TcpStream> {
    let addrs = tokio::net::lookup_host((host, port)).await?;
    for addr in addrs {
        if blocked(addr.ip()) {
            continue;
        }
        if let Ok(stream) = TcpStream::connect(addr).await {
            return Ok(stream);
        }
    }
    Err(anyhow!("the target host refused every address"))
}
