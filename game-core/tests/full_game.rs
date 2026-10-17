use game_core::{
    remotes::{ConfigurationInput, PlayerInput, RemoteInput},
    Color, Game, MessageToGame, MessageToRemote, Rect, Renderer, RoomId, Sprite, SynchronizePlayer,
};

/// The map the shells ship: a full game has to fit on it.
const MAP: &str = include_str!("../../game-native/assets/map.txt");
const PLAYERS: u32 = 16;
/// Both shells draw text 24 px tall.
const TEXT_HEIGHT: i32 = 24;

#[derive(Default)]
struct Sprites {
    faces: Vec<Rect>,
    texts: Vec<(i32, i32)>,
    // The QR code is the only thing the lobby draws in black
    qr_code: Vec<Rect>,
}

impl Renderer for Sprites {
    fn clear(&mut self, _: Color) {}
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        if color == Color::BLACK {
            self.qr_code.push(rect);
        }
    }
    fn draw_line(&mut self, _: (i32, i32), _: (i32, i32), _: Color) {}
    fn draw_sprite(&mut self, sprite: Sprite, rect: Rect) {
        if matches!(sprite, Sprite::PlayerFace) {
            self.faces.push(rect);
        }
    }
    fn draw_text(&mut self, _: &str, x: i32, y: i32, _: Color) {
        self.texts.push((x, y));
    }
}

fn send(game: &mut Game, socket_id: u32, input: ConfigurationInput) {
    game.handle(MessageToGame::PlayerInput(PlayerInput {
        socket_id,
        remote_input: RemoteInput::ConfigurationInput(input),
    }));
}

/// Fills the lobby with named players, each on its own remote, in a room the server named: the
/// QR code is as wide as that room's URL makes it.
fn fill_lobby(game: &mut Game) {
    game.handle(MessageToGame::RoomId(RoomId("neHYAADXozH7I5rW_DMMpg".to_owned())));
    for socket_id in 0..PLAYERS {
        game.handle(MessageToGame::SynchronizePlayer(SynchronizePlayer {
            socket_id,
            player_id: format!("player-{socket_id}"),
        }));
        send(game, socket_id, ConfigurationInput::SetName(format!("player {socket_id}")));
    }
}

fn overlaps(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.w as i32
        && b.x < a.x + a.w as i32
        && a.y < b.y + b.h as i32
        && b.y < a.y + a.h as i32
}

fn assert_on_screen(rect: &Rect, resolution: (u32, u32)) {
    let (width, height) = (resolution.0 as i32, resolution.1 as i32);
    assert!(
        rect.x >= 0
            && rect.y >= 0
            && rect.x + rect.w as i32 <= width
            && rect.y + rect.h as i32 <= height,
        "{rect:?} is off a {width}x{height} screen"
    );
}

#[test]
fn a_full_lobby_fits_on_screen() {
    let resolution = (800, 600);
    let mut game = Game::new(MAP, resolution);
    fill_lobby(&mut game);
    game.tick(1.0);

    let mut sprites = Sprites::default();
    game.render(&mut sprites);

    assert_eq!(sprites.faces.len(), PLAYERS as usize, "every player is listed in the lobby");
    for face in &sprites.faces {
        assert_on_screen(face, resolution);
    }
    for (_, y) in &sprites.texts {
        assert!(y + TEXT_HEIGHT <= resolution.1 as i32, "a name at y {y} runs off the bottom");
    }

    assert!(!sprites.qr_code.is_empty(), "the lobby draws the room's QR code");
    for face in &sprites.faces {
        assert!(
            !sprites.qr_code.iter().any(|module| overlaps(face, module)),
            "{face:?} is drawn over the QR code, which a phone then cannot scan"
        );
    }

    let mut rows: Vec<i32> = sprites.faces.iter().map(|face| face.y).collect();
    rows.sort();
    rows.dedup();
    assert_eq!(rows.len(), PLAYERS as usize, "no two players share a row");
}

#[test]
fn every_player_of_a_full_game_gets_its_own_spawn() {
    let resolution = (1600, 900);
    let mut game = Game::new(MAP, resolution);
    fill_lobby(&mut game);
    for socket_id in 0..PLAYERS {
        send(&mut game, socket_id, ConfigurationInput::Ready);
    }
    game.tick(1.0);

    let outbox = game.drain_outbox();
    assert_eq!(outbox.len(), PLAYERS as usize, "every player is told the game starts");
    assert!(outbox.iter().all(|(_, message)| matches!(message, MessageToRemote::GameStart)));

    let mut sprites = Sprites::default();
    game.render(&mut sprites);

    assert_eq!(sprites.faces.len(), PLAYERS as usize, "every player spawned and is drawn");
    for face in &sprites.faces {
        assert_on_screen(face, resolution);
    }

    let mut spawns: Vec<(i32, i32)> = sprites.faces.iter().map(|face| (face.x, face.y)).collect();
    spawns.sort();
    spawns.dedup();
    assert_eq!(spawns.len(), PLAYERS as usize, "no two players share a spawn");
}

#[test]
fn one_ready_player_starts_a_round_that_keeps_running() {
    let resolution = (1600, 900);
    let mut game = Game::new(MAP, resolution);
    game.handle(MessageToGame::SynchronizePlayer(SynchronizePlayer {
        socket_id: 0,
        player_id: "player-0".to_owned(),
    }));
    send(&mut game, 0, ConfigurationInput::Ready);
    game.tick(1.0);

    let outbox = game.drain_outbox();
    assert_eq!(outbox.len(), 1, "the solo player is told the game starts");
    assert!(matches!(outbox[0].1, MessageToRemote::GameStart));

    // Nothing can take the last player off the board, so the round does not end on its own
    for _ in 0..300 {
        game.tick(1. / 30.);
    }
    assert!(
        game.drain_outbox().is_empty(),
        "a solo round keeps running: no GameOver, and so no second GameStart"
    );

    let mut sprites = Sprites::default();
    game.render(&mut sprites);
    assert_eq!(sprites.faces.len(), 1, "the solo player is still on the board");
}
