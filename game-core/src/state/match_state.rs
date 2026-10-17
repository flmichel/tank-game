use std::collections::HashMap;

use tracing::debug;

use crate::balance::{self, Loadout, Upgrade};

/// Everything that outlives a round.
///
/// Round entities are destroyed and rebuilt every round, so none of this can live on them. It is
/// keyed by `Player::id`, which a remote keeps across reconnects, so a player who drops mid-match
/// comes back to its money and its tank.
#[derive(Default)]
pub struct Match {
    /// Rounds completed so far.
    pub round: u32,
    wallets: HashMap<String, u32>,
    round_wins: HashMap<String, u32>,
    loss_streaks: HashMap<String, u32>,
    loadouts: HashMap<String, Loadout>,
    /// Ids of players who took someone out this round, recorded by `ResolveHits` and paid out by
    /// `AwardMoney`.
    pub kills_this_round: Vec<String>,
}

impl Match {
    pub fn wallet(&self, player_id: &str) -> u32 {
        self.wallets
            .get(player_id)
            .copied()
            .unwrap_or(balance::STARTING_MONEY)
    }

    pub fn round_wins(&self, player_id: &str) -> u32 {
        self.round_wins.get(player_id).copied().unwrap_or(0)
    }

    pub fn loss_streak(&self, player_id: &str) -> u32 {
        self.loss_streaks.get(player_id).copied().unwrap_or(0)
    }

    pub fn loadout(&self, player_id: &str) -> Loadout {
        self.loadouts.get(player_id).copied().unwrap_or_default()
    }

    pub fn credit(&mut self, player_id: &str, amount: u32) {
        let wallet = self
            .wallets
            .entry(player_id.to_owned())
            .or_insert(balance::STARTING_MONEY);
        *wallet += amount;
    }

    pub fn win_round(&mut self, player_id: &str) {
        self.credit(player_id, balance::ROUND_WIN_REWARD);
        *self.round_wins.entry(player_id.to_owned()).or_insert(0) += 1;
        self.loss_streaks.insert(player_id.to_owned(), 0);
    }

    pub fn lose_round(&mut self, player_id: &str) {
        let streak = self
            .loss_streaks
            .entry(player_id.to_owned())
            .or_insert(0);
        *streak += 1;
        let bonus = balance::loss_bonus(*streak);
        self.credit(player_id, bonus);
    }

    /// Buys one level of `upgrade`, if the player can afford it and it is not maxed out.
    /// Returns whether the purchase went through.
    pub fn buy(&mut self, player_id: &str, upgrade: Upgrade) -> bool {
        let mut loadout = self.loadout(player_id);
        let level = loadout.level(upgrade);
        if level >= balance::MAX_UPGRADE_LEVEL {
            debug!("{player_id} cannot buy {:?}: already at max level", upgrade);
            return false;
        }

        let price = balance::upgrade_price(level);
        let wallet = self.wallet(player_id);
        if wallet < price {
            debug!("{player_id} cannot afford {:?}: {wallet} < {price}", upgrade);
            return false;
        }

        self.wallets.insert(player_id.to_owned(), wallet - price);
        loadout.raise(upgrade);
        self.loadouts.insert(player_id.to_owned(), loadout);
        debug!("{player_id} bought {:?} for {price}", upgrade);
        true
    }
}

impl Match {
    /// Raises an upgrade level without charging for it.
    ///
    /// The game itself only ever grants upgrades through `buy`; this exists for harnesses that
    /// need a given loadout on the board, such as a balance sweep.
    pub fn grant(&mut self, player_id: &str, upgrade: Upgrade) {
        let mut loadout = self.loadout(player_id);
        if loadout.level(upgrade) >= balance::MAX_UPGRADE_LEVEL {
            return;
        }
        loadout.raise(upgrade);
        self.loadouts.insert(player_id.to_owned(), loadout);
    }
}
