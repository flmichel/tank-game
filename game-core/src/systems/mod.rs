//! The step pipeline. Each system owns one decision and they run in the order declared by
//! `Game::new`'s dispatcher:
//!
//! ```text
//! SynchronizePlayerToRemote -> RetrievePlayerForInputs -> BotThink -> ApplyPurchases -> Lobby
//!   -> ApplyInputs -> MoveAndCollideWalls -> ResolveHits -> ResolveRound -> AwardMoney
//!   -> Shoot -> TickShotCooldown
//! ```

mod bot;
mod combat;
mod economy;
mod inputs;
mod lobby;
mod round;
mod sync;

pub use bot::BotThink;
pub use combat::{MoveAndCollideWalls, ResolveHits, Shoot, TickShotCooldown};
pub use economy::{ApplyPurchases, AwardMoney};
pub use inputs::{ApplyInputs, RetrievePlayerForInputs};
pub use lobby::Lobby;
pub use round::ResolveRound;
pub use sync::SynchronizePlayerToRemote;

use specs::{Entity, WriteStorage};

use crate::{
    components::{Circle, Movement, Position},
    state::{game_state::Phase, Block, Map},
};

/// Per-step execution context, rewritten by `Game::step` before every dispatch.
///
/// `phase` is a snapshot taken before any system runs, so a system that changes
/// `State::phase` (`Lobby` starting a round, `ResolveRound` ending one) only takes effect on the
/// next step. Every phase-gated system in one step therefore agrees on which phase it is in.
pub struct Step {
    pub phase: Phase,
    /// Set by `ResolveRound`: the systems after it do not run on the step a round ends.
    pub round_ended: bool,
}

impl Step {
    pub fn new(phase: Phase) -> Step {
        Step { phase, round_ended: false }
    }

    pub fn is_in_game(&self) -> bool {
        self.phase == Phase::InGame
    }

    pub fn is_configuring(&self) -> bool {
        matches!(self.phase, Phase::BeforeNextGame | Phase::BreakInGame)
    }
}

pub(crate) fn get_position_block_center(block: Block) -> Position {
    Position {
        x: 0.5 + block.0 as f64,
        y: 0.5 + block.1 as f64,
    }
}

pub(crate) fn has_wall_collision(position: &Position, circle: &Circle, map: &Map) -> bool {
    for hit_point in circle.hit_points.iter() {
        if map.is_wall(position.x + hit_point.x, position.y + hit_point.y) {
            return true;
        }
    }

    false
}

pub(crate) fn has_bullet_player_collision(
    player_position: &Position,
    player_circle: &Circle,
    bullet_position: &Position,
    bullet_circle: &Circle,
) -> bool {
    let distance = ((player_position.x - bullet_position.x).powi(2)
        + (player_position.y - bullet_position.y).powi(2))
    .sqrt();
    distance < player_circle.get_radius() + bullet_circle.get_radius()
}

pub(crate) fn remove_from_board(
    entity: Entity,
    positions: &mut WriteStorage<Position>,
    movements: &mut WriteStorage<Movement>,
    circles: &mut WriteStorage<Circle>,
) {
    positions.remove(entity);
    movements.remove(entity);
    circles.remove(entity);
}

/// Blocks between samples when tracing a line of sight. Finer than the narrowest wall, so a
/// bot cannot shoot through a corner.
const LINE_OF_SIGHT_STEP: f64 = 0.1;

/// Whether nothing solid stands between the two points.
pub(crate) fn has_line_of_sight(from: &Position, to: &Position, map: &Map) -> bool {
    let distance = from.distance_to(to);
    let direction = from.direction_to(to);
    let samples = (distance / LINE_OF_SIGHT_STEP).ceil() as u32;

    for sample in 1..samples {
        let travelled = sample as f64 * LINE_OF_SIGHT_STEP;
        if map.is_wall(
            from.x + direction.cos() * travelled,
            from.y + direction.sin() * travelled,
        ) {
            return false;
        }
    }

    true
}
