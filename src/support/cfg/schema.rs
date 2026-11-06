use serde::Deserialize;

fn default_sleep() -> u64 {
    3600
}

fn default_retry() -> u32 {
    3
}

fn default_heartbeat() -> u64 {
    60
}

#[derive(Clone, Deserialize)]
pub struct Settings {
    #[serde(default = "default_sleep")]
    pub sleep_seconds: u64,
    #[serde(default = "default_retry")]
    pub max_retries: u32,
    #[serde(default = "default_heartbeat")]
    pub heartbeat_minutes: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            sleep_seconds: default_sleep(),
            max_retries: default_retry(),
            heartbeat_minutes: default_heartbeat(),
        }
    }
}

#[derive(Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
}
