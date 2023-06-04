use specs::{Component, VecStorage};
use specs_derive::Component;

use crate::remotes::PlayerInput;

pub enum MessageToGame {
    PlayerInput(PlayerInput),
    RoomId(RoomId),
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct RoomId(pub String);
