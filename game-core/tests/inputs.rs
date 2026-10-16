use std::{collections::HashMap, f64::consts::PI};

use game_core::{
    remotes::{ConfigurationInput, GameInput, RemoteInput, Sticks},
    Color, Game, MessageToGame, Rect, Renderer, Sprite, STEPS_PER_SECOND,
};

const STEP: f64 = 1. / STEPS_PER_SECOND as f64;

/// Where each tank (by name) and missile is drawn.
#[derive(Default)]
struct Frame {
    tanks: HashMap<String, (i32, i32)>,
    missiles: Vec<(i32, i32)>,
}

impl Renderer for Frame {
    fn clear(&mut self, _: Color) {}
    fn fill_rect(&mut self, _: Rect, _: Color) {}
    fn draw_line(&mut self, _: (i32, i32), _: (i32, i32), _: Color) {}
    fn draw_sprite(&mut self, sprite: Sprite, rect: Rect) {
        if let Sprite::Missile = sprite {
            self.missiles.push((rect.x, rect.y));
        }
    }
    fn draw_text(&mut self, text: &str, x: i32, y: i32, _: Color) {
        self.tanks.insert(text.to_owned(), (x, y));
    }
}

fn frame(game: &Game) -> Frame {
    let mut frame = Frame::default();
    game.render(&mut frame);
    frame
}

fn send(game: &mut Game, socket_id: u32, remote_input: RemoteInput) {
    game.handle(MessageToGame::from_remote(socket_id, remote_input));
}

fn sticks(seq: u32, movement: Option<f64>, aim: Option<f64>) -> RemoteInput {
    RemoteInput::Sticks(Sticks { seq, movement, aim })
}

#[test]
fn every_input_queued_before_a_step_is_applied_in_that_step() {
    let mut game = Game::new("s      s\n", (1600, 900));
    for (socket_id, name) in [(0, "mover"), (1, "shooter")] {
        for input in [
            ConfigurationInput::PlayerId(name.to_owned()),
            ConfigurationInput::SetName(name.to_owned()),
            ConfigurationInput::Ready,
        ] {
            send(&mut game, socket_id, RemoteInput::ConfigurationInput(input));
        }
    }
    game.tick(STEP);
    assert_eq!(
        game.drain_outbox().len(),
        2,
        "both players joined, got ready and started the game in one step"
    );
    let start = frame(&game);

    // The mover goes east, then a late state from before arrives
    send(&mut game, 0, sticks(2, Some(0.), None));
    send(&mut game, 0, sticks(1, None, None));
    // The shooter's last aim state says west, its shot says south, then the release state follows
    send(&mut game, 1, sticks(1, None, Some(PI)));
    send(
        &mut game,
        1,
        RemoteInput::GameInput(GameInput::Shoot(PI / 2.)),
    );
    send(&mut game, 1, sticks(2, None, None));
    game.tick(STEP);
    game.tick(STEP);
    let first = frame(&game);
    for _ in 0..5 {
        game.tick(STEP);
    }
    let last = frame(&game);

    let (mover_start, mover) = (start.tanks["mover"], last.tanks["mover"]);
    assert!(
        mover.0 > mover_start.0 && mover.1 == mover_start.1,
        "the stale stop is ignored"
    );

    assert_eq!(
        (first.missiles.len(), last.missiles.len()),
        (1, 1),
        "the shot is fired"
    );
    let (from, to) = (first.missiles[0], last.missiles[0]);
    assert!(
        to.1 > from.1 && to.0 == from.0,
        "the missile flies south, not west"
    );
}
