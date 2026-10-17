use specs::{Entities, Entity, Join, LendJoin, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};
use tracing::trace;

use crate::{
    components::{AimStatus, Bullet, Circle, Movement, Player, Position, ShootStatus, Stats},
    state::{game_state::State, match_state::Match},
};

use super::{has_bullet_player_collision, has_wall_collision, remove_from_board, Step};

/// Integrates movement for everything on the board. Players stop against walls, bullets die on
/// them.
pub struct MoveAndCollideWalls;

impl<'a> System<'a> for MoveAndCollideWalls {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Circle>,
        WriteStorage<'a, Movement>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Bullet>,
        ReadExpect<'a, State>,
        ReadExpect<'a, Step>,
    );

    fn run(
        &mut self,
        (entities, mut circles, mut movements, mut positions, bullets, state, step): Self::SystemData,
    ) {
        if !step.is_in_game() {
            return;
        }

        for (entity, circle, movement, position, bullet) in (
            &entities,
            &mut circles,
            &mut movements,
            &mut positions,
            (&bullets).maybe(),
        )
            .join()
        {
            let next_position = position.next(movement);
            if !has_wall_collision(&next_position, circle, &state.map) {
                position.update(&next_position);
            } else if bullet.is_some() {
                entities.delete(entity).expect("Failed to delete entity");
            }
        }
    }
}

/// A bullet that reaches a player costs it a hit point, and the dead leave the board: no more
/// moving, shooting, or being hit. A fatal hit is credited to whoever fired it.
pub struct ResolveHits;

impl<'a> System<'a> for ResolveHits {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Circle>,
        WriteStorage<'a, Movement>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Bullet>,
        WriteExpect<'a, Match>,
        ReadExpect<'a, Step>,
    );

    fn run(
        &mut self,
        (entities, mut players, mut circles, mut movements, mut positions, bullets, mut match_state, step): Self::SystemData,
    ) {
        if !step.is_in_game() {
            return;
        }

        for (player_circle, player_position, player) in (&circles, &positions, &mut players).join()
        {
            for (bullet_entity, bullet_circle, bullet_position, bullet) in
                (&entities, &circles, &positions, &bullets).join()
            {
                if has_bullet_player_collision(
                    player_position,
                    player_circle,
                    bullet_position,
                    bullet_circle,
                ) && bullet.owner_id != player.id
                {
                    if player.take_hit() {
                        match_state.kills_this_round.push(bullet.owner_id.clone());
                    }
                    entities
                        .delete(bullet_entity)
                        .expect("Failed to delete entity");
                }
            }
        }

        for (entity, player) in (&entities, &players).join() {
            if !player.is_alive {
                remove_from_board(entity, &mut positions, &mut movements, &mut circles);
            }
        }
    }
}

/// Turns an armed player into a bullet, at the speed its loadout paid for. Skipped on the step a
/// round ends, so a dying player's last shot does not outlive the round.
pub struct Shoot;

impl<'a> System<'a> for Shoot {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Bullet>,
        WriteStorage<'a, Circle>,
        WriteStorage<'a, Movement>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Stats>,
        ReadExpect<'a, Step>,
    );

    fn run(
        &mut self,
        (entities, mut players, mut bullets, mut circles, mut movements, mut positions, stats, step): Self::SystemData,
    ) {
        if !step.is_in_game() || step.round_ended {
            return;
        }

        let mut new_bullets = vec![];
        for (player, position, stats) in (&mut players, &positions, &stats).join() {
            if player.shoot == ShootStatus::Shooting {
                match player.aim {
                    AimStatus::Aim(direction) => new_bullets.push(BulletData {
                        entity: entities.create(),
                        bullet: Bullet::new(player.id.clone()),
                        position: position.clone(),
                        movement: Movement::new_bullet_movement(
                            direction,
                            stats.bullet_blocks_per_second,
                        ),
                        circle: Circle::new_bullet_circle(),
                    }),
                    AimStatus::None => {
                        trace!("player must be aiming when shooting")
                    }
                }
                player.update_after_shot(stats.seconds_between_shots);
            }
        }
        for bullet in new_bullets {
            bullets.insert(bullet.entity, bullet.bullet).unwrap();
            positions.insert(bullet.entity, bullet.position).unwrap();
            movements.insert(bullet.entity, bullet.movement).unwrap();
            circles.insert(bullet.entity, bullet.circle).unwrap();
        }
    }
}

struct BulletData {
    entity: Entity,
    bullet: Bullet,
    position: Position,
    movement: Movement,
    circle: Circle,
}

/// Counts down the steps until each player may shoot again.
pub struct TickShotCooldown;

impl<'a> System<'a> for TickShotCooldown {
    type SystemData = (WriteStorage<'a, Player>, ReadExpect<'a, Step>);

    fn run(&mut self, (mut players, step): Self::SystemData) {
        if !step.is_in_game() || step.round_ended {
            return;
        }

        for player in (&mut players).join() {
            if let ShootStatus::StepsLeftUntilNextShot(steps) = player.shoot {
                if steps > 0 {
                    player.shoot = ShootStatus::StepsLeftUntilNextShot(steps - 1);
                } else {
                    player.shoot = ShootStatus::CanShoot;
                }
            }
        }
    }
}
