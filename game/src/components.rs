use specs::{Component, VecStorage};
use specs_derive::Component;

use crate::{remotes::RemoteInput, render::renderer};

const BLOCK_PER_SECOND: f64 = 0.5;
const DEFAULT_PLAYER_RADIUS: f64 = 0.1;

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

    pub fn next(&mut self, movement: &Movement) -> Position {
        let x = self.x + movement.speed * movement.direction.cos();
        let y = self.y + movement.speed * movement.direction.sin();
        Position { x, y }
    }

    pub fn update(&mut self, new_position: &Position) {
        self.x = new_position.x;
        self.y = new_position.y;
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
        self.speed = BLOCK_PER_SECOND / renderer::FRAME_PER_SECOND as f64;
    }

    pub fn stop(&mut self) {
        self.speed = 0.;
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Circle {
    radius: f64,
}

impl Circle {
    pub fn new_player_circle() -> Circle {
        Circle {
            radius: DEFAULT_PLAYER_RADIUS,
        }
    }

    pub fn get_size(&self) -> f64 {
        self.radius * 2.
    }

    pub fn get_radius(&self) -> f64 {
        self.radius
    }
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
