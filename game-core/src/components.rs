use std::f32::consts::PI;

use specs::{Component, VecStorage};
use specs_derive::Component;

use crate::{
    game::STEPS_PER_SECOND,
    remotes::{ConfigurationInput, Sticks},
};

const PLAYER_BLOCKS_PER_SECOND: f64 = 0.5;
const BULLET_BLOCKS_PER_SECOND: f64 = 1.;
const DEFAULT_PLAYER_RADIUS: f64 = 0.1;
const DEFAULT_BULLET_RADIUS: f64 = 0.05;
const DEFAULT_HIT_POINTS_NUMBER: u32 = 20;
const SECONDS_BETWEEN_SHOTS: f64 = 0.5;

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
            speed: BULLET_BLOCKS_PER_SECOND / STEPS_PER_SECOND as f64,
        }
    }

    pub fn set_player_direction(&mut self, direction: f64) {
        self.direction = direction;
        self.speed = PLAYER_BLOCKS_PER_SECOND / STEPS_PER_SECOND as f64;
    }

    pub fn stop(&mut self) {
        self.speed = 0.;
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Circle {
    radius: f64,
    pub hit_points: Vec<Position>,
}

impl Circle {
    pub fn new_player_circle() -> Circle {
        Circle {
            radius: DEFAULT_PLAYER_RADIUS,
            hit_points: Self::build_hit_points(DEFAULT_PLAYER_RADIUS),
        }
    }

    pub fn new_bullet_circle() -> Circle {
        Circle {
            radius: DEFAULT_BULLET_RADIUS,
            hit_points: Self::build_hit_points(DEFAULT_BULLET_RADIUS),
        }
    }

    pub fn get_size(&self) -> f64 {
        self.radius * 2.
    }

    pub fn get_radius(&self) -> f64 {
        self.radius
    }

    pub fn build_hit_points(radius: f64) -> Vec<Position> {
        let mut circle_points = Vec::new();
        let center_x = 0.0; // Assuming the circle is centered at the origin (0, 0)
        let center_y = 0.0;

        for i in 0..DEFAULT_HIT_POINTS_NUMBER {
            let angle = (i as f64 / DEFAULT_HIT_POINTS_NUMBER as f64) * 2.0 * PI as f64;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            circle_points.push(Position { x, y });
        }

        circle_points
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Player {
    pub socket_id: u32,
    pub id: String,
    pub name: String,
    pub status: ReadyStatus,
    pub aim: AimStatus,
    pub shoot: ShootStatus,
    pub input: Input,
    pub is_alive: bool,
    pub is_winner: bool,
}

/// What the player's remote asked for: every message is folded in by `RetrievePlayerForInputs`,
/// then `HandleInputs` consumes it according to the phase.
#[derive(Default)]
pub struct Input {
    pub sticks: Sticks,
    pub shoot: Option<f64>,
    pub configuration: Vec<ConfigurationInput>,
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
    StepsLeftUntilNextShot(u32),
}

impl Player {
    pub fn new(socket_id: u32, id: String) -> Player {
        Player {
            socket_id,
            id,
            name: String::new(),
            status: ReadyStatus::NotReady,
            aim: AimStatus::None,
            shoot: ShootStatus::CanShoot,
            input: Input::default(),
            is_alive: true,
            is_winner: false,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.status == ReadyStatus::Ready
    }

    pub fn update_after_shot(&mut self) {
        self.aim = AimStatus::None;
        self.shoot = ShootStatus::StepsLeftUntilNextShot(
            (SECONDS_BETWEEN_SHOTS * STEPS_PER_SECOND as f64) as u32,
        );
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
    pub owner_id: String,
}

impl Bullet {
    pub fn new(owner_id: String) -> Bullet {
        Bullet { owner_id }
    }
}