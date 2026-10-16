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
