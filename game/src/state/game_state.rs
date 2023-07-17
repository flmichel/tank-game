use sdl2::{render::Canvas, video::Window, Sdl};

use crate::room_code::RoomCode;

use super::Map;

pub struct State {
    pub room_code: RoomCode,
    pub phase: Phase,
    pub number_of_ready_players: u32,
    pub map: Map,
    pub resolution: (u32, u32),
}

pub struct Assets {
    pub canvas: Canvas<Window>,
    pub sdl_context: Sdl,
}

pub enum Phase {
    BeforeNextGame,
    InGame,
    BreakInGame,
}
