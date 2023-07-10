use sdl2::pixels::Color;
use specs::{Component, VecStorage};
use specs_derive::Component;

use crate::remotes::RemoteInput;

const PLAYER_MOVEMENT_SPEED: f64 = 5.;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn new() -> Position {
        Position { x: 0., y: 0. }
    }

    pub fn update(&mut self, movement: &Movement) {
        self.x += movement.speed * movement.direction.cos();
        self.y += movement.speed * movement.direction.sin();
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Movement {
    direction: f64,
    speed: f64,
}

impl Movement {
    pub fn new() -> Movement {
        Movement {
            direction: 0.,
            speed: 0.,
        }
    }

    pub fn set_player_direction(&mut self, direction: f64) {
        self.direction = direction;
        self.speed = PLAYER_MOVEMENT_SPEED;
    }

    pub fn stop(&mut self) {
        self.speed = 0.;
    }
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

    pub fn is_ready(&self) -> bool {
        self.status == ReadyStatus::Ready
    }
}

#[derive(PartialEq)]
pub enum ReadyStatus {
    Ready,
    NotReady,
}
