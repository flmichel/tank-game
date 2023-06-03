use serde::Deserialize;

pub struct PlayerInput {
    pub player_id: u32,
    pub remote_input: RemoteInput,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum RemoteInput {
    GameInput(GameInput),
    ReadyToPlay,
    NewPlayer(String),
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameInput {
    Aim(f64),
    Shoot(f64),
    Bomb,
    Stop,
    Move(f64),
}
