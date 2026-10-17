use std::collections::VecDeque;

use specs::{Builder, Component, Dispatcher, DispatcherBuilder, VecStorage, World, WorldExt};
use specs_derive::Component;
use tracing::debug;

use crate::{
    balance::{Loadout, Upgrade},
    bot::{BotKind, BOT_ID_PREFIX},
    components::{Bullet, Circle, Movement, Player, Position, Stats},
    remotes::{ConfigurationInput, PlayerInput, RemoteInput},
    render::{self, Renderer},
    room_code::RoomCode,
    state::{
        game_state::{Phase, State},
        match_state::Match,
        Map,
    },
    systems::{
        ApplyInputs, ApplyPurchases, AwardMoney, BotThink, Lobby, MoveAndCollideWalls,
        ResolveHits, ResolveRound, RetrievePlayerForInputs, Shoot, Step,
        SynchronizePlayerToRemote, TickShotCooldown,
    },
};

// A step costs ~1 µs, so a high rate is free and keeps the wait for the next step under 9 ms
pub const STEPS_PER_SECOND: u32 = 120;
const STEP: f64 = 1. / STEPS_PER_SECOND as f64;
// After a long pause (e.g. a hidden browser tab) drop the backlog instead of fast-forwarding.
const MAX_LAG: f64 = 0.25;

/// Inbound port: everything the outside world tells the game.
pub enum MessageToGame {
    PlayerInput(PlayerInput),
    RoomId(RoomId),
    SynchronizePlayer(SynchronizePlayer),
    /// Adds a bot player, which needs no remote and readies itself up.
    AddBot(BotKind),
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
    bots_added: u32,
    /// Mixed into each bot's generator, so bots differ between matches as well as from
    /// each other.
    seed: u64,
}

impl Game {
    pub fn new(map: &str, resolution: (u32, u32)) -> Self {
        Game::new_with_seed(map, resolution, rand::random())
    }

    /// Same game, with spawn draws that repeat for a given seed. Bot decisions are already
    /// reproducible, so a match of bots alone replays exactly, which is what a balance
    /// sweep needs.
    pub fn new_with_seed(map: &str, resolution: (u32, u32), seed: u64) -> Self {
        let mut world = World::new();
        world.register::<RoomId>();
        world.register::<SynchronizePlayer>();
        world.register::<Position>();
        world.register::<Movement>();
        world.register::<Circle>();
        world.register::<Player>();
        world.register::<Stats>();
        world.register::<Bullet>();

        world.insert(State {
            room_code: RoomCode::new("Error, the game could not connect to server".to_owned()),
            phase: Phase::BeforeNextGame,
            number_of_ready_players: 0,
            map: Map::parse_with_seed(map, resolution, seed),
            resolution,
        });
        world.insert(Match::default());
        world.insert(Outbox::default());
        world.insert(Vec::<PlayerInput>::new());
        world.insert(Step::new(Phase::BeforeNextGame));

        // One job per system, in order: a player must exist before its inputs are
        // folded in, inputs before they are applied, movement before collisions, and the
        // round must be resolved before anything spawns into the next step.
        let dispatcher = DispatcherBuilder::new()
            .with(SynchronizePlayerToRemote, "SynchronizePlayerToRemote", &[])
            .with(
                RetrievePlayerForInputs,
                "RetrievePlayerForInputs",
                &["SynchronizePlayerToRemote"],
            )
            .with(BotThink, "BotThink", &["RetrievePlayerForInputs"])
            .with(ApplyPurchases, "ApplyPurchases", &["BotThink"])
            .with(Lobby, "Lobby", &["ApplyPurchases"])
            .with(ApplyInputs, "ApplyInputs", &["Lobby"])
            .with(MoveAndCollideWalls, "MoveAndCollideWalls", &["ApplyInputs"])
            .with(ResolveHits, "ResolveHits", &["MoveAndCollideWalls"])
            .with(ResolveRound, "ResolveRound", &["ResolveHits"])
            .with(AwardMoney, "AwardMoney", &["ResolveRound"])
            .with(Shoot, "Shoot", &["AwardMoney"])
            .with(TickShotCooldown, "TickShotCooldown", &["Shoot"])
            .build();

        Game {
            world,
            dispatcher,
            inbox: VecDeque::new(),
            bots_added: 0,
            seed,
            lag: 0.,
        }
    }

    pub fn handle(&mut self, message: MessageToGame) {
        self.inbox.push_back(message);
    }

    /// Advances the game by `dt` seconds, in fixed steps of 1 / STEPS_PER_SECOND.
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
                MessageToGame::AddBot(kind) => {
                    self.bots_added += 1;
                    let id = format!("{}{}", BOT_ID_PREFIX, self.bots_added);
                    debug!("Adding {} bot \"{}\".", kind.label(), id);
                    let player = Player::new_bot(id, kind, bot_seed(self.seed, self.bots_added));
                    self.world.create_entity().with(player).build();
                }
                MessageToGame::SynchronizePlayer(synchronize_player) => {
                    self.world.create_entity().with(synchronize_player).build();
                }
            }
        }

        // Every system in this step agrees on the phase, even when one of them changes it
        let phase = self.world.read_resource::<State>().phase;
        *self.world.write_resource::<Step>() = Step::new(phase);

        self.dispatcher.dispatch(&self.world);
        self.world.maintain();
    }
}

/// Seeds each bot from the match seed and its own number, so bot decisions are reproducible
/// for a given match and no two bots run in lockstep.
fn bot_seed(match_seed: u64, bot_number: u32) -> u64 {
    // Multiplied, not XOR-ed: neighbouring bot numbers must not produce seeds that differ
    // in one bit, or two bots wander and fire in lockstep
    0x9E37_79B9_7F4A_7C15_u64
        .wrapping_mul(bot_number as u64 + 1)
        .wrapping_add(match_seed)
}

impl Game {
    /// Money `player_id` can spend between rounds.
    pub fn wallet(&self, player_id: &str) -> u32 {
        self.world.read_resource::<Match>().wallet(player_id)
    }

    /// Rounds `player_id` has won in this match.
    pub fn round_wins(&self, player_id: &str) -> u32 {
        self.world.read_resource::<Match>().round_wins(player_id)
    }

    /// What `player_id` has bought.
    pub fn loadout(&self, player_id: &str) -> Loadout {
        self.world.read_resource::<Match>().loadout(player_id)
    }

    /// Rounds completed in this match.
    pub fn rounds_played(&self) -> u32 {
        self.world.read_resource::<Match>().round
    }
}

impl Game {
    /// Puts an upgrade on a player without charging for it, for harnesses and tests.
    pub fn grant_upgrade(&mut self, player_id: &str, upgrade: Upgrade) {
        self.world
            .write_resource::<Match>()
            .grant(player_id, upgrade);
    }
}
