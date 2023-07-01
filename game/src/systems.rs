use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteStorage};

use crate::{
    components::{Player, ReadyStatus},
    game_state::{Phase, State},
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

pub struct HandleInputs;

impl<'a> System<'a> for HandleInputs {
    type SystemData = (WriteStorage<'a, Player>, ReadExpect<'a, State>);

    fn run(&mut self, (players, state): Self::SystemData) {
        match state.phase {
            Phase::BeforeNextGame | Phase::BreakInGame => self.handle_configuration_inputs(players),
            Phase::InGame => {
                print!("not supported yet");
            }
        }
    }
}

impl HandleInputs {
    fn handle_configuration_inputs<'a>(&self, mut players: WriteStorage<'a, Player>) {
        for mut player in (&mut players).join() {
            match &player.next_input {
                RemoteInput::GameInput(_) => println!("not allowded: Game hasn't started yet"),
                RemoteInput::ConfigurationInput(ConfigurationInput::NotReady) => {
                    player.status = ReadyStatus::NotReady;
                }
                RemoteInput::ConfigurationInput(ConfigurationInput::Ready) => {
                    player.status = ReadyStatus::Ready;
                }
                RemoteInput::ConfigurationInput(ConfigurationInput::SetName(name)) => {
                    if !name.is_empty() {
                        player.name = name.to_string();
                    }
                }
                RemoteInput::NoInput => {
                    print!("nothing to do")
                }
            }
        }
    }
}
