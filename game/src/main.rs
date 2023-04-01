mod controllers;
mod game;
mod room_code;
mod server_communication;
mod signal;
mod server_connection;

use futures_channel::mpsc::{self, unbounded};
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
use server_communication::{connect_to_server, SdpMessage, ServerMessage};
use std::time::Duration;

const PLAYER_MOVEMENT_SPEED: i32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
struct Player {
    position: Point,
    sprite: Rect,
    speed: i32,
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
    use self::Direction::*;
    match player.direction {
        Left => {
            player.position = player.position.offset(-player.speed, 0);
        }
        Right => {
            player.position = player.position.offset(player.speed, 0);
        }
        Up => {
            player.position = player.position.offset(0, -player.speed);
        }
        Down => {
            player.position = player.position.offset(0, player.speed);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let (tx_server, mut rx_server) = unbounded();
    let (tx_game, mut rx_game) = unbounded();

    tokio::task::spawn(async move {
        connect_to_server(tx_server, &mut rx_game).await;
    });

    /*loop {
        println!("enter new sdp> ");
        let line = signal::must_read_stdin().expect("failed to read stdin");
        let sdp_offer = signal::decode(line.as_str()).expect("failed to decode");
        let sender = tx.clone();

        tokio::spawn(async move {
            start_peer_connection(sdp_offer, sender).await;
        });

        if let Some(answer) = rx.recv().await {
            println!(
                "\nmain thread got answer: {}",
                signal::encode(answer.as_str())
            );
        }
    }*/

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
        speed: 0,
        direction: Direction::Right,
    };

    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;

    let mut room_code = RoomCode::new("Hello World".to_owned());
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    repeat: false,
                    ..
                } => {
                    player.speed = PLAYER_MOVEMENT_SPEED;
                    player.direction = Direction::Left;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    repeat: false,
                    ..
                } => {
                    player.speed = PLAYER_MOVEMENT_SPEED;
                    player.direction = Direction::Right;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    repeat: false,
                    ..
                } => {
                    player.speed = PLAYER_MOVEMENT_SPEED;
                    player.direction = Direction::Up;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    repeat: false,
                    ..
                } => {
                    player.speed = PLAYER_MOVEMENT_SPEED;
                    player.direction = Direction::Down;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    repeat: false,
                    ..
                }
                | Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    repeat: false,
                    ..
                }
                | Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    repeat: false,
                    ..
                }
                | Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    repeat: false,
                    ..
                } => {
                    player.speed = 0;
                }
                _ => {}
            }
        }

        if let Ok(Some(message)) = rx_server.try_next() {
            match message {
                ServerMessage::RoomId(id) => { println!("creating qrcode with room_id {}", id);
                room_code =
                    RoomCode::new(format!("http://192.168.0.106:5500/?room-id={}", id).to_owned());
                },
                ServerMessage::SdpOffer(sdpMessage)
            }
        }
        if let Ok(Some(ServerMessage::RoomId(id))) = rx_server.try_next() {
            println!("creating qrcode with room_id {}", id);
            room_code =
                RoomCode::new(format!("http://192.168.0.106:5500/?room-id={}", id).to_owned());
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
