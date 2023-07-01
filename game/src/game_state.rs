use sdl2::{
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    Sdl,
};

use crate::room_code::RoomCode;

pub struct State {
    pub room_code: RoomCode,
    pub phase: Phase,
    pub number_of_ready_players: u32,
}

pub struct Assets<'a> {
    pub canvas: Canvas<Window>,
    pub sdl_context: Sdl,
    pub player_face: Texture<'a>,
}

pub enum Phase {
    BeforeNextGame,
    InGame,
    BreakInGame,
}
