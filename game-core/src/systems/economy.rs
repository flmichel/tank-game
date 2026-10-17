use specs::{Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    balance,
    components::{Player, Stats},
    remotes::ConfigurationInput,
    state::match_state::Match,
};

use super::Step;

/// Spends money between rounds. Buy requests are taken out of the input and everything else is
/// left for `Lobby`, which runs next.
pub struct ApplyPurchases;

impl<'a> System<'a> for ApplyPurchases {
    type SystemData = (
        WriteStorage<'a, Player>,
        WriteExpect<'a, Match>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (mut players, mut match_state, step): Self::SystemData) {
        if !step.is_configuring() {
            return;
        }

        for player in (&mut players).join() {
            if player.input.configuration.is_empty() {
                continue;
            }

            let mut purchases = Vec::new();
            player.input.configuration.retain(|input| match input {
                ConfigurationInput::Buy(upgrade) => {
                    purchases.push(*upgrade);
                    false
                }
                _ => true,
            });

            for upgrade in purchases {
                match_state.buy(&player.id, upgrade);
            }
        }
    }
}

/// Pays out the round that just ended: kills, then the win or the loss bonus.
///
/// A draw leaves no winner, so everyone is paid as a loser: the round still funds the next
/// one. Only players that were spawned into the round are paid, which is what `Stats`
/// marks: a phone that connects mid-round has a `Player` but never took part.
pub struct AwardMoney;

impl<'a> System<'a> for AwardMoney {
    type SystemData = (
        ReadStorage<'a, Player>,
        ReadStorage<'a, Stats>,
        WriteExpect<'a, Match>,
        ReadExpect<'a, Step>,
    );

    fn run(&mut self, (players, stats, mut match_state, step): Self::SystemData) {
        if !step.round_ended {
            return;
        }

        for killer in std::mem::take(&mut match_state.kills_this_round) {
            match_state.credit(&killer, balance::KILL_REWARD);
        }

        for (player, _) in (&players, &stats).join() {
            if player.is_winner {
                match_state.win_round(&player.id);
            } else {
                match_state.lose_round(&player.id);
            }
        }

        match_state.round += 1;
    }
}
