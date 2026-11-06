use anyhow::Result;
use serde_json::Value;
use wreq::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use wreq::Client;

use crate::net::proxy::Proxy;
use crate::support::consts::app;

pub fn build(proxy: Option<&Proxy>) -> Result<Client> {
    let mut builder = Client::builder()
        .emulation(wreq_util::Emulation::OkHttp3_14)
        .timeout(std::time::Duration::from_secs(45))
        .gzip(true);
    if let Some(found) = proxy {
        builder = builder.proxy(wreq::Proxy::all(found.url())?);
    }
    Ok(builder.build()?)
}

pub fn plain() -> Result<HeaderMap> {
    let mut head = HeaderMap::new();
    head.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    head.insert(USER_AGENT, HeaderValue::from_static(app::UA));
    head.insert("Origin", HeaderValue::from_static(app::ORIGIN));
    head.insert("Referer", HeaderValue::from_static("http://tauri.localhost/"));
    head.insert("X-Requested-With", HeaderValue::from_static("teneo.node"));
    head.insert("x-api-key", HeaderValue::from_static(app::API_KEY));
    Ok(head)
}


pub fn bearer(token: &str) -> Result<HeaderMap> {
    let mut head = plain()?;
    head.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );
    Ok(head)
}

pub async fn get_json(client: &Client, url: &str, head: HeaderMap) -> Result<(u16, Value)> {
    let reply = client.get(url).headers(head).send().await?;
    let status = reply.status().as_u16();
    let text = reply.text().await?;
    let value = serde_json::from_str(&text).unwrap_or(Value::String(text));
    Ok((status, value))
}

pub async fn post_json(
    client: &Client,
    url: &str,
    head: HeaderMap,
    body: &Value,
) -> Result<(u16, Value)> {
    let reply = client.post(url).headers(head).json(body).send().await?;
    let status = reply.status().as_u16();
    let text = reply.text().await?;
    let value = serde_json::from_str(&text).unwrap_or(Value::String(text));
    Ok((status, value))
}
