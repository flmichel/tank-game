use futures_channel::mpsc::unbounded;
use game::components::{Circle, Movement, Player, Position};
use game::game::{MessageToGame, RoomId};
use game::game_state::{Assets, Phase, State};
use game::remotes::PlayerInput;
use game::renderer::SystemData;
use game::systems::{HandleInputs, RetrievePlayerForInputs};
use game::{players_connector, renderer, room_code, server_communicator};
use players_connector::PlayersConnector;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, TextureCreator};
// "self" imports the "image" module itself as well as everything else we listed
use anyhow::Result;
use image::LoadTexture;
use room_code::RoomCode;
use sdl2::image::{self, InitFlag};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::{Window, WindowContext};
use server_communicator::ServerCommunicator;
use specs::{Builder, Dispatcher, DispatcherBuilder, World, WorldExt};
use std::time::Duration;
use tokio::spawn;

#[derive(Debug, Clone, Copy)]
struct Direction(f64);

/*fn render(
    canvas: &mut WindowCanvas,
    texture: &Texture,
    player: &Player,
    room_code: &RoomCode,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(173, 216, 230));
    canvas.clear();

    let squares = room_code.get_qr_code_squares(10);
    squares.iter().for_each(|(square, color)| {
        canvas.set_draw_color(*color);
        let mut square = square.clone();
        square.offset(10, 10);
        canvas.fill_rect(square).unwrap();
    });

    let (width, height) = canvas.output_size()?;

    // Treat the center of the screen as the (0, 0) coordinate
    let screen_position = player.position + Point::new(width as i32 / 2, height as i32 / 2);
    let screen_rect = Rect::from_center(
        screen_position,
        player.sprite.width(),
        player.sprite.height(),
    );

    canvas.copy(texture, player.sprite, screen_rect)?;

    canvas.present();

    Ok(())
}*/

/*
fn update_player(player: &mut Player) {
    let x = (player.speed * player.direction.0.cos()) as i32;
    let y = (player.speed * player.direction.0.sin()) as i32;
    player.position = player.position.offset(x, y);
}
*/

#[tokio::main]
async fn main() -> Result<(), String> {
    let (sender_to_server, receiver_server) = unbounded();
    let (sender_to_player_connector, receiver_player_connector) = unbounded();
    let (sender_to_game, mut receiver_game) = unbounded();

    let mut players_connector = PlayersConnector::new(
        sender_to_server.clone(),
        sender_to_game.clone(),
        receiver_player_connector,
    );
    spawn(async move { players_connector.start().await });

    let server_communicator = ServerCommunicator::new(
        sender_to_game,
        sender_to_player_connector,
        "ws://localhost:5000",
    );
    spawn(async move { server_communicator.start(receiver_server).await });

    let mut assets = load_assets();

    let mut world = create_world();
    let mut dispatcher = create_dispatcher();

    let mut event_pump = assets.sdl_context.event_pump()?;

    let ttf_context = sdl2::ttf::init().unwrap();
    let font = load_font(&ttf_context);

    let texture_creator: TextureCreator<WindowContext> = assets.canvas.texture_creator();

    let player_face = texture_creator
        .load_texture("assets/grin.png")
        .expect("Failed to load player face");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }

        if let Ok(Some(message)) = receiver_game.try_next() {
            match message {
                MessageToGame::RoomId(id) => {
                    let mut game_state = world.write_resource::<State>();
                    game_state.room_code = RoomCode::new(
                        format!("http://192.168.0.108:8080/?room-id={}", id.0).to_owned(),
                    );
                }
                MessageToGame::PlayerInput(player_input) => {
                    world.create_entity().with(player_input).build();
                }
            }
        }

        dispatcher.dispatch(&mut world);

        // Render
        renderer::render(
            &mut assets,
            SystemData::new(world.system_data()),
            &font,
            &player_face,
        )?;

        // Time management
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 20));
    }

    Ok(())
}

fn create_world() -> World {
    let mut world = World::new();
    world.register::<RoomId>();
    world.register::<PlayerInput>();
    world.register::<Position>();
    world.register::<Movement>();
    world.register::<Circle>();
    world.register::<Player>();

    let game_state = State {
        room_code: RoomCode::new("Error, the game could not connect to server".to_owned()),
        phase: Phase::BeforeNextGame,
        number_of_ready_players: 0,
    };
    world.insert(game_state);

    world
}

fn create_dispatcher() -> Dispatcher<'static, 'static> {
    let dispatcher = DispatcherBuilder::new()
        .with(RetrievePlayerForInputs, "RetrievePlayerForInputs", &[])
        .with(HandleInputs, "HandleInputs", &["RetrievePlayerForInputs"])
        .build();

    dispatcher
}
fn load_font(ttf_context: &Sdl2TtfContext) -> Font {
    // Load a font from a file
    let font_path = "assets/NotoSans-Medium.ttf";
    let font_size = 24;
    ttf_context.load_font(font_path, font_size).unwrap()
}

fn load_assets<'a>() -> Assets {
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
        .build()
        .expect("could not make a canvas");

    Assets {
        canvas,
        sdl_context,
    }
}
