use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum MessageToGame {
    Input(Input),
    RoomId(String),
    PlayerDirection(f64),
}

#[derive(Deserialize)]
pub enum Input {
    A,
    B,
}
