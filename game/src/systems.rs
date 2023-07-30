use specs::{Entities, Entity, Join, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    components::{AimStatus, Bullet, Circle, Movement, Player, Position, ReadyStatus, ShootStatus},
    remotes::{ConfigurationInput, GameInput, PlayerInput, RemoteInput},
    state::{
        game_state::{Phase, State},
        Block, Map,
    },
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
            println!("player inputs: {:?}", input);

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
        Entities<'a>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Bullet>,
        WriteStorage<'a, Movement>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Circle>,
        WriteExpect<'a, State>,
    );

    fn run(
        &mut self,
        (entities, players, bullet, movements, positions, circles, state): Self::SystemData,
    ) {
        match state.phase {
            Phase::BeforeNextGame | Phase::BreakInGame => self.handle_configuration_inputs(
                entities, players, movements, positions, circles, state,
            ),
            Phase::InGame => {
                let (players, movements) = self.handle_game_inputs(players, movements);
                self.update_game(
                    entities, players, bullet, circles, movements, positions, state,
                );
            }
        }
    }
}

impl HandleInputs {
    fn handle_configuration_inputs<'a>(
        &self,
        entities: Entities<'a>,
        mut players: WriteStorage<'a, Player>,
        mut movements: WriteStorage<'a, Movement>,
        mut positions: WriteStorage<'a, Position>,
        mut circles: WriteStorage<'a, Circle>,
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

            for (player_entity, _) in (&entities, &players).join() {
                let spawn = state.map.get_spawn_block();
                match spawn {
                    Err(err) => print!("Couldn't spawn player: {}", err),
                    Ok(spawn) => {
                        let spawn_position = get_position_block_center(spawn);

                        positions.insert(player_entity, spawn_position).unwrap();
                        movements.insert(player_entity, Movement::new()).unwrap();
                        circles
                            .insert(player_entity, Circle::new_player_circle())
                            .unwrap();
                    }
                }
            }
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
            match player.next_input {
                RemoteInput::GameInput(GameInput::Move(direction)) => {
                    movement.set_player_direction(direction);
                }
                RemoteInput::GameInput(GameInput::Stop) => movement.stop(),
                RemoteInput::GameInput(GameInput::Aim(direction)) => {
                    player.aim = AimStatus::Aim(direction);
                }
                RemoteInput::GameInput(GameInput::Shoot) => {
                    if player.shoot == ShootStatus::CanShoot {
                        println!("Player is gonna shoot");
                        player.shoot = ShootStatus::Shooting;
                    } else {
                        player.aim = AimStatus::None;
                    }
                }
                RemoteInput::ConfigurationInput(_) => {
                    println!("configuration input not allowed: game has started")
                }
                RemoteInput::NoInput => {}
            }

            player.next_input = RemoteInput::NoInput;
        }

        (players, movements)
    }

    fn update_game<'a>(
        &self,
        entities: Entities<'a>,
        mut players: WriteStorage<Player>,
        mut bullets: WriteStorage<'a, Bullet>,
        mut circles: WriteStorage<'a, Circle>,
        mut movements: WriteStorage<'a, Movement>,
        mut positions: WriteStorage<'a, Position>,
        state: WriteExpect<'a, State>,
    ) {
        // Wall detection
        for (circle, movement, position) in (&mut circles, &mut movements, &mut positions).join() {
            let next_position = position.next(movement);
            if !has_wall_collision(&next_position, circle, &state.map) {
                position.update(&next_position);
            }
        }

        // Generate new bullets
        let mut new_bullets = vec![];
        for (player, position) in (&mut players, &positions).join() {
            if player.shoot == ShootStatus::Shooting {
                match player.aim {
                    AimStatus::Aim(direction) => new_bullets.push(BulletData {
                        entity: entities.create(),
                        bullet: Bullet::new(player.id),
                        position: position.clone(),
                        movement: Movement::new_bullet_movement(direction),
                        circle: Circle::new_bullet_circle(),
                    }),
                    AimStatus::None => {
                        println!("player must be aiming when shooting")
                    }
                }
                player.update_after_shot();
            }
        }
        for bullet in new_bullets {
            bullets.insert(bullet.entity, bullet.bullet).unwrap();
            positions.insert(bullet.entity, bullet.position).unwrap();
            movements.insert(bullet.entity, bullet.movement).unwrap();
            circles.insert(bullet.entity, bullet.circle).unwrap();
        }

        // Update players
        for player in (&mut players).join() {
            if let ShootStatus::FrameLeftUntilNextShot(number_of_frames) = player.shoot {
                if number_of_frames > 0 {
                    player.shoot = ShootStatus::FrameLeftUntilNextShot(number_of_frames - 1);
                } else {
                    player.shoot = ShootStatus::CanShoot;
                }
            }
        }
    }
}

fn get_position_block_center(block: Block) -> Position {
    Position {
        x: 0.5 + block.0 as f64,
        y: 0.5 + block.1 as f64,
    }
}

fn has_wall_collision(position: &Position, circle: &Circle, map: &Map) -> bool {
    let top_block = &Block(position.x as u8, (position.y - circle.get_radius()) as u8);
    let bottom_block = &Block(position.x as u8, (position.y + circle.get_radius()) as u8);
    let right_block = &Block((position.x + circle.get_radius()) as u8, position.y as u8);
    let left_block = &Block((position.x - circle.get_radius()) as u8, position.y as u8);

    map.is_wall(left_block)
        || map.is_wall(right_block)
        || map.is_wall(top_block)
        || map.is_wall(bottom_block)
}

struct BulletData {
    entity: Entity,
    bullet: Bullet,
    position: Position,
    movement: Movement,
    circle: Circle,
}
