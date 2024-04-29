use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BotConfig {
    pub admin_pubkeys: Vec<String>,
    pub bot_names: Vec<String>,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelayConfig {
    pub write: Vec<String>,
    pub read: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub relay_servers: RelayConfig,
    pub bot: BotConfig,
}
