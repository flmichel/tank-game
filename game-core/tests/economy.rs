//! The money a match hands out, round after round.

use std::collections::HashMap;

use game_core::{
    balance,
    remotes::{ConfigurationInput, GameInput, PlayerInput, RemoteInput},
    Color, Game, MessageToGame, MessageToRemote, Rect, Renderer, Sprite, SynchronizePlayer,
};

const SHOOTER: &str = "player-0";
const TARGET: &str = "player-1";

/// Where each tank is drawn, by name. Spawns are handed out at random, so a test that needs a
/// specific player to win has to find its opponent before it shoots.
#[derive(Default)]
struct Frame(HashMap<String, (i32, i32)>);

impl Renderer for Frame {
    fn clear(&mut self, _: Color) {}
    fn fill_rect(&mut self, _: Rect, _: Color) {}
    fn draw_line(&mut self, _: (i32, i32), _: (i32, i32), _: Color) {}
    fn draw_sprite(&mut self, _: Sprite, _: Rect) {}
    fn draw_text(&mut self, text: &str, x: i32, y: i32, _: Color) {
        self.0.insert(text.to_owned(), (x, y));
    }
}

fn send(game: &mut Game, socket_id: u32, remote_input: RemoteInput) {
    game.handle(MessageToGame::PlayerInput(PlayerInput { socket_id, remote_input }));
}

fn run_seconds(game: &mut Game, seconds: u32) {
    for _ in 0..seconds * 30 {
        game.tick(1. / 30.);
    }
}

/// Exactly 2 spawns, two blocks apart on one row.
fn two_player_game() -> Game {
    let mut game = Game::new("s s\n", (1600, 900));
    for (socket_id, name) in [SHOOTER, TARGET].iter().enumerate() {
        let socket_id = socket_id as u32;
        game.handle(MessageToGame::SynchronizePlayer(SynchronizePlayer {
            socket_id,
            player_id: (*name).to_owned(),
        }));
        send(
            &mut game,
            socket_id,
            RemoteInput::ConfigurationInput(ConfigurationInput::SetName((*name).to_owned())),
        );
    }
    game
}

/// `SHOOTER` takes `TARGET` out, whichever spawn each of them drew, so the same player
/// wins every round and the loser builds a streak.
fn play_a_round(game: &mut Game) {
    play_a_round_won_by(game, 0);
}

/// Readies both players, then has `socket_id` shoot the other one down.
fn play_a_round_won_by(game: &mut Game, socket_id: u32) {
    for socket in 0..2 {
        send(game, socket, RemoteInput::ConfigurationInput(ConfigurationInput::Ready));
    }
    run_seconds(game, 1);

    // Aim at wherever the opponent actually spawned
    let mut frame = Frame::default();
    game.render(&mut frame);
    let (shooter_name, target_name) =
        if socket_id == 0 { (SHOOTER, TARGET) } else { (TARGET, SHOOTER) };
    let shooter = frame.0[shooter_name];
    let target = frame.0[target_name];
    let aim = ((target.1 - shooter.1) as f64).atan2((target.0 - shooter.0) as f64);

    send(game, socket_id, RemoteInput::GameInput(GameInput::Shoot(aim)));
    run_seconds(game, 3);

    assert!(
        game.drain_outbox()
            .iter()
            .any(|(_, message)| matches!(message, MessageToRemote::GameOver)),
        "the shot should have ended the round"
    );
}
#[test]
fn a_won_round_pays_the_win_and_the_kill() {
    let mut game = two_player_game();
    play_a_round(&mut game);

    assert_eq!(game.rounds_played(), 1);
    assert_eq!(game.round_wins(SHOOTER), 1);
    assert_eq!(game.round_wins(TARGET), 0);

    assert_eq!(
        game.wallet(SHOOTER),
        balance::STARTING_MONEY + balance::ROUND_WIN_REWARD + balance::KILL_REWARD,
        "the winner is paid for the round and for the kill"
    );
    assert_eq!(
        game.wallet(TARGET),
        balance::STARTING_MONEY + balance::loss_bonus(1),
        "a lost round still funds the next one"
    );
}

#[test]
fn the_loss_bonus_grows_with_the_streak() {
    let mut game = two_player_game();
    for _ in 0..3 {
        play_a_round(&mut game);
    }

    assert_eq!(game.rounds_played(), 3);
    assert_eq!(game.round_wins(SHOOTER), 3);
    assert_eq!(game.round_wins(TARGET), 0);

    let won_each_round = balance::ROUND_WIN_REWARD + balance::KILL_REWARD;
    assert_eq!(game.wallet(SHOOTER), balance::STARTING_MONEY + 3 * won_each_round);

    // Three rounds lost in a row pay a bonus that grows every time
    let lost = balance::loss_bonus(1) + balance::loss_bonus(2) + balance::loss_bonus(3);
    assert_eq!(game.wallet(TARGET), balance::STARTING_MONEY + lost);
    assert!(
        game.wallet(TARGET) < game.wallet(SHOOTER),
        "losing must never out-earn winning, or saving becomes a strategy"
    );
}

#[test]
fn a_win_resets_the_loss_streak() {
    let mut game = two_player_game();
    play_a_round(&mut game);
    let after_one_loss = game.wallet(TARGET);

    // The loser of round 1 wins round 2: its next loss pays the first-loss bonus again
    play_a_round_won_by(&mut game, 1);
    play_a_round(&mut game);

    assert_eq!(
        game.wallet(TARGET),
        after_one_loss + balance::ROUND_WIN_REWARD + balance::KILL_REWARD + balance::loss_bonus(1),
        "a win in between resets the streak, so the next loss is a first loss again"
    );
}



#[test]
fn the_first_loss_pays_the_base_bonus() {
    // Pins what the balance table documents, rather than restating the formula
    assert_eq!(balance::loss_bonus(1), balance::LOSS_BONUS_BASE);
    assert_eq!(
        balance::loss_bonus(2),
        balance::LOSS_BONUS_BASE + balance::LOSS_BONUS_PER_STREAK
    );
    assert!(
        balance::loss_bonus(99) < balance::ROUND_WIN_REWARD,
        "no streak may ever pay more than winning"
    );
}
