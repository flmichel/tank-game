use game_core::{
    remotes::{ConfigurationInput, GameInput, PlayerInput, RemoteInput},
    Color, Game, MessageToGame, MessageToRemote, Rect, Renderer, Sprite, SynchronizePlayer,
};

#[derive(Default)]
struct Sprites {
    tanks: Vec<Rect>,
    missiles: usize,
}

impl Renderer for Sprites {
    fn clear(&mut self, _: Color) {}
    fn fill_rect(&mut self, _: Rect, _: Color) {}
    fn draw_line(&mut self, _: (i32, i32), _: (i32, i32), _: Color) {}
    fn draw_sprite(&mut self, sprite: Sprite, rect: Rect) {
        match sprite {
            Sprite::PlayerFace => self.tanks.push(rect),
            Sprite::Missile => self.missiles += 1,
        }
    }
    fn draw_text(&mut self, _: &str, _: i32, _: i32, _: Color) {}
}

fn send(game: &mut Game, socket_id: u32, remote_input: RemoteInput) {
    game.handle(MessageToGame::PlayerInput(PlayerInput { socket_id, remote_input }));
}

fn run_seconds(game: &mut Game, seconds: u32) {
    for _ in 0..seconds * 30 {
        game.tick(1. / 30.);
    }
}

fn start_round(game: &mut Game) {
    for socket_id in 0..2 {
        send(game, socket_id, RemoteInput::ConfigurationInput(ConfigurationInput::Ready));
    }
    run_seconds(game, 1);
    let outbox = game.drain_outbox();
    assert_eq!(outbox.len(), 2, "both players are told the round starts");
    assert!(outbox.iter().all(|(_, message)| matches!(message, MessageToRemote::GameStart)));
}

#[test]
fn a_round_ends_on_a_hit_and_the_next_one_starts_clean() {
    // Exactly 2 spawns, on one row: whichever tank is west shoots the east one
    let mut game = Game::new("s s\n", (1600, 900));
    for socket_id in 0..2 {
        game.handle(MessageToGame::SynchronizePlayer(SynchronizePlayer {
            socket_id,
            player_id: format!("player-{socket_id}"),
        }));
    }
    start_round(&mut game);

    // Both shoot east: the west bullet hits the east tank after ~2s, the east bullet is still
    // flying across the map when the round ends.
    for socket_id in 0..2 {
        send(&mut game, socket_id, RemoteInput::GameInput(GameInput::Shoot(0.)));
    }
    run_seconds(&mut game, 3);

    let outbox = game.drain_outbox();
    assert_eq!(outbox.len(), 2, "both players are told the round is over");
    assert!(outbox.iter().all(|(_, message)| matches!(message, MessageToRemote::GameOver)));

    // Ready is only accepted in the lobby, so a new start proves round 1 ended
    start_round(&mut game);
    run_seconds(&mut game, 1);

    let mut sprites = Sprites::default();
    game.render(&mut sprites);
    assert_eq!(sprites.missiles, 0, "no bullet carries over to the next round");
    // The map file sets the scale: 3 blocks across a 1600 px window
    let block_size = 1600 / 3;
    let mut spawn_columns: Vec<i32> = sprites.tanks.iter().map(|rect| rect.x / block_size).collect();
    spawn_columns.sort();
    assert_eq!(spawn_columns, [0, 2], "both tanks are alive, on the map's two spawns");
}
