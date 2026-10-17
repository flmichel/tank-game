use specs::{Entities, Join, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    components::{Bullet, Circle, Movement, Player, Position},
    game::{MessageToRemote, Outbox},
    state::game_state::{Phase, State},
};

use super::{remove_from_board, Step};

/// Ends the round once the last opponent is gone (none left is a draw), clears the board and
/// hands the game back to the lobby. Solo there is no opponent to lose, so that round runs until
/// the player leaves.
pub struct ResolveRound;

impl<'a> System<'a> for ResolveRound {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Circle>,
        WriteStorage<'a, Movement>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Bullet>,
        WriteExpect<'a, State>,
        WriteExpect<'a, Outbox>,
        WriteExpect<'a, Step>,
    );

    fn run(
        &mut self,
        (entities, mut players, mut circles, mut movements, mut positions, bullets, mut state, mut outbox, mut step): Self::SystemData,
    ) {
        if !step.is_in_game() {
            return;
        }

        let players_needed = players.join().count().min(2);
        if (&players, &positions).join().count() >= players_needed {
            return;
        }

        for (entity, player) in (&entities, &mut players).join() {
            player.is_winner = positions.contains(entity);
            remove_from_board(entity, &mut positions, &mut movements, &mut circles);
            if let Some(socket_id) = player.socket_id() {
                outbox.0.push((socket_id, MessageToRemote::GameOver));
            }
        }
        for (entity, _) in (&entities, &bullets).join() {
            entities.delete(entity).expect("Failed to delete entity");
        }
        state.phase = Phase::BeforeNextGame;
        step.round_ended = true;
    }
}
