//! Every tunable number in one place.
//!
//! Balancing a round-based economy means changing these fifty times, so nothing here is spread
//! across the systems that use them. Rules of thumb taken from Counter-Strike, whose economy this
//! copies in shape:
//!
//! - winning must pay far more than losing, or saving becomes a strategy;
//! - the loss bonus exists to stop a match snowballing, not to reward losing;
//! - every round has to matter, so even a lost round funds *something*.

use serde::Deserialize;

/// What a player starts a match with, enough for exactly one cheap upgrade.
pub const STARTING_MONEY: u32 = 500;
/// Paid to the last tank standing.
pub const ROUND_WIN_REWARD: u32 = 1000;
/// Paid for taking an opponent out, win or lose.
pub const KILL_REWARD: u32 = 200;
/// First lost round pays this, and each consecutive loss adds `LOSS_BONUS_PER_STREAK`.
pub const LOSS_BONUS_BASE: u32 = 300;
pub const LOSS_BONUS_PER_STREAK: u32 = 150;
/// Extra steps the bonus can climb: a fifth consecutive loss pays the same as the fourth.
/// 300 + 4 * 150 = 900, still well below a single round win.
pub const LOSS_BONUS_MAX_STREAK: u32 = 4;

/// Upgrades are bought one level at a time and each level costs more than the last.
pub const MAX_UPGRADE_LEVEL: u8 = 3;
pub const UPGRADE_PRICE_PER_LEVEL: u32 = 400;

// Level 0 stats: a tank with nothing bought behaves exactly as it did before upgrades existed.
pub const BASE_PLAYER_BLOCKS_PER_SECOND: f64 = 0.5;
pub const BASE_BULLET_BLOCKS_PER_SECOND: f64 = 1.;
pub const BASE_SECONDS_BETWEEN_SHOTS: f64 = 0.5;
pub const BASE_HIT_POINTS: u32 = 1;
pub const PLAYER_RADIUS: f64 = 0.1;
pub const BULLET_RADIUS: f64 = 0.05;

// What one level buys.
pub const BULLET_SPEED_PER_LEVEL: f64 = 0.35;
pub const MOVE_SPEED_PER_LEVEL: f64 = 0.15;
/// Multiplicative, so the reload gain shrinks as it stacks instead of reaching zero.
pub const RELOAD_FACTOR_PER_LEVEL: f64 = 0.8;
pub const HIT_POINTS_PER_ARMOR_LEVEL: u32 = 1;

/// What a player can spend money on. Wire form is kebab-case: `{"buy":"bullet-speed"}`.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Upgrade {
    BulletSpeed,
    Reload,
    MoveSpeed,
    Armor,
}

impl Upgrade {
    pub const ALL: [Upgrade; 4] =
        [Upgrade::BulletSpeed, Upgrade::Reload, Upgrade::MoveSpeed, Upgrade::Armor];

    pub fn label(&self) -> &'static str {
        match self {
            Upgrade::BulletSpeed => "bullet speed",
            Upgrade::Reload => "reload",
            Upgrade::MoveSpeed => "speed",
            Upgrade::Armor => "armor",
        }
    }
}

/// Price of taking an upgrade from `level` to `level + 1`.
pub fn upgrade_price(level: u8) -> u32 {
    UPGRADE_PRICE_PER_LEVEL * (level as u32 + 1)
}

/// What a player has bought. Survives rounds; the tank entity does not.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Loadout {
    pub bullet_speed: u8,
    pub reload: u8,
    pub move_speed: u8,
    pub armor: u8,
}

impl Loadout {
    pub fn level(&self, upgrade: Upgrade) -> u8 {
        match upgrade {
            Upgrade::BulletSpeed => self.bullet_speed,
            Upgrade::Reload => self.reload,
            Upgrade::MoveSpeed => self.move_speed,
            Upgrade::Armor => self.armor,
        }
    }

    pub fn raise(&mut self, upgrade: Upgrade) {
        let level = match upgrade {
            Upgrade::BulletSpeed => &mut self.bullet_speed,
            Upgrade::Reload => &mut self.reload,
            Upgrade::MoveSpeed => &mut self.move_speed,
            Upgrade::Armor => &mut self.armor,
        };
        *level += 1;
    }
}

/// Loss bonus for a player who has now lost `streak` rounds in a row, this one included.
pub fn loss_bonus(streak: u32) -> u32 {
    // `streak` counts this loss, so the first one pays the base and nothing more
    let extra = streak.saturating_sub(1).min(LOSS_BONUS_MAX_STREAK);
    LOSS_BONUS_BASE + LOSS_BONUS_PER_STREAK * extra
}
