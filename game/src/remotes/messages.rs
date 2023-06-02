use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]

pub enum RemoteInput {
    GameInput(GameInput),
    ReadyToPlay,
    SetUsername(String),
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
