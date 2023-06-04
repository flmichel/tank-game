use specs::{Entities, Join, ReadStorage, System, WriteStorage};

use crate::{
    components::Player,
    remotes::{ConfigurationInput, PlayerInput, RemoteInput},
};

pub struct RetrievePlayerForInputs;

impl<'a> System<'a> for RetrievePlayerForInputs {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, PlayerInput>,
        WriteStorage<'a, Player>,
    );

    fn run(&mut self, (entities, player_inputs, mut players): Self::SystemData) {
        for (entity, input) in (&entities, &player_inputs).join() {
            let player = self.retrieve_player(&mut players, input.player_id);

            // Update the players next input
            if let Some(player) = player {
                // Perform action if `option_value` is `Some`
                player.next_input = input.remote_input.clone();
            } else {
                // If the input is SetName we create a new player if it doesn't exist
                if let RemoteInput::ConfigurationInput(ConfigurationInput::SetName(name)) =
                    &input.remote_input
                {
                    players
                        .insert(
                            entities.create(),
                            Player::new(input.player_id, name.to_string()),
                        )
                        .unwrap();
                    print!("new player created")
                } else {
                    println!("No player found");
                    // TODO Log this case correctly
                }
            }
            // delete the entity
            entities.delete(entity).expect("Failed to delete entity");
        }
    }
}

impl RetrievePlayerForInputs {
    fn retrieve_player<'a>(
        &self,
        players: &'a mut WriteStorage<Player>,
        player_id: u32,
    ) -> Option<&'a mut Player> {
        for player in players.join() {
            if player.id == player_id {
                return Some(player);
            }
        }
        None
    }
}
