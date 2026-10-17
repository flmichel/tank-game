//! Bots are players without a remote.

use game_core::{
    balance,
    bot::BotKind,
    remotes::{ConfigurationInput, PlayerInput, RemoteInput},
    Color, Game, MessageToGame, MessageToRemote, Rect, Renderer, Sprite, SynchronizePlayer,
};

#[derive(Default)]
struct Sprites {
    tanks: usize,
    missiles: usize,
}

impl Renderer for Sprites {
    fn clear(&mut self, _: Color) {}
    fn fill_rect(&mut self, _: Rect, _: Color) {}
    fn draw_line(&mut self, _: (i32, i32), _: (i32, i32), _: Color) {}
    fn draw_sprite(&mut self, sprite: Sprite, _: Rect) {
        match sprite {
            Sprite::PlayerFace => self.tanks += 1,
            Sprite::Missile => self.missiles += 1,
        }
    }
    fn draw_text(&mut self, _: &str, _: i32, _: i32, _: Color) {}
}

fn frame(game: &Game) -> Sprites {
    let mut sprites = Sprites::default();
    game.render(&mut sprites);
    sprites
}

fn run_seconds(game: &mut Game, seconds: u32) {
    for _ in 0..seconds * 30 {
        game.tick(1. / 30.);
    }
}

#[test]
fn a_dummy_bot_fills_a_seat_without_a_remote() {
    let mut game = Game::new("s s\n", (1600, 900));
    game.handle(MessageToGame::SynchronizePlayer(SynchronizePlayer {
        socket_id: 0,
        player_id: "player-0".to_owned(),
    }));
    game.handle(MessageToGame::AddBot(BotKind::Dummy));
    game.handle(MessageToGame::PlayerInput(PlayerInput {
        socket_id: 0,
        remote_input: RemoteInput::ConfigurationInput(ConfigurationInput::Ready),
    }));
    run_seconds(&mut game, 1);

    // The bot readies itself up, so one ready human is enough to start
    let outbox = game.drain_outbox();
    assert_eq!(outbox.len(), 1, "only the human has a remote to be told about the round");
    assert!(matches!(outbox[0].1, MessageToRemote::GameStart));
    assert_eq!(outbox[0].0, 0, "the message goes to the human's socket");

    assert_eq!(frame(&game).tanks, 2, "the human and the bot are both on the board");

    // A dummy is target practice: it never shoots, so a solo human can test the controls
    run_seconds(&mut game, 5);
    assert_eq!(frame(&game).missiles, 0, "a dummy bot fires nothing");
    assert!(
        game.drain_outbox().is_empty(),
        "nothing killed anyone, so the round is still running"
    );
}

#[test]
fn a_hunter_bot_closes_in_and_wins_the_round() {
    let mut game = Game::new("s s\n", (1600, 900));
    game.handle(MessageToGame::AddBot(BotKind::Hunter));
    game.handle(MessageToGame::AddBot(BotKind::Dummy));

    // Bots ready themselves up, so a game of only bots plays on its own
    for _ in 0..30 * 30 {
        game.tick(1. / 30.);
        if game.rounds_played() > 0 {
            break;
        }
    }

    assert_eq!(game.rounds_played(), 1, "the hunter should have ended the round");
    assert_eq!(game.round_wins("bot-1"), 1, "the hunter won");
    assert_eq!(game.round_wins("bot-2"), 0, "the dummy lost");
    assert_eq!(
        game.wallet("bot-1"),
        balance::STARTING_MONEY + balance::ROUND_WIN_REWARD + balance::KILL_REWARD
    );
    assert_eq!(game.wallet("bot-2"), balance::STARTING_MONEY + balance::loss_bonus(1));
}

#[test]
fn a_seeded_match_of_bots_replays() {
    // What a balance sweep rests on: two runs of the same seed must differ only in what the
    // sweep changed. Wanderers are the bots that actually draw from their generator.
    let outcome = |seed| {
        let mut game = Game::new_with_seed("s    s\n", (1600, 900), seed);
        game.handle(MessageToGame::AddBot(BotKind::Wanderer));
        game.handle(MessageToGame::AddBot(BotKind::Wanderer));
        for _ in 0..30 * 30 {
            game.tick(1. / 30.);
            if game.rounds_played() > 0 {
                break;
            }
        }
        (
            game.rounds_played(),
            game.wallet("bot-1"),
            game.wallet("bot-2"),
        )
    };

    assert_eq!(outcome(7), outcome(7), "the same seed must replay exactly");
    assert_eq!(outcome(12), outcome(12));
}
