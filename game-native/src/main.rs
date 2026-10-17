use futures_channel::mpsc::unbounded;
use game_core::{bot::BotKind, Game, MessageToGame};
use game_native::{
    players_connector::PlayersConnector, renderer::SdlRenderer,
    server_communicator::ServerCommunicator, startup::setup_logger,
};
use sdl2::event::Event;
// "self" imports the "image" module itself as well as everything else we listed
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Instant;
use tokio::spawn;
use tracing::{error, warn};

struct Assets {
    canvas: Canvas<Window>,
    sdl_context: Sdl,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && !args[1].starts_with("--") {
        setup_logger(&args[1]);
    } else {
        setup_logger("info");
    }

    let bots = parse_bots(&args);

    let (sender_to_server, receiver_server) = unbounded();
    let (sender_to_player_connector, receiver_player_connector) = unbounded();
    let (sender_to_game, mut receiver_game) = unbounded();
    let (sender_data_channel, mut receiver_data_channel) = unbounded();

    let mut players_connector = PlayersConnector::new(
        sender_to_server.clone(),
        sender_to_game.clone(),
        receiver_player_connector,
        sender_data_channel,
    );
    spawn(async move { players_connector.start().await });

    let server_communicator = ServerCommunicator::new(
        sender_to_game,
        sender_to_player_connector,
        //"ws://tank-game.flmichel.duckdns.org/ws",
        "ws://0.0.0.0:5000/ws",
    );
    spawn(async move { server_communicator.start(receiver_server).await });

    let mut assets = load_assets();

    let map = std::fs::read_to_string("assets/map.txt").expect("Failed to load map");
    let mut game = Game::new(&map, assets.canvas.window().size());

    // Bots need no remote: with one the game is playable, and testable, alone
    for _ in 0..bots.0 {
        game.handle(MessageToGame::AddBot(bots.1));
    }

    let mut event_pump = assets.sdl_context.event_pump()?;

    let ttf_context = sdl2::ttf::init().unwrap();
    let font = load_font(&ttf_context);

    let texture_creator: TextureCreator<WindowContext> = assets.canvas.texture_creator();

    let player_face = texture_creator
        .load_texture("assets/grin.png")
        .expect("Failed to load player face");

    let missile = texture_creator
        .load_texture("assets/missile.png")
        .expect("Failed to load missile");

    let mut renderer = SdlRenderer {
        canvas: &mut assets.canvas,
        font: &font,
        player_face: &player_face,
        missile: &missile,
    };
    let mut data_channels = HashMap::new();
    let mut last_frame = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }

        while let Ok(Some((socket_id, data_channel))) = receiver_data_channel.try_next() {
            data_channels.insert(socket_id, data_channel);
        }
        while let Ok(Some(message)) = receiver_game.try_next() {
            game.handle(message);
        }

        let now = Instant::now();
        game.tick((now - last_frame).as_secs_f64());
        last_frame = now;

        game.render(&mut renderer);

        for (socket_id, message) in game.drain_outbox() {
            let Some(data_channel) = data_channels.get(&socket_id).map(Arc::clone) else {
                warn!("No data channel for socket id \"{socket_id}\".");
                continue;
            };
            let message = serde_json::to_string(&message).unwrap();
            spawn(async move {
                if let Err(err) = data_channel.send_text(message).await {
                    error!("error sending a message to a remote: {err}");
                }
            });
        }

        // Blocks until the next vsync, which paces the loop at the display's refresh rate.
        // ponytail: the driver may queue 2-3 frames; limiting that needs Metal/OpenGL directly
        renderer.canvas.present();
    }

    Ok(())
}

/// `--bots <count>[:<kind>]`, kind being dummy (the default), wanderer, or hunter.
fn parse_bots(args: &[String]) -> (u32, BotKind) {
    let Some(value) = args.iter().position(|arg| arg == "--bots").and_then(|at| args.get(at + 1))
    else {
        return (0, BotKind::Dummy);
    };

    let (count, kind) = value.split_once(':').unwrap_or((value, "dummy"));
    let count = count.parse().unwrap_or_else(|_| {
        warn!("--bots {value}: not a bot count, starting without bots");
        0
    });
    let kind = BotKind::parse(kind).unwrap_or_else(|| {
        warn!("--bots {value}: unknown bot kind \"{kind}\", using dummy");
        BotKind::Dummy
    });

    (count, kind)
}

fn load_font(ttf_context: &Sdl2TtfContext) -> Font {
    // Load a font from a file
    let font_path = "assets/NotoSans-Medium.ttf";
    let font_size = 24;
    ttf_context.load_font(font_path, font_size).unwrap()
}

fn load_assets() -> Assets {
    let sdl_context = sdl2::init().expect("failed to create context");
    let video_subsystem = sdl_context
        .video()
        .expect("failed to create video subsystem");

    // Leading "_" tells Rust that this is an unused variable that we don't care about. It has to
    // stay unused because if we don't have any variable at all then Rust will treat it as a
    // temporary value and drop it right away!
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG).expect("failed to get image");

    let window = video_subsystem
        .window("tank game", 800, 600)
        .fullscreen_desktop() // Set fullscreen mode
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let canvas: Canvas<Window> = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("could not make a canvas");

    Assets {
        canvas,
        sdl_context,
    }
}
