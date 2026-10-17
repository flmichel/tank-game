use serde::Deserialize;

use crate::balance::Upgrade;

#[derive(Debug)]
pub struct PlayerInput {
    pub socket_id: u32,
    pub remote_input: RemoteInput,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RemoteInput {
    Sticks(Sticks),
    GameInput(GameInput),
    ConfigurationInput(ConfigurationInput),
}

/// Full stick state, streamed at 30 Hz on the unreliable "inputs" channel:
/// `{"seq":12,"move":1.57,"aim":null}`, a `null` stick being released.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Sticks {
    pub seq: u32,
    #[serde(rename = "move")]
    pub movement: Option<f64>,
    pub aim: Option<f64>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum GameInput {
    // Carries its direction: on the reliable channel it can overtake the latest aim state
    Shoot(f64),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationInput {
    Ready,
    NotReady,
    SetName(String),
    /// `{"buy":"bullet-speed"}`, honoured between rounds only
    Buy(Upgrade),
    PlayerId(String),
}
