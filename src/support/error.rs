#![allow(dead_code)]

use anyhow::Error;

pub fn reason(err: &Error) -> String {
    if let Some(found) = err.downcast_ref::<wreq::Error>() {
        return transport(found);
    }
    if let Some(found) = err.downcast_ref::<tokio_tungstenite::tungstenite::Error>() {
        return socket(found);
    }
    sanitize(&err.to_string())
}

fn transport(err: &wreq::Error) -> String {
    if err.is_timeout() {
        return "the request timed out".to_string();
    }
    if err.is_connect() || err.is_dns() || err.is_connection_reset() {
        return "the connection could not be established".to_string();
    }
    if err.is_connect() || err.is_proxy_connect() || err.is_request() || err.is_tls() {
        return "the connection could not be established".to_string();
    }
    if err.is_body() || err.is_decode() {
        return "the reply could not be read".to_string();
    }
    if let Some(status) = err.status() {
        return format!("the server answered {}", status.as_u16());
    }
    if err.is_redirect() {
        return "the request was sent to another address".to_string();
    }
    if err.is_request() || err.is_tls() {
        return "the connection could not be established".to_string();
    }
    "the request failed before an answer arrived".to_string()
}

fn socket(err: &tokio_tungstenite::tungstenite::Error) -> String {
    match err {
        tokio_tungstenite::tungstenite::Error::Io(_)
        | tokio_tungstenite::tungstenite::Error::Tls(_)
        | tokio_tungstenite::tungstenite::Error::Protocol(_) => {
            "the connection could not be established".to_string()
        }
        _ => "the relay channel was closed".to_string(),
    }
}

pub fn sanitize(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<&str>>().join(" ");
    let kept: Vec<&str> = flat
        .split(' ')
        .filter(|word| !word.contains("://"))
        .collect();
    let joined = kept.join(" ");
    if joined.is_empty() {
        return "the request failed before an answer arrived".to_string();
    }
    if joined.chars().count() > 120 {
        let cut: String = joined.chars().take(120).collect();
        let trimmed = cut.rsplit_once(' ').map(|(head, _)| head).unwrap_or(&cut);
        return trimmed.trim().to_string();
    }
    joined
}

pub fn refusal(status: u16) -> &'static str {
    match status {
        400 | 401 | 422 => "the node credential was refused",
        403 => "the firewall refused the request",
        404 => "the route was not found",
        409 => "the request clashed with the current state",
        429 => "the server is limiting requests from this address",
        500..=599 => "the beacon gateway is not answering right now",
        _ => "the server refused the request",
    }
}

pub fn label(status: u16) -> &'static str {
    match status {
        400 | 401 | 422 => "a refused node credential",
        403 => "a firewall refusal",
        404 => "a missing route",
        409 => "a blocked state",
        429 => "a rate limit from the server",
        500..=599 => "no answer from the beacon gateway",
        _ => "a refusal from the server",
    }
}
