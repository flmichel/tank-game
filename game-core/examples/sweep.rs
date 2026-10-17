//! Bot-vs-bot balance sweep: how much does one upgrade actually buy?
//!
//! Runs seeded hunter-vs-hunter matches with one side upgraded and the other stock, and reports
//! how often the upgraded tank wins. A round win rate well above 50% means the upgrade is worth
//! more than its price; near 50% means it is cosmetic.
//!
//! ```text
//! cargo run -p game-core --example sweep                      # bullet-speed vs a wanderer
//! cargo run -p game-core --example sweep -- reload 100 hunter

//! ```
//!
//! Matches of bots alone replay exactly for a given seed, so two runs of this sweep differ only
//! in what is being swept.

use game_core::{
    balance::{self, Upgrade},
    bot::BotKind,
    Game, MessageToGame,
};

/// Two spawns four blocks apart: enough room that bullet speed and reload both matter.
const MAP: &str = "s    s\n";
const RESOLUTION: (u32, u32) = (1600, 900);
/// A round that has not resolved by here is a stalemate, not a win for anyone.
const MAX_ROUND_SECONDS: u32 = 30;
const UPGRADED: &str = "bot-1";
const STOCK: &str = "bot-2";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let upgrade = args
        .first()
        .map(|name| parse_upgrade(name))
        .unwrap_or(Upgrade::BulletSpeed);
    let rounds: u32 = args.get(1).and_then(|n| n.parse().ok()).unwrap_or(40);
    // A deterministic opponent makes every round a mirror match, which only ever answers
    // "does this upgrade decide a duel". A wanderer brings the variance a rate needs.
    let opponent = args
        .get(2)
        .and_then(|name| BotKind::parse(name))
        .unwrap_or(BotKind::Wanderer);

    println!(
        "{} upgraded hunter vs {}, {rounds} rounds per level, on \"{}\"",
        upgrade.label(),
        opponent.label(),
        MAP.trim_end()
    );
    println!("{:>7}  {:>6}  {:>6}  {:>6}  {:>8}", "level", "price", "won", "drawn", "win rate");

    for level in 0..=balance::MAX_UPGRADE_LEVEL {
        let outcome = run_level(upgrade, level, rounds, opponent);
        let price: u32 = (0..level).map(balance::upgrade_price).sum();
        println!(
            "{level:>7}  {price:>6}  {:>6}  {:>6}  {:>7.0}%",
            outcome.upgraded_wins,
            outcome.draws,
            100. * outcome.upgraded_wins as f64 / rounds as f64,
        );
    }
}

#[derive(Default)]
struct Outcome {
    upgraded_wins: u32,
    draws: u32,
}

/// One round per seed, so a stalemate cannot poison the rounds after it.
fn run_level(upgrade: Upgrade, level: u8, rounds: u32, opponent: BotKind) -> Outcome {
    let mut outcome = Outcome::default();

    for round in 0..rounds {
        let mut game = Game::new_with_seed(MAP, RESOLUTION, round as u64);
        game.handle(MessageToGame::AddBot(BotKind::Hunter));
        game.handle(MessageToGame::AddBot(opponent));
        for _ in 0..level {
            game.grant_upgrade(UPGRADED, upgrade);
        }

        for _ in 0..MAX_ROUND_SECONDS * 30 {
            game.tick(1. / 30.);
            if game.rounds_played() > 0 {
                break;
            }
        }

        if game.rounds_played() == 0 {
            eprintln!("level {level}, seed {round}: round never resolved");
            continue;
        }
        if game.round_wins(UPGRADED) > 0 {
            outcome.upgraded_wins += 1;
        } else if game.round_wins(STOCK) == 0 {
            // Both died in the same step
            outcome.draws += 1;
        }
    }

    outcome
}

fn parse_upgrade(name: &str) -> Upgrade {
    match name {
        "bullet-speed" => Upgrade::BulletSpeed,
        "reload" => Upgrade::Reload,
        "move-speed" => Upgrade::MoveSpeed,
        "armor" => Upgrade::Armor,
        other => {
            eprintln!("unknown upgrade \"{other}\", try one of:");
            for upgrade in Upgrade::ALL {
                eprintln!("  {:?}", upgrade);
            }
            std::process::exit(2);
        }
    }
}
