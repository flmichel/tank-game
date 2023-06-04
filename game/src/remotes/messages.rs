use serde::Deserialize;
use specs::Component;
use specs::VecStorage;
use specs_derive::Component;

#[derive(Component)]
#[storage(VecStorage)]
pub struct PlayerInput {
    pub player_id: u32,
    pub remote_input: RemoteInput,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum RemoteInput {
    GameInput(GameInput),
    ConfigurationInput(ConfigurationInput),
    NoInput,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum GameInput {
    Aim(f64),
    Shoot(f64),
    Bomb,
    Stop,
    Move(f64),
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationInput {
    Ready,
    NotReady,
    SetName(String),
}
