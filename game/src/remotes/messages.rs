use serde::Deserialize;
use specs::Component;
use specs::VecStorage;
use specs_derive::Component;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct PlayerInput {
    pub socket_id: u32,
    pub remote_input: RemoteInput,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RemoteInput {
    GameInput(GameInput),
    ConfigurationInput(ConfigurationInput),
    NoInput,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum GameInput {
    Aim(f64),
    Shoot,
    Stop,
    Move(f64),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationInput {
    Ready,
    NotReady,
    SetName(String),
    PlayerId(String),
}
