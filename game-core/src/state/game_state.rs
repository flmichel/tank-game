use crate::room_code::RoomCode;

use super::Map;

pub struct State {
    pub room_code: RoomCode,
    pub phase: Phase,
    pub number_of_ready_players: u32,
    pub map: Map,
    pub resolution: (u32, u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    BeforeNextGame,
    InGame,
    BreakInGame,
}
