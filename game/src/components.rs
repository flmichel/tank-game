use sdl2::pixels::Color;
use specs::{Component, VecStorage};
use specs_derive::Component;

use crate::remotes::{PlayerInput, RemoteInput};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Movement {
    direction: f32,
    speed: f32,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Circle {
    radius: i32,
    color: Color,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub status: ReadyStatus,
    pub milliseconds_until_next_shot: i32,
    pub next_input: RemoteInput,
}

impl Player {
    pub fn new(id: u32, name: String) -> Player {
        Player {
            id,
            name,
            status: ReadyStatus::NotReady,
            milliseconds_until_next_shot: 0,
            next_input: RemoteInput::NoInput,
        }
    }
}

pub enum ReadyStatus {
    Ready,
    NotReady,
}
