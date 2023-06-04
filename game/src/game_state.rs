use crate::room_code::RoomCode;

pub struct State {
    pub room_code: RoomCode,
    pub phase: Phase,
}

pub enum Phase {
    BeforeNextGame,
    InGame,
    BreakInGame,
}
