#![allow(dead_code)]

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct Proxy {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub user: Option<String>,
    pub pass: Option<String>,
}

impl Proxy {
    pub fn parse(line: &str) -> Result<Proxy> {
        let raw = line.trim();
        if raw.is_empty() {
            return Err(anyhow!("proxy.txt holds an empty line"));
        }

        let (scheme, rest) = match raw.split_once("://") {
            Some((head, tail)) => (head.to_ascii_lowercase(), tail.to_string()),
            None => ("socks5".to_string(), raw.to_string()),
        };
        if !matches!(
            scheme.as_str(),
            "socks5" | "socks5h" | "socks" | "http" | "https"
        ) {
            return Err(anyhow!("proxy scheme \"{}\" is not supported", scheme));
        }

        let (creds, hostport) = match rest.rsplit_once('@') {
            Some((head, tail)) => (Some(head.to_string()), tail.to_string()),
            None => (None, rest),
        };
        if hostport.is_empty() {
            return Err(anyhow!("proxy.txt carries no host"));
        }

        let (host, port) = match hostport.rsplit_once(':') {
            Some((head, tail)) => {
                let port = tail
                    .parse::<u16>()
                    .map_err(|_| anyhow!("the proxy port is not a number"))?;
                (head.to_string(), port)
            }
            None => (hostport, 1080),
        };
        if host.is_empty() {
            return Err(anyhow!("proxy.txt carries no host"));
        }

        let (user, pass) = match creds {
            Some(raw) => match raw.split_once(':') {
                Some((head, tail)) => (Some(percent_decode(head)), Some(percent_decode(tail))),
                None => (Some(percent_decode(&raw)), None),
            },
            None => (None, None),
        };

        Ok(Proxy {
            scheme,
            host,
            port,
            user,
            pass,
        })
    }

    pub fn url(&self) -> String {
        let scheme = match self.scheme.as_str() {
            "socks" | "socks5h" => "socks5",
            other => other,
        };
        match (&self.user, &self.pass) {
            (Some(user), Some(pass)) => format!(
                "{}://{}:{}@{}:{}",
                scheme,
                percent_encode(user),
                percent_encode(pass),
                self.host,
                self.port
            ),
            _ => format!("{}://{}:{}", scheme, self.host, self.port),
        }
    }

    pub fn masked(&self) -> String {
        let octets: Vec<&str> = self.host.split('.').collect();
        if octets.len() == 4 {
            return format!(
                "http://user:pass@{}*****{}:{}",
                octets[0], octets[3], self.port
            );
        }
        if self.host.chars().count() > 4 {
            let head: String = self.host.chars().take(2).collect();
            let tail: String = self
                .host
                .chars()
                .rev()
                .take(2)
                .collect::<Vec<char>>()
                .into_iter()
                .rev()
                .collect();
            return format!("http://user:pass@{}*****{}:{}", head, tail, self.port);
        }
        "http://user:pass@***:***".to_string()
    }
}

pub fn pick(list: &[Proxy], index: usize) -> Option<&Proxy> {
    if list.is_empty() {
        return None;
    }
    list.get(index % list.len())
}

fn percent_encode(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            other => out.push_str(&format!("%{:02X}", other)),
        }
    }
    out
}

fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut idx = 0;
    while idx < bytes.len() {
        if bytes[idx] == b'%' && idx + 2 < bytes.len() {
            let hex = &raw[idx + 1..idx + 3];
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                out.push(byte);
                idx += 3;
                continue;
            }
        }
        out.push(bytes[idx]);
        idx += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masked_hides_the_middle() {
        let proxy = Proxy::parse("socks5://user:secret@192.0.2.10:10043").unwrap();
        assert_eq!(proxy.masked(), "http://user:pass@19*****10:10043");
        assert!(!proxy.masked().contains("secret"));
    }

    #[test]
    fn url_round_trips_the_credentials() {
        let proxy = Proxy::parse("socks5://yuuri:p%40ss@10.0.0.1:1080").unwrap();
        assert_eq!(proxy.url(), "socks5://yuuri:p%40ss@10.0.0.1:1080");
    }
}

pub fn load(path: &str) -> Result<Option<Proxy>> {
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    for line in raw.lines() {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        return Ok(Some(Proxy::parse(text)?));
    }
    Ok(None)
}

pub fn load_all(path: &str) -> Result<Vec<Proxy>> {
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    let mut out = Vec::new();
    for line in raw.lines() {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        match Proxy::parse(text) {
            Ok(found) => out.push(found),
            Err(_) => continue,
        }
    }
    Ok(out)
}
