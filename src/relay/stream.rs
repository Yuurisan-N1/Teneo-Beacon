use std::collections::HashMap;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{client_async_tls_with_config, MaybeTlsStream, WebSocketStream};

use super::frames::{self, OP_CLOSE, OP_CONNECT, OP_DATA};
use crate::net::proxy::Proxy;
use crate::support::consts::{app, net};
use crate::support::log::lg;

pub type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn open(key: &str, token: &str, email: &str, proxy: Option<&Proxy>) -> Result<Ws> {
    let mut ws = dial(key, proxy).await?;
    let hello = json!({
        "requestId": uuid::Uuid::new_v4().to_string(),
        "access_token": token,
        "account_id": email,
    });
    ws.send(Message::Text(hello.to_string().into())).await?;
    Ok(ws)
}

async fn dial(key: &str, proxy: Option<&Proxy>) -> Result<Ws> {
    let mut request = net::WS.into_client_request()?;
    {
        let head = request.headers_mut();
        head.insert("Authorization", HeaderValue::from_str(key)?);
        head.insert("x-api-key", HeaderValue::from_static(app::API_KEY));
        head.insert("Origin", HeaderValue::from_static(app::ORIGIN));
        head.insert("User-Agent", HeaderValue::from_static(app::UA));
    }
    let (host, port) = target()?;
    let tcp = match proxy {
        Some(found) => crate::net::tunnel::connect(found, &host, port).await?,
        None => crate::net::tunnel::direct(&host, port).await?,
    };
    let tls = Some(tokio_tungstenite::Connector::Rustls(
        crate::net::tunnel::config()?,
    ));
    let (ws, _) = client_async_tls_with_config(request, tcp, None, tls).await?;
    Ok(ws)
}

fn target() -> Result<(String, u16)> {
    let raw = net::WS;
    let rest = raw.strip_prefix("wss://").unwrap_or(raw);
    let authority = rest.split('/').next().unwrap_or_default();
    let (host, port) = match authority.rsplit_once(':') {
        Some((head, tail)) => (head.to_string(), tail.parse::<u16>()?),
        None => (authority.to_string(), 443),
    };
    Ok((host, port))
}

pub async fn serve(mut ws: Ws) -> Option<u16> {
    let (out_tx, mut out_rx) = mpsc::channel::<Vec<u8>>(512);
    let mut streams: HashMap<u32, mpsc::Sender<Vec<u8>>> = HashMap::new();
    loop {
        tokio::select! {
            incoming = ws.next() => {
                match incoming {
                    Some(Ok(Message::Binary(data))) => {
                        if let Some(frame) = frames::decode(&data) {
                            match frame.opcode {
                                OP_CONNECT => {
                                    let payload: Value =
                                        serde_json::from_slice(&frame.payload).unwrap_or(Value::Null);
                                    let host = payload
                                        .get("host")
                                        .and_then(Value::as_str)
                                        .unwrap_or_default()
                                        .to_string();
                                    let port = payload
                                        .get("port")
                                        .and_then(Value::as_u64)
                                        .unwrap_or(0) as u16;
                                    if host.is_empty() || port == 0 {
                                        continue;
                                    }
                                    let (sink, inbox) = mpsc::channel::<Vec<u8>>(256);
                                    streams.insert(frame.stream, sink);
                                    spawn_tcp(frame.stream, host, port, out_tx.clone(), inbox);
                                }
                                OP_DATA => {
                                    if let Some(sink) = streams.get(&frame.stream) {
                                        let _ = sink.send(frame.payload).await;
                                    }
                                }
                                OP_CLOSE => {
                                    streams.remove(&frame.stream);
                                }
                                frames::OP_CONTROL => {}
                                _ => {}
                            }
                        }
                    }
                    Some(Ok(Message::Text(text))) => report(&text),
                    Some(Ok(Message::Close(frame))) => {
                        return frame.map(|found| u16::from(found.code));
                    }
                    None => return None,
                    Some(Err(_)) => return None,
                    _ => {}
                }
            }
            outgoing = out_rx.recv() => {
                match outgoing {
                    Some(chunk) => {
                        if ws.send(Message::Binary(chunk.into())).await.is_err() {
                            return None;
                        }
                    }
                    None => return None,
                }
            }
        }
    }
}

fn report(text: &str) {
    let body = text.trim_start_matches(|c: char| (c as u32) < 0x20);
    let value: Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return,
    };
    let kind = value
        .get("type")
        .or_else(|| value.get("event"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    match kind {
        "points-update" => {
            if let Some(points) = value.get("points").and_then(Value::as_u64) {
                lg(&format!("The gateway credited the account with {} points", points));
            }
        }
        "beacon-update" => {
            if let Some(fragments) = value.get("unclaimedFragments").and_then(Value::as_u64) {
                lg(&format!(
                    "The gateway reports {} unclaimed fragments for the account",
                    fragments
                ));
            }
        }
        _ => {}
    }
}

fn spawn_tcp(
    id: u32,
    host: String,
    port: u16,
    out: mpsc::Sender<Vec<u8>>,
    mut inbox: mpsc::Receiver<Vec<u8>>,
) {
    tokio::spawn(async move {
        let tcp = match super::socket::dial(&host, port).await {
            Ok(stream) => stream,
            Err(_) => {
                let _ = out.send(frames::encode(id, OP_CLOSE, b"")).await;
                return;
            }
        };
        let (mut reader, mut writer) = tcp.into_split();
        let mut buf = vec![0u8; 16_384];
        loop {
            tokio::select! {
                read = reader.read(&mut buf) => {
                    match read {
                        Ok(0) | Err(_) => break,
                        Ok(size) => {
                            if out
                                .send(frames::encode(id, OP_DATA, &buf[..size]))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                }
                data = inbox.recv() => {
                    match data {
                        Some(chunk) => {
                            if writer.write_all(&chunk).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
        let _ = out.send(frames::encode(id, OP_CLOSE, b"")).await;
    });
}
