use specs::{Entities, Join, System, WriteStorage};

use crate::{bot::BOT_ID_PREFIX, components::Player, game::SynchronizePlayer};

/// A remote announces its player id before anything else: reattach it to its existing player, or
/// create one. Runs in every phase, so a player can reconnect mid-round.
pub struct SynchronizePlayerToRemote;

impl<'a> System<'a> for SynchronizePlayerToRemote {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, SynchronizePlayer>,
        WriteStorage<'a, Player>,
    );

    fn run(&mut self, (entities, synchronize_players, mut players): Self::SystemData) {
        for (entity, synchronize_player) in (&entities, &synchronize_players).join() {
            // A phone picks its own player id. One that looks like a bot would attach the
            // phone to the bot, which has no socket, leaving that phone with no player at
            // all: move it out of the way instead.
            let player_id = match synchronize_player.player_id.strip_prefix(BOT_ID_PREFIX) {
                Some(rest) => format!("remote-{rest}"),
                None => synchronize_player.player_id.clone(),
            };
            if let Some(player) = retrieve_player_from_id(&mut players, &player_id) {
                player.attach_to_socket(synchronize_player.socket_id);
                // A new remote numbers its stick states from scratch
                player.input = Default::default();
            } else {
                let player_entity = entities.create();

                players
                    .insert(
                        player_entity,
                        Player::new_human(synchronize_player.socket_id, player_id),
                    )
                    .unwrap();
            }
            // delete the entity
            entities.delete(entity).expect("Failed to delete entity");
        }
    }
}

fn retrieve_player_from_id<'a>(
    players: &'a mut WriteStorage<Player>,
    player_id: &String,
) -> Option<&'a mut Player> {
    for player in players.join() {
        if &player.id == player_id {
            return Some(player);
        }
    }
    None
}
