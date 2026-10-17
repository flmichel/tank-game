use specs::{Entities, Entity, Join, LendJoin, System, WriteExpect, WriteStorage};
use tracing::{debug, error, trace, warn};

use crate::{
    components::{AimStatus, Bullet, Circle, Movement, Player, Position, ReadyStatus, ShootStatus}, game::{MessageToRemote, Outbox, SynchronizePlayer}, remotes::{ConfigurationInput, GameInput, PlayerInput, RemoteInput}, state::{
        game_state::{Phase, State},
        Block, Map,
    }
};

pub struct RetrievePlayerForInputs;

impl<'a> System<'a> for RetrievePlayerForInputs {
    type SystemData = (WriteExpect<'a, Vec<PlayerInput>>, WriteStorage<'a, Player>);

    fn run(&mut self, (mut player_inputs, mut players): Self::SystemData) {
        for input in player_inputs.drain(..) {
            let player = self.retrieve_player_from_socket(&mut players, input.socket_id);
            trace!("Player inputs: {:?}", input);

            let Some(player) = player else {
                warn!("No player found with socket id \"{}\"", input.socket_id);
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

impl RetrievePlayerForInputs {
    fn retrieve_player_from_socket<'a>(
        &self,
        players: &'a mut WriteStorage<Player>,
        socket_id: u32,
    ) -> Option<&'a mut Player> {
        for player in players.join() {
            if player.socket_id == socket_id {
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
        WriteExpect<'a, Outbox>,
    );

    fn run(
        &mut self,
        (entities, players, bullet, movements, positions, circles, state, outbox): Self::SystemData,
    ) {
        match state.phase {
            Phase::BeforeNextGame | Phase::BreakInGame => self.handle_configuration_inputs(
                entities, players, movements, positions, circles, state, outbox,
            ),
            Phase::InGame => {
                let (players, movements) = self.handle_game_inputs(players, movements);
                self.update_game(
                    entities, players, bullet, circles, movements, positions, state, outbox,
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
        mut outbox: WriteExpect<'a, Outbox>,
    ) {
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
                }
            }
        }

        if self.can_game_start(&players) {
            state.phase = Phase::InGame;
            state.map.reset_spawns();

            for (player_entity, player) in (&entities, &mut players).join() {
                player.status = ReadyStatus::NotReady;
                player.is_alive = true;
                player.is_winner = false;
                player.aim = AimStatus::None;
                player.shoot = ShootStatus::CanShoot;
                outbox.0.push((player.socket_id, MessageToRemote::GameStart));
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
                    }
                }
            }
        }
    }

    // One player is enough: a solo round has no opponent, so it is for trying the controls out
    fn can_game_start(&self, players: &WriteStorage<Player>) -> bool {
        self.all_players_are_ready(players)
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
            if !player.input.configuration.is_empty() {
                trace!("configuration input not allowed: game has started");
                player.input.configuration.clear();
            }

            match player.input.sticks.movement {
                Some(direction) => movement.set_player_direction(direction),
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
        mut state: WriteExpect<'a, State>,
        mut outbox: WriteExpect<'a, Outbox>,
    ) {
        // Wall detection: players stop against walls, bullets are destroyed
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

        // Bullet - Player collision: the player dies, the bullet is destroyed
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
                    player.is_alive = false;
                    entities.delete(bullet_entity).expect("Failed to delete entity");
                }
            }
        }

        // Dead players leave the board: they can no longer move, shoot or be hit
        for (entity, player) in (&entities, &players).join() {
            if !player.is_alive {
                remove_from_board(entity, &mut positions, &mut movements, &mut circles);
            }
        }

        // End the round once the last opponent is gone (none left is a draw). Solo there is no
        // opponent to lose, so that round runs until the player leaves.
        let players_needed = players.join().count().min(2);
        if (&players, &positions).join().count() < players_needed {
            for (entity, player) in (&entities, &mut players).join() {
                player.is_winner = positions.contains(entity);
                remove_from_board(entity, &mut positions, &mut movements, &mut circles);
                outbox.0.push((player.socket_id, MessageToRemote::GameOver));
            }
            for (entity, _) in (&entities, &bullets).join() {
                entities.delete(entity).expect("Failed to delete entity");
            }
            state.phase = Phase::BeforeNextGame;
            return;
        }

        // Generate new bullets
        let mut new_bullets = vec![];
        for (player, position) in (&mut players, &positions).join() {
            if player.shoot == ShootStatus::Shooting {
                match player.aim {
                    AimStatus::Aim(direction) => new_bullets.push(BulletData {
                        entity: entities.create(),
                        bullet: Bullet::new(player.id.clone()),
                        position: position.clone(),
                        movement: Movement::new_bullet_movement(direction),
                        circle: Circle::new_bullet_circle(),
                    }),
                    AimStatus::None => {
                        trace!("player must be aiming when shooting")
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

fn get_position_block_center(block: Block) -> Position {
    Position {
        x: 0.5 + block.0 as f64,
        y: 0.5 + block.1 as f64,
    }
}

fn has_wall_collision(position: &Position, circle: &Circle, map: &Map) -> bool {
    for hit_point in circle.hit_points.iter() {
        if map.is_wall(position.x + hit_point.x, position.y + hit_point.y) {
            return true;
        }
    }

    false
}

fn remove_from_board(
    entity: Entity,
    positions: &mut WriteStorage<Position>,
    movements: &mut WriteStorage<Movement>,
    circles: &mut WriteStorage<Circle>,
) {
    positions.remove(entity);
    movements.remove(entity);
    circles.remove(entity);
}

fn has_bullet_player_collision(
    player_position: &Position,
    player_circle: &Circle,
    bullet_position: &Position,
    bullet_circle: &Circle,
) -> bool {
    let distance = ((player_position.x - bullet_position.x).powi(2)
        + (player_position.y - bullet_position.y).powi(2))
    .sqrt();
    distance < player_circle.get_radius() + bullet_circle.get_radius()
}

struct BulletData {
    entity: Entity,
    bullet: Bullet,
    position: Position,
    movement: Movement,
    circle: Circle,
}

pub struct SynchronizePlayerToRemote;

impl<'a> System<'a> for SynchronizePlayerToRemote {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, SynchronizePlayer>,
        WriteStorage<'a, Player>,
    );

    fn run(
        &mut self,
        (entities, synchronize_players, mut players): Self::SystemData,
    ) {
        for (entity, synchronize_player) in (&entities, &synchronize_players).join() {
            if let Some(player) = retrieve_player_from_id(&mut players, &synchronize_player.player_id) {
                player.socket_id = synchronize_player.socket_id;
                // A new remote numbers its stick states from scratch
                player.input = Default::default();
            } else {
                let player_entity = entities.create();

                players
                    .insert(player_entity, Player::new(
                        synchronize_player.socket_id,
                        synchronize_player.player_id.clone()))
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
