//! Bots are players without a remote.
//!
//! A bot never touches the world directly: `BotThink` fills the same `Player::input` a phone
//! would, and every system downstream treats it like any other player. That keeps bots honest
//! (they cannot do what a human could not) and makes them usable as a balance instrument.

use std::f64::consts::PI;

/// How a bot plays. Listed from "target practice" to "actual opponent".
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BotKind {
    /// Spawns and does nothing: a target to shoot at while testing alone.
    #[default]
    Dummy,
    /// Walks in random directions and fires on cooldown. Enough to exercise a full round.
    Wanderer,
    /// Closes on the nearest living opponent and shoots when it has a clear line.
    Hunter,
}

impl BotKind {
    pub fn parse(name: &str) -> Option<BotKind> {
        match name {
            "dummy" => Some(BotKind::Dummy),
            "wanderer" => Some(BotKind::Wanderer),
            "hunter" => Some(BotKind::Hunter),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            BotKind::Dummy => "dummy",
            BotKind::Wanderer => "wanderer",
            BotKind::Hunter => "hunter",
        }
    }
}

/// Prefix of every bot player id, which `SynchronizePlayerToRemote` keeps remotes out of.
pub const BOT_ID_PREFIX: &str = "bot-";

/// Steps a wanderer keeps a heading before picking a new one (half a second at 120 Hz).
pub const WANDER_STEPS: u32 = 60;
/// A hunter stops closing in once it is this near, so it does not drive into its target.
pub const HUNTER_PREFERRED_DISTANCE: f64 = 1.5;

pub struct Bot {
    pub kind: BotKind,
    pub rng: Rng,
    /// Steps left on the current heading, for `BotKind::Wanderer`.
    pub steps_until_turn: u32,
}

impl Bot {
    pub fn new(kind: BotKind, seed: u64) -> Bot {
        Bot { kind, rng: Rng::new(seed), steps_until_turn: 0 }
    }
}

/// xorshift64*, seeded per bot.
///
/// Bots carry their own generator rather than reaching for a thread-local one, so a bot
/// given the same board always decides the same thing: the test suite depends on it, and a
/// balance sweep needs runs that differ only in what is being swept.
///
/// A whole match still does not replay: `Map::get_spawn_block` draws from the global
/// generator, so who spawns where changes every round.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        // Only zero is a fixed point for xorshift; forcing a bit instead would erase
        // differences between seeds that are one bit apart
        Rng(if seed == 0 { 1 } else { seed })
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Uniform in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform angle in radians.
    pub fn next_angle(&mut self) -> f64 {
        self.next_f64() * 2. * PI
    }
}
