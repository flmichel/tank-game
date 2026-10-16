use std::collections::VecDeque;

use specs::{Builder, Component, Dispatcher, DispatcherBuilder, VecStorage, World, WorldExt};
use specs_derive::Component;
use tracing::debug;

use crate::{
    components::{Bullet, Circle, Movement, Player, Position},
    remotes::{ConfigurationInput, PlayerInput, RemoteInput},
    render::{self, Renderer},
    room_code::RoomCode,
    state::{
        game_state::{Phase, State},
        Map,
    },
    systems::{HandleInputs, RetrievePlayerForInputs, SynchronizePlayerToRemote},
};

pub const FRAME_PER_SECOND: u32 = 30;
const STEP: f64 = 1. / FRAME_PER_SECOND as f64;
// After a long pause (e.g. a hidden browser tab) drop the backlog instead of fast-forwarding.
const MAX_LAG: f64 = 0.25;

/// Inbound port: everything the outside world tells the game.
pub enum MessageToGame {
    PlayerInput(PlayerInput),
    RoomId(RoomId),
    SynchronizePlayer(SynchronizePlayer),
}

impl MessageToGame {
    /// A remote first sends its player id (to keep its player across reconnects), then inputs.
    pub fn from_remote(socket_id: u32, remote_input: RemoteInput) -> MessageToGame {
        match remote_input {
            RemoteInput::ConfigurationInput(ConfigurationInput::PlayerId(player_id)) => {
                MessageToGame::SynchronizePlayer(SynchronizePlayer { socket_id, player_id })
            }
            remote_input => MessageToGame::PlayerInput(PlayerInput { socket_id, remote_input }),
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct SynchronizePlayer {
    pub socket_id: u32,
    pub player_id: String,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct RoomId(pub String);

#[derive(serde::Serialize)]
pub enum MessageToRemote {
    GameOver,
    GameStart,
    PlayerDied,
}

/// Outbound port: messages for remotes, keyed by socket id. Systems push, the shell delivers.
#[derive(Default)]
pub struct Outbox(pub Vec<(u32, MessageToRemote)>);

pub struct Game {
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
    inbox: VecDeque<MessageToGame>,
    lag: f64,
}

impl Game {
    pub fn new(map: &str, resolution: (u32, u32)) -> Self {
        let mut world = World::new();
        world.register::<RoomId>();
        world.register::<SynchronizePlayer>();
        world.register::<Position>();
        world.register::<Movement>();
        world.register::<Circle>();
        world.register::<Player>();
        world.register::<Bullet>();

        world.insert(State {
            room_code: RoomCode::new("Error, the game could not connect to server".to_owned()),
            phase: Phase::BeforeNextGame,
            number_of_ready_players: 0,
            map: Map::parse(map, resolution),
            resolution,
        });
        world.insert(Outbox::default());
        world.insert(Vec::<PlayerInput>::new());

        // A remote's player must exist before its inputs are applied
        let dispatcher = DispatcherBuilder::new()
            .with(SynchronizePlayerToRemote, "SynchronizePlayerToRemote", &[])
            .with(
                RetrievePlayerForInputs,
                "RetrievePlayerForInputs",
                &["SynchronizePlayerToRemote"],
            )
            .with(HandleInputs, "HandleInputs", &["RetrievePlayerForInputs"])
            .build();

        Game {
            world,
            dispatcher,
            inbox: VecDeque::new(),
            lag: 0.,
        }
    }

    pub fn handle(&mut self, message: MessageToGame) {
        self.inbox.push_back(message);
    }

    /// Advances the game by `dt` seconds, in fixed steps of 1 / FRAME_PER_SECOND.
    pub fn tick(&mut self, dt: f64) {
        self.lag = (self.lag + dt).min(MAX_LAG);
        while self.lag >= STEP {
            self.lag -= STEP;
            self.step();
        }
    }

    pub fn render(&self, renderer: &mut impl Renderer) {
        render::render(&self.world, renderer);
    }

    pub fn drain_outbox(&mut self) -> Vec<(u32, MessageToRemote)> {
        std::mem::take(&mut self.world.write_resource::<Outbox>().0)
    }

    // Every message received since the last step is applied in this step, so inputs never queue up.
    fn step(&mut self) {
        for message in self.inbox.drain(..) {
            match message {
                MessageToGame::RoomId(id) => {
                    debug!("Received room id \"{}\" from server.", id.0);
                    self.world.write_resource::<State>().room_code =
                        RoomCode::new(format!("http://192.168.178.24:8080/?room-id={}", id.0));
                }
                MessageToGame::PlayerInput(input) => {
                    self.world.write_resource::<Vec<PlayerInput>>().push(input);
                }
                MessageToGame::SynchronizePlayer(synchronize_player) => {
                    self.world.create_entity().with(synchronize_player).build();
                }
            }
        }

        self.dispatcher.dispatch(&self.world);
        self.world.maintain();
    }
}
