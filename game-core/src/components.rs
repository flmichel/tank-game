use std::f64::consts::PI;

use specs::{Component, VecStorage};
use specs_derive::Component;

use crate::{
    balance::{self, Loadout},
    bot::{Bot, BotKind},
    game::STEPS_PER_SECOND,
    remotes::{ConfigurationInput, Sticks},
};

/// Points sampled around a circle's edge for wall collision. Unrelated to a tank's health.
const CIRCLE_HIT_POINT_COUNT: u32 = 20;

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

    pub fn distance_to(&self, other: &Position) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn direction_to(&self, other: &Position) -> f64 {
        (other.y - self.y).atan2(other.x - self.x)
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

    pub fn new_bullet_movement(direction: f64, blocks_per_second: f64) -> Movement {
        Movement {
            direction,
            speed: blocks_per_second / STEPS_PER_SECOND as f64,
        }
    }

    pub fn set_player_direction(&mut self, direction: f64, blocks_per_second: f64) {
        self.direction = direction;
        self.speed = blocks_per_second / STEPS_PER_SECOND as f64;
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
            radius: balance::PLAYER_RADIUS,
            hit_points: Self::build_hit_points(balance::PLAYER_RADIUS),
        }
    }

    pub fn new_bullet_circle() -> Circle {
        Circle {
            radius: balance::BULLET_RADIUS,
            hit_points: Self::build_hit_points(balance::BULLET_RADIUS),
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

        for i in 0..CIRCLE_HIT_POINT_COUNT {
            let angle = (i as f64 / CIRCLE_HIT_POINT_COUNT as f64) * 2.0 * PI as f64;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            circle_points.push(Position { x, y });
        }

        circle_points
    }
}

/// What drives a player: a phone on the other end of a socket, or the game itself.
///
/// Everything downstream of `BotThink` reads `Player::input` and does not care which this is.
pub enum Controller {
    Human { socket_id: u32 },
    Bot(Bot),
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Player {
    pub controller: Controller,
    pub id: String,
    pub name: String,
    pub status: ReadyStatus,
    pub aim: AimStatus,
    pub shoot: ShootStatus,
    pub input: Input,
    pub is_alive: bool,
    pub is_winner: bool,
    /// Hits left before this tank dies, set from its loadout at spawn.
    pub health: u32,
}

/// What the player's remote asked for: every message is folded in by `RetrievePlayerForInputs`,
/// then `Lobby` or `ApplyInputs` consumes it, according to the phase.
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
    fn new(controller: Controller, id: String) -> Player {
        Player {
            controller,
            id,
            name: String::new(),
            status: ReadyStatus::NotReady,
            aim: AimStatus::None,
            shoot: ShootStatus::CanShoot,
            input: Input::default(),
            is_alive: true,
            is_winner: false,
            health: balance::BASE_HIT_POINTS,
        }
    }

    pub fn new_human(socket_id: u32, id: String) -> Player {
        Player::new(Controller::Human { socket_id }, id)
    }

    pub fn new_bot(id: String, kind: BotKind, seed: u64) -> Player {
        let mut player = Player::new(Controller::Bot(Bot::new(kind, seed)), id);
        player.name = kind.label().to_owned();
        player
    }

    /// `None` for a bot: there is no remote to send to, and `Outbox` is keyed by socket.
    pub fn socket_id(&self) -> Option<u32> {
        match self.controller {
            Controller::Human { socket_id } => Some(socket_id),
            Controller::Bot(_) => None,
        }
    }

    /// A reconnecting remote takes over its player on a new socket. A bot has no remote, so
    /// this cannot happen to one: `SynchronizePlayerToRemote` never offers it a socket.
    pub fn attach_to_socket(&mut self, socket_id: u32) {
        match &mut self.controller {
            Controller::Human { socket_id: current } => *current = socket_id,
            Controller::Bot(_) => {}
        }
    }

    pub fn is_ready(&self) -> bool {
        self.status == ReadyStatus::Ready
    }

    /// Spends one hit point. Returns whether this hit was the fatal one.
    ///
    /// Several bullets can overlap a tank in the same step, and the dead only leave the
    /// board once the step is over, so a hit on an already-dead tank is not a kill.
    pub fn take_hit(&mut self) -> bool {
        if !self.is_alive {
            return false;
        }

        self.health = self.health.saturating_sub(1);
        if self.health == 0 {
            self.is_alive = false;
        }
        !self.is_alive
    }

    pub fn update_after_shot(&mut self, seconds_between_shots: f64) {
        self.aim = AimStatus::None;
        self.shoot = ShootStatus::StepsLeftUntilNextShot(
            (seconds_between_shots * STEPS_PER_SECOND as f64) as u32,
        );
    }
}

#[derive(PartialEq)]
pub enum ReadyStatus {
    Ready,
    NotReady,
}

/// A tank's numbers for one round, derived from what its player has bought.
///
/// Written at spawn so an upgrade bought between rounds takes effect on the next one, and so the
/// systems never read the balance table directly.
#[derive(Component, Clone, Debug, PartialEq)]
#[storage(VecStorage)]
pub struct Stats {
    pub move_blocks_per_second: f64,
    pub bullet_blocks_per_second: f64,
    pub seconds_between_shots: f64,
    pub max_health: u32,
}

impl Stats {
    pub fn from_loadout(loadout: &Loadout) -> Stats {
        Stats {
            move_blocks_per_second: balance::BASE_PLAYER_BLOCKS_PER_SECOND
                + balance::MOVE_SPEED_PER_LEVEL * loadout.move_speed as f64,
            bullet_blocks_per_second: balance::BASE_BULLET_BLOCKS_PER_SECOND
                + balance::BULLET_SPEED_PER_LEVEL * loadout.bullet_speed as f64,
            seconds_between_shots: balance::BASE_SECONDS_BETWEEN_SHOTS
                * balance::RELOAD_FACTOR_PER_LEVEL.powi(loadout.reload as i32),
            max_health: balance::BASE_HIT_POINTS
                + balance::HIT_POINTS_PER_ARMOR_LEVEL * loadout.armor as u32,
        }
    }
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
