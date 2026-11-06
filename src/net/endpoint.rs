use crate::support::consts::net;

pub fn state() -> String {
    format!("{}/api/beacon/state", net::GATEWAY)
}

pub fn fleet() -> String {
    format!("{}/api/beacon/fleet", net::GATEWAY)
}

pub fn challenge() -> String {
    format!("{}/api/beacon/challenge", net::GATEWAY)
}

pub fn boost() -> String {
    format!("{}/api/beacon/boost", net::GATEWAY)
}

pub fn register() -> String {
    format!("{}/node/self-register", net::GATEWAY)
}

pub fn heartbeat() -> String {
    format!("{}/api/heartbeat", net::API)
}

pub fn points() -> String {
    format!("{}/api/points", net::API)
}
