use specs::{Join, ReadExpect, ReadStorage, System, WriteStorage};
use tracing::trace;

use crate::{
    components::{AimStatus, Movement, Player, ShootStatus, Stats},
    remotes::{GameInput, PlayerInput, RemoteInput},
};

use super::Step;

/// Folds every message received since the last step into its player's `Input`. Phase-agnostic:
/// `Lobby` and `ApplyInputs` decide what the phase allows.
pub struct RetrievePlayerForInputs;

impl<'a> System<'a> for RetrievePlayerForInputs {
    type SystemData = (
        specs::WriteExpect<'a, Vec<PlayerInput>>,
        WriteStorage<'a, Player>,
    );

    fn run(&mut self, (mut player_inputs, mut players): Self::SystemData) {
        for input in player_inputs.drain(..) {
            let player = retrieve_player_from_socket(&mut players, input.socket_id);
            trace!("Player inputs: {:?}", input);

            let Some(player) = player else {
                tracing::warn!("No player found with socket id \"{}\"", input.socket_id);
                continue;
            };
            match input.remote_input {
                // The stick state is unreliable and unordered: drop states older than the applied one
                RemoteInput::Sticks(sticks) if sticks.seq > player.input.sticks.seq => {
                    player.input.sticks = sticks;
                }
                RemoteInput::Sticks(sticks) => trace!("Ignoring stale stick state {}", sticks.seq),
                RemoteInput::GameInput(GameInput::Shoot(direction)) => {
                    player.input.shoot = Some(direction);
                }
                RemoteInput::ConfigurationInput(configuration_input) => {
                    player.input.configuration.push(configuration_input);
                }
            }
        }
    }
}

fn retrieve_player_from_socket<'a>(
    players: &'a mut WriteStorage<Player>,
    socket_id: u32,
) -> Option<&'a mut Player> {
    for player in players.join() {
        if player.socket_id() == Some(socket_id) {
            return Some(player);
        }
    }
    None
}

/// Turns a round's intent into state: sticks become movement and aim, a shoot request arms the
/// player for `Shoot`. Only players on the board have a `Movement`, so the dead are skipped.
pub struct ApplyInputs;

impl<'a> System<'a> for ApplyInputs {
    type SystemData = (
        WriteStorage<'a, Player>,
        WriteStorage<'a, Movement>,
        ReadStorage<'a, Stats>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (mut players, mut movements, stats, step): Self::SystemData) {
        if !step.is_in_game() {
            return;
        }

        for (player, movement, stats) in (&mut players, &mut movements, &stats).join() {
            if !player.input.configuration.is_empty() {
                trace!("configuration input not allowed: game has started");
                player.input.configuration.clear();
            }

            match player.input.sticks.movement {
                Some(direction) => {
                    movement.set_player_direction(direction, stats.move_blocks_per_second)
                }
                None => movement.stop(),
            }
            player.aim = match player.input.sticks.aim {
                Some(direction) => AimStatus::Aim(direction),
                None => AimStatus::None,
            };
            if let Some(direction) = player.input.shoot.take() {
                if player.shoot == ShootStatus::CanShoot {
                    trace!("Player is gonna shoot");
                    player.aim = AimStatus::Aim(direction);
                    player.shoot = ShootStatus::Shooting;
                } else {
                    player.aim = AimStatus::None;
                }
            }
        }
    }
}
