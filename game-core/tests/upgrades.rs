//! What money buys, and when it may be spent.

use game_core::{
    balance::{self, Upgrade},
    remotes::{ConfigurationInput, GameInput, PlayerInput, RemoteInput},
    Color, Game, MessageToGame, Rect, Renderer, Sprite, SynchronizePlayer, STEPS_PER_SECOND,
};

/// One spawn and room to the east for a bullet to fly down.
const MAP: &str = "s    \n";
const PLAYER: &str = "player-0";

#[derive(Default)]
struct Missiles(Vec<i32>);

impl Renderer for Missiles {
    fn clear(&mut self, _: Color) {}
    fn fill_rect(&mut self, _: Rect, _: Color) {}
    fn draw_line(&mut self, _: (i32, i32), _: (i32, i32), _: Color) {}
    fn draw_sprite(&mut self, sprite: Sprite, rect: Rect) {
        if let Sprite::Missile = sprite {
            self.0.push(rect.x);
        }
    }
    fn draw_text(&mut self, _: &str, _: i32, _: i32, _: Color) {}
}

fn send(game: &mut Game, remote_input: RemoteInput) {
    game.handle(MessageToGame::PlayerInput(PlayerInput { socket_id: 0, remote_input }));
}

fn configure(game: &mut Game, input: ConfigurationInput) {
    send(game, RemoteInput::ConfigurationInput(input));
}

fn run_steps(game: &mut Game, steps: u32) {
    for _ in 0..steps {
        game.tick(1. / STEPS_PER_SECOND as f64);
    }
}

/// A solo player who has bought `upgrades`, one second into firing a shot east.
fn bullet_x_after_a_second(upgrades: &[Upgrade]) -> i32 {
    let mut game = Game::new(MAP, (1600, 900));
    game.handle(MessageToGame::SynchronizePlayer(SynchronizePlayer {
        socket_id: 0,
        player_id: PLAYER.to_owned(),
    }));
    for upgrade in upgrades {
        configure(&mut game, ConfigurationInput::Buy(*upgrade));
    }
    configure(&mut game, ConfigurationInput::Ready);
    run_steps(&mut game, 1);

    send(&mut game, RemoteInput::GameInput(GameInput::Shoot(0.)));
    run_steps(&mut game, STEPS_PER_SECOND);

    let mut missiles = Missiles::default();
    game.render(&mut missiles);
    assert_eq!(missiles.0.len(), 1, "the shot is in flight");
    missiles.0[0]
}

#[test]
fn bought_bullet_speed_makes_the_shot_faster() {
    let stock = bullet_x_after_a_second(&[]);
    let upgraded = bullet_x_after_a_second(&[Upgrade::BulletSpeed]);

    assert!(
        upgraded > stock,
        "an upgraded bullet should be further down the map after a second: {upgraded} vs {stock}"
    );
}

#[test]
fn an_upgrade_is_paid_for_and_capped_by_the_wallet() {
    let mut game = Game::new(MAP, (1600, 900));
    game.handle(MessageToGame::SynchronizePlayer(SynchronizePlayer {
        socket_id: 0,
        player_id: PLAYER.to_owned(),
    }));
    run_steps(&mut game, 1);
    assert_eq!(game.wallet(PLAYER), balance::STARTING_MONEY);

    configure(&mut game, ConfigurationInput::Buy(Upgrade::BulletSpeed));
    run_steps(&mut game, 1);
    let first_price = balance::upgrade_price(0);
    assert_eq!(game.loadout(PLAYER).bullet_speed, 1, "the first level is bought");
    assert_eq!(game.wallet(PLAYER), balance::STARTING_MONEY - first_price);

    // The second level costs more than what is left: the purchase is refused, not borrowed against
    configure(&mut game, ConfigurationInput::Buy(Upgrade::BulletSpeed));
    run_steps(&mut game, 1);
    assert_eq!(game.loadout(PLAYER).bullet_speed, 1, "the second level is unaffordable");
    assert_eq!(game.wallet(PLAYER), balance::STARTING_MONEY - first_price);
}

#[test]
fn money_cannot_be_spent_during_a_round() {
    let mut game = Game::new(MAP, (1600, 900));
    game.handle(MessageToGame::SynchronizePlayer(SynchronizePlayer {
        socket_id: 0,
        player_id: PLAYER.to_owned(),
    }));
    configure(&mut game, ConfigurationInput::Ready);
    run_steps(&mut game, 2);

    // The round is running: a buy request is dropped like any other configuration input
    configure(&mut game, ConfigurationInput::Buy(Upgrade::Armor));
    run_steps(&mut game, 2);

    assert_eq!(game.wallet(PLAYER), balance::STARTING_MONEY, "nothing was spent mid-round");
    assert_eq!(game.loadout(PLAYER), Default::default(), "nothing was bought mid-round");
}
