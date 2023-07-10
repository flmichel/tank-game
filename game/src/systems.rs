use specs::{Entities, Join, ReadExpect, ReadStorage, System, Write, WriteExpect, WriteStorage};

use crate::{
    components::{Movement, Player, Position, ReadyStatus},
    game_state::{Phase, State},
    remotes::{ConfigurationInput, GameInput, PlayerInput, RemoteInput},
};

pub struct RetrievePlayerForInputs;

impl<'a> System<'a> for RetrievePlayerForInputs {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, PlayerInput>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Movement>,
        WriteStorage<'a, Position>,
    );

    fn run(
        &mut self,
        (entities, player_inputs, mut players, mut movements, mut positions): Self::SystemData,
    ) {
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
                    let player_entity = entities.create();
                    players
                        .insert(
                            player_entity,
                            Player::new(input.player_id, name.to_string()),
                        )
                        .unwrap();

                    positions.insert(player_entity, Position::new()).unwrap();

                    movements.insert(player_entity, Movement::new()).unwrap();
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
    type SystemData = (
        WriteStorage<'a, Player>,
        WriteStorage<'a, Movement>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, State>,
    );

    fn run(&mut self, (players, movements, positions, state): Self::SystemData) {
        match state.phase {
            Phase::BeforeNextGame | Phase::BreakInGame => {
                self.handle_configuration_inputs(players, state)
            }
            Phase::InGame => {
                let (players, movements) = self.handle_game_inputs(players, movements);
                self.update_game(players, movements, positions, state);
            }
        }
    }
}

impl HandleInputs {
    fn handle_configuration_inputs<'a>(
        &self,
        mut players: WriteStorage<'a, Player>,
        mut state: WriteExpect<'a, State>,
    ) {
        for mut player in (&mut players).join() {
            match &player.next_input {
                RemoteInput::GameInput(_) => {
                    println!("game input is not allowded: Game hasn't started yet")
                }
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

        if self.all_players_are_ready(&players) {
            state.phase = Phase::InGame;
        }
    }

    fn all_players_are_ready(&self, players: &WriteStorage<Player>) -> bool {
        !&players.is_empty() && players.join().all(|player| player.is_ready())
    }

    fn handle_game_inputs<'a>(
        &self,
        mut players: WriteStorage<'a, Player>,
        mut movements: WriteStorage<'a, Movement>,
    ) -> (WriteStorage<'a, Player>, WriteStorage<'a, Movement>) {
        for (player, movement) in (&mut players, &mut movements).join() {
            match &player.next_input {
                RemoteInput::GameInput(GameInput::Move(direction)) => {
                    movement.set_player_direction(*direction);
                }
                RemoteInput::GameInput(GameInput::Stop) => movement.stop(),
                RemoteInput::GameInput(_) => print!("not implemented yet"),
                RemoteInput::ConfigurationInput(_) => {
                    println!("configuration input not allowed: game has started")
                }
                RemoteInput::NoInput => {}
            }
        }

        (players, movements)
    }

    fn update_game<'a>(
        &self,
        mut players: WriteStorage<'a, Player>,
        mut movements: WriteStorage<'a, Movement>,
        mut positions: WriteStorage<'a, Position>,
        mut state: WriteExpect<'a, State>,
    ) {
        for (movement, position) in (&mut movements, &mut positions).join() {
            position.update(movement);
        }
    }
}
