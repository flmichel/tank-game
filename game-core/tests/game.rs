use game_core::{
    remotes::{ConfigurationInput, PlayerInput, RemoteInput},
    Color, Game, MessageToGame, MessageToRemote, Rect, Renderer, Sprite, SynchronizePlayer,
};

#[derive(Default)]
struct SpriteCounter(usize);

impl Renderer for SpriteCounter {
    fn clear(&mut self, _: Color) {}
    fn fill_rect(&mut self, _: Rect, _: Color) {}
    fn draw_line(&mut self, _: (i32, i32), _: (i32, i32), _: Color) {}
    fn draw_sprite(&mut self, _: Sprite, _: Rect) {
        self.0 += 1;
    }
    fn draw_text(&mut self, _: &str, _: i32, _: i32, _: Color) {}
}

#[test]
fn two_ready_players_start_the_game() {
    let mut game = Game::new("s  s\n", (1600, 900));
    for socket_id in 0..2 {
        game.handle(MessageToGame::SynchronizePlayer(SynchronizePlayer {
            socket_id,
            player_id: format!("player-{socket_id}"),
        }));
        game.handle(MessageToGame::PlayerInput(PlayerInput {
            socket_id,
            remote_input: RemoteInput::ConfigurationInput(ConfigurationInput::Ready),
        }));
    }

    // A long frame is capped; the 4 queued messages are all applied in its first step
    game.tick(1.0);

    let outbox = game.drain_outbox();
    assert_eq!(outbox.iter().map(|(socket_id, _)| *socket_id).collect::<Vec<_>>(), [0, 1]);
    assert!(outbox.iter().all(|(_, message)| matches!(message, MessageToRemote::GameStart)));
    assert!(game.drain_outbox().is_empty());

    let mut renderer = SpriteCounter::default();
    game.render(&mut renderer);
    assert_eq!(renderer.0, 2, "both tanks are drawn once the game has started");
}
