//! Signaling messages exchanged with the server over its WebSocket.
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum ServerMessage {
    RoomId(String),
    SdpOffer(SdpMessage),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct SdpMessage {
    pub id: u32,
    pub data: String,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum MessageToServer {
    SdpAnswer(SdpMessage),
}

/// Latency probe from a remote on its "inputs" channel, echoed straight back by the shell:
/// `{"ping":1234.5}` → `{"pong":1234.5}`.
#[derive(Deserialize)]
pub struct Ping {
    pub ping: f64,
}

#[derive(Serialize)]
pub struct Pong {
    pub pong: f64,
}
