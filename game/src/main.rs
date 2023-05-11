mod controllers;
mod game;
mod players_connector;
mod remote_communicator;
mod room_code;
mod server_communicator;
mod signal;

use futures_channel::mpsc::unbounded;
use game::{Input, MessageToGame};
use players_connector::PlayersConnector;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
// "self" imports the "image" module itself as well as everything else we listed
use anyhow::Result;
use image::LoadTexture;
use room_code::RoomCode;
use sdl2::image::{self, InitFlag};
use server_communicator::{MessageToServer, SdpMessage, ServerCommunicator};
use std::time::Duration;
use tokio::spawn;

const PLAYER_MOVEMENT_SPEED: f64 = 5.0;

#[derive(Debug, Clone, Copy)]
struct Direction(f64);

#[derive(Debug)]
struct Player {
    position: Point,
    sprite: Rect,
    speed: f64,
    direction: Direction,
}

fn render(
    canvas: &mut WindowCanvas,
    color: Color,
    texture: &Texture,
    player: &Player,
    room_code: &RoomCode,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.clear();

    let squares = room_code.get_qr_code_squares(10);
    squares.iter().for_each(|(square, color)| {
        canvas.set_draw_color(*color);
        canvas.fill_rect(*square).unwrap();
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
}

// Update player a fixed amount based on their speed.
// WARNING: Calling this function too often or at a variable speed will cause the player's speed
// to be unpredictable!
fn update_player(player: &mut Player) {
    let x = (player.speed * player.direction.0.cos()) as i32;
    let y = (player.speed * player.direction.0.sin()) as i32;
    player.position = player.position.offset(x, y);
}

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
        //receiver_server,
        "ws://localhost:5000",
    );
    spawn(async move { server_communicator.start(receiver_server).await });

    let answer = MessageToServer::SdpAnswer(SdpMessage {
        data: "hello".to_owned(),
        id: 23,
    });
    //sender_to_server.unbounded_send(answer).unwrap();

    let sdl_context = sdl2::init().expect("failed to create context");
    let video_subsystem = sdl_context.video()?;
    // Leading "_" tells Rust that this is an unused variable that we don't care about. It has to
    // stay unused because if we don't have any variable at all then Rust will treat it as a
    // temporary value and drop it right away!
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG).expect("failed to get image");

    let window = video_subsystem
        .window("game tutorial", 800, 600)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .load_texture("assets/bardo.png")
        .expect("failed to load texture");

    let mut player = Player {
        position: Point::new(0, 0),
        sprite: Rect::new(0, 0, 26, 36),
        speed: 0.0,
        direction: Direction(0.0),
    };

    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;

    let mut room_code = RoomCode::new("Hello World".to_owned());
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
                    println!("creating qrcode with room_id {}", id);
                    room_code = RoomCode::new(
                        format!("http://192.168.0.103:8080/?room-id={}", id).to_owned(),
                    );
                }
                MessageToGame::Input(_) => {}
                MessageToGame::PlayerDirection(direction) => {
                    player.speed = PLAYER_MOVEMENT_SPEED;
                    player.direction = Direction(direction);
                }
            }
        }

        // Update
        i = (i + 1) % 255;
        update_player(&mut player);

        // Render
        render(
            &mut canvas,
            Color::RGB(i, 64, 255 - i),
            &texture,
            &player,
            &room_code,
        )?;

        // Time management!
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 20));
    }

    Ok(())
}
