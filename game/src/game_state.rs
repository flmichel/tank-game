use crate::room_code::RoomCode;

pub struct State {
    pub room_code: RoomCode,
    pub phase: Phase,
    pub number_of_ready_players: u32,
}

pub enum Phase {
    BeforeNextGame,
    InGame,
    BreakInGame,
}
