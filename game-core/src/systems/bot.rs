use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteStorage};

use crate::{
    bot::{BotKind, HUNTER_PREFERRED_DISTANCE, WANDER_STEPS},
    components::{Controller, Player, Position, ShootStatus},
    remotes::ConfigurationInput,
    state::game_state::State,
};

use super::{has_line_of_sight, Step};

/// Gives every bot its intent for this step, writing the same `Player::input` a phone writes.
///
/// Runs after `RetrievePlayerForInputs` so that all intent, human and bot, is gathered before any
/// phase consumes it.
pub struct BotThink;

impl<'a> System<'a> for BotThink {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Player>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, State>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (entities, mut players, positions, state, step): Self::SystemData) {
        if step.is_configuring() {
            for player in (&mut players).join() {
                if matches!(player.controller, Controller::Bot(_)) && !player.is_ready() {
                    // A bot readies up the way a phone does, so `Lobby` remains the only place
                    // that decides when a round starts
                    player.input.configuration.push(ConfigurationInput::Ready);
                }
            }
            return;
        }
        if !step.is_in_game() {
            return;
        }

        // Taken before any bot acts, so they all decide from the same board
        let board: Vec<(String, Position)> = (&players, &positions)
            .join()
            .filter(|(player, _)| player.is_alive)
            .map(|(player, position)| (player.id.clone(), position.clone()))
            .collect();

        for (entity, player) in (&entities, &mut players).join() {
            if !player.is_alive {
                continue;
            }
            let Controller::Bot(bot) = &mut player.controller else {
                continue;
            };
            let Some(own_position) = positions.get(entity) else {
                continue;
            };

            match bot.kind {
                // Target practice: it spawns and waits to be shot
                BotKind::Dummy => {}
                BotKind::Wanderer => {
                    if bot.steps_until_turn == 0 {
                        player.input.sticks.movement = Some(bot.rng.next_angle());
                        bot.steps_until_turn = WANDER_STEPS;
                    } else {
                        bot.steps_until_turn -= 1;
                    }
                    if player.shoot == ShootStatus::CanShoot {
                        player.input.shoot = Some(bot.rng.next_angle());
                    }
                }
                BotKind::Hunter => {
                    let target = board
                        .iter()
                        .filter(|(id, _)| id != &player.id)
                        .min_by(|(_, a), (_, b)| {
                            own_position
                                .distance_to(a)
                                .total_cmp(&own_position.distance_to(b))
                        });

                    let Some((_, target_position)) = target else {
                        // Alone on the board: hold still rather than drive into a wall
                        player.input.sticks.movement = None;
                        player.input.sticks.aim = None;
                        continue;
                    };

                    let direction = own_position.direction_to(target_position);
                    let distance = own_position.distance_to(target_position);

                    player.input.sticks.aim = Some(direction);
                    // Close the distance, then stop: shooting from too close leaves no room to dodge
                    player.input.sticks.movement = (distance > HUNTER_PREFERRED_DISTANCE).then_some(direction);

                    if player.shoot == ShootStatus::CanShoot
                        && has_line_of_sight(own_position, target_position, &state.map)
                    {
                        player.input.shoot = Some(direction);
                    }
                }
            }
        }
    }
}
