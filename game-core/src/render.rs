use specs::{Join, World, WorldExt};

use crate::{
    components::*,
    state::{
        game_state::{Phase, State},
        BlockKind, Map,
    },
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const RED: Color = Color::rgb(255, 0, 0);
    pub const GREEN: Color = Color::rgb(0, 255, 0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Rect { x, y, w, h }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Sprite {
    PlayerFace,
    Missile,
}

/// Drawing port: the core decides what to draw, each shell implements how.
pub trait Renderer {
    fn clear(&mut self, color: Color);
    fn fill_rect(&mut self, rect: Rect, color: Color);
    fn draw_line(&mut self, from: (i32, i32), to: (i32, i32), color: Color);
    fn draw_sprite(&mut self, sprite: Sprite, rect: Rect);
    fn draw_text(&mut self, text: &str, x: i32, y: i32, color: Color);
}

const AIMING_LINE_LENGTH: f64 = 2.;

pub fn render(world: &World, renderer: &mut impl Renderer) {
    let state = world.read_resource::<State>();
    match state.phase {
        Phase::BeforeNextGame => render_before_game(world, &state, renderer),
        Phase::InGame => render_game(world, &state, renderer),
        Phase::BreakInGame => {}
    }
}

fn render_before_game(world: &World, state: &State, renderer: &mut impl Renderer) {
    renderer.clear(Color::rgb(173, 216, 230));

    for (square, color) in state.room_code.get_qr_code_squares(10) {
        renderer.fill_rect(Rect::new(square.x + 10, square.y + 10, square.w, square.h), color);
    }

    let mut y = 200;
    for player in world.read_storage::<Player>().join() {
        let status_color = match player.status {
            ReadyStatus::Ready => Color::GREEN,
            ReadyStatus::NotReady => Color::RED,
        };
        renderer.fill_rect(Rect::new(500, y, 40, 40), status_color);
        renderer.draw_sprite(Sprite::PlayerFace, Rect::new(460, y, 40, 40));
        if !player.name.is_empty() {
            renderer.draw_text(&player.name, 560, y + 10, Color::RED);
        }
        y += 60;
    }
}

fn render_game(world: &World, state: &State, renderer: &mut impl Renderer) {
    renderer.clear(Color::rgb(173, 216, 150));
    render_map(&state.map, renderer);

    let block_size = state.map.block_size() as f64;
    let entities = world.entities();
    let positions = world.read_storage::<Position>();
    let circles = world.read_storage::<Circle>();
    let players = world.read_storage::<Player>();

    // Circles are either players or bullets
    for (entity, position, circle) in (&entities, &positions, &circles).join() {
        let rect = Rect::new(
            ((position.x - circle.get_radius()) * block_size) as i32,
            ((position.y - circle.get_radius()) * block_size) as i32,
            (circle.get_size() * block_size) as u32,
            (circle.get_size() * block_size) as u32,
        );

        match players.get(entity) {
            None => renderer.draw_sprite(Sprite::Missile, rect),
            Some(player) if player.is_alive => {
                renderer.draw_sprite(Sprite::PlayerFace, rect);
                renderer.draw_text(
                    &player.name,
                    (position.x * block_size) as i32,
                    (position.y * block_size) as i32,
                    Color::RED,
                );
                render_aiming_line(renderer, position, &player.aim, block_size);
            }
            Some(_) => {}
        }
    }
}

fn render_map(map: &Map, renderer: &mut impl Renderer) {
    let block_size = map.block_size();
    renderer.clear(Color::WHITE);

    for (row_index, row) in map.block_matrix.iter().enumerate() {
        for (col_index, block_kind) in row.iter().enumerate() {
            let color = match block_kind {
                BlockKind::Wall => Color::BLACK,
                BlockKind::Ground => Color::WHITE,
            };
            let block = Rect::new(
                col_index as i32 * block_size as i32,
                row_index as i32 * block_size as i32,
                block_size,
                block_size,
            );
            renderer.fill_rect(block, color);
        }
    }
}

fn render_aiming_line(renderer: &mut impl Renderer, position: &Position, aim: &AimStatus, block_size: f64) {
    if let AimStatus::Aim(direction) = aim {
        let start = ((position.x * block_size) as i32, (position.y * block_size) as i32);
        let end = (
            ((position.x + direction.cos() * AIMING_LINE_LENGTH) * block_size) as i32,
            ((position.y + direction.sin() * AIMING_LINE_LENGTH) * block_size) as i32,
        );
        renderer.draw_line(start, end, Color::GREEN);
    }
}
