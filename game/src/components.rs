use specs::{Component, VecStorage};
use specs_derive::Component;

use crate::{remotes::RemoteInput, render::renderer};

const PLAYER_BLOCKS_PER_SECOND: f64 = 0.5;
const BULLET_BLOCKS_PER_SECOND: f64 = 1.;
const DEFAULT_PLAYER_RADIUS: f64 = 0.1;
const DEFAULT_BULLET_RADIUS: f64 = 0.05;
const NUMBER_OF_FRAMES_BETWEEN_SHOTS: u32 = 15;

#[derive(Component, Clone)]
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

    pub fn new_bullet_movement(direction: f64) -> Movement {
        Movement {
            direction,
            speed: BULLET_BLOCKS_PER_SECOND / renderer::FRAME_PER_SECOND as f64,
        }
    }

    pub fn set_player_direction(&mut self, direction: f64) {
        self.direction = direction;
        self.speed = PLAYER_BLOCKS_PER_SECOND / renderer::FRAME_PER_SECOND as f64;
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

    pub fn new_bullet_circle() -> Circle {
        Circle {
            radius: DEFAULT_BULLET_RADIUS,
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
    pub aim: AimStatus,
    pub shoot: ShootStatus,
    pub next_input: RemoteInput,
}

#[derive(PartialEq)]
pub enum AimStatus {
    Aim(f64),
    None,
}

#[derive(PartialEq)]
pub enum ShootStatus {
    CanShoot,
    Shooting,
    FrameLeftUntilNextShot(u32),
}

impl Player {
    pub fn new(id: u32, name: String) -> Player {
        Player {
            id,
            name,
            status: ReadyStatus::NotReady,
            aim: AimStatus::None,
            shoot: ShootStatus::CanShoot,
            next_input: RemoteInput::NoInput,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.status == ReadyStatus::Ready
    }

    pub fn update_after_shot(&mut self) {
        self.aim = AimStatus::None;
        self.shoot = ShootStatus::FrameLeftUntilNextShot(NUMBER_OF_FRAMES_BETWEEN_SHOTS);
    }
}

#[derive(PartialEq)]
pub enum ReadyStatus {
    Ready,
    NotReady,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Bullet {
    owner_id: u32,
}

impl Bullet {
    pub fn new(owner_id: u32) -> Bullet {
        Bullet { owner_id }
    }
}
