use crate::remotes::PlayerInput;

pub enum MessageToGame {
    PlayerInput(PlayerInput),
    RoomId(String),
}
