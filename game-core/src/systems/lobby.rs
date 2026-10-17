use specs::{Entities, Join, ReadExpect, System, WriteExpect, WriteStorage};
use tracing::{debug, error};

use crate::{
    components::{AimStatus, Circle, Movement, Player, Position, ReadyStatus, ShootStatus, Stats},
    game::{MessageToRemote, Outbox},
    remotes::ConfigurationInput,
    state::{
        game_state::{Phase, State},
        match_state::Match,
    },
};

use super::{get_position_block_center, Step};

/// Between rounds: applies configuration input (ready, name) and starts the next round once
/// everyone is ready. Game input is dropped here rather than queued, so nothing a player pressed
/// in the lobby leaks into the round.
pub struct Lobby;

impl<'a> System<'a> for Lobby {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Movement>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Circle>,
        WriteStorage<'a, Stats>,
        WriteExpect<'a, State>,
        WriteExpect<'a, Outbox>,
        ReadExpect<'a, Match>,
        ReadExpect<'a, Step>,
    );

    fn run(
        &mut self,
        (entities, mut players, mut movements, mut positions, mut circles, mut stats, mut state, mut outbox, match_state, step): Self::SystemData,
    ) {
        if !step.is_configuring() {
            return;
        }

        for player in (&mut players).join() {
            // Game inputs are not allowed before the game starts: drop them, so none is left over
            let input = std::mem::take(&mut player.input);
            player.input.sticks.seq = input.sticks.seq;
            if input.shoot.is_some() {
                debug!("Game input is not allowed: Game hasn't started yet.")
            }

            for configuration_input in input.configuration {
                match configuration_input {
                    ConfigurationInput::NotReady => player.status = ReadyStatus::NotReady,
                    ConfigurationInput::Ready => player.status = ReadyStatus::Ready,
                    ConfigurationInput::SetName(name) => {
                        if !name.is_empty() {
                            player.name = name;
                        }
                    }
                    ConfigurationInput::PlayerId(_) => {}
                    // Already spent by `ApplyPurchases`, which runs before this system
                    ConfigurationInput::Buy(_) => {}
                }
            }
        }

        if !can_game_start(&players) {
            return;
        }

        state.phase = Phase::InGame;
        state.map.reset_spawns();

        for (player_entity, player) in (&entities, &mut players).join() {
            player.status = ReadyStatus::NotReady;
            player.is_alive = true;
            player.is_winner = false;
            player.aim = AimStatus::None;
            player.shoot = ShootStatus::CanShoot;
            if let Some(socket_id) = player.socket_id() {
                outbox.0.push((socket_id, MessageToRemote::GameStart));
            }

            // What the player bought between rounds takes effect now, for this round
            let round_stats = Stats::from_loadout(&match_state.loadout(&player.id));
            player.health = round_stats.max_health;

            let spawn = state.map.get_spawn_block();
            match spawn {
                Err(err) => error!("Couldn't spawn player: {}.", err),
                Ok(spawn) => {
                    let spawn_position = get_position_block_center(spawn);

                    positions.insert(player_entity, spawn_position).unwrap();
                    movements.insert(player_entity, Movement::new()).unwrap();
                    circles
                        .insert(player_entity, Circle::new_player_circle())
                        .unwrap();
                    stats.insert(player_entity, round_stats).unwrap();
                }
            }
        }
    }
}

// One player is enough: a solo round has no opponent, so it is for trying the controls out
fn can_game_start(players: &WriteStorage<Player>) -> bool {
    all_players_are_ready(players)
}

fn all_players_are_ready(players: &WriteStorage<Player>) -> bool {
    !&players.is_empty() && players.join().all(|player| player.is_ready())
}
