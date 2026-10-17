use specs::{Join, World, WorldExt};

use crate::{
    components::*,
    state::{
        match_state::Match,
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
    /// Draws `text` with (x, y) as the top left of its box, 24 px tall in every shell.
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

const QR_SQUARE_SIZE: usize = 10;
const QR_MARGIN: i32 = 10;

fn render_before_game(world: &World, state: &State, renderer: &mut impl Renderer) {
    renderer.clear(Color::rgb(173, 216, 230));

    for (square, color) in state.room_code.get_qr_code_squares(QR_SQUARE_SIZE) {
        renderer.fill_rect(
            Rect::new(square.x + QR_MARGIN, square.y + QR_MARGIN, square.w, square.h),
            color,
        );
    }

    // The code is as wide as the room's URL makes it, and a phone cannot scan one drawn over
    let roster_left = 2 * QR_MARGIN + state.room_code.size(QR_SQUARE_SIZE) as i32;
    let players = world.read_storage::<Player>();
    let match_state = world.read_resource::<Match>();
    render_roster(
        &players.join().collect::<Vec<_>>(),
        &match_state,
        roster_left,
        state.resolution,
        renderer,
    );
}

// The lobby roster sits to the right of the QR code. A row is 60 px while that fits, then rows
// shrink and wrap into further columns, so a full game of players is still all on screen.
const ROSTER_TOP: i32 = 20;
const MAX_ROW_HEIGHT: i32 = 60;
// Both shells draw text 24 px tall and the roster cannot scale it, so a row never goes under this.
const MIN_ROW_HEIGHT: i32 = 36;
const TEXT_HEIGHT: i32 = 24;

fn render_roster(
    players: &[&Player],
    match_state: &Match,
    left: i32,
    resolution: (u32, u32),
    renderer: &mut impl Renderer,
) {
    if players.is_empty() {
        return;
    }

    let height = (resolution.1 as i32 - ROSTER_TOP).max(MIN_ROW_HEIGHT);
    let rows_that_fit = (height / MIN_ROW_HEIGHT).max(1) as usize;
    let columns = players.len().div_ceil(rows_that_fit);
    let rows = players.len().div_ceil(columns);
    let row_height = (height / rows as i32).min(MAX_ROW_HEIGHT);
    let column_width = (resolution.0 as i32 - left).max(0) / columns as i32;

    let icon = (row_height * 2 / 3) as u32;
    let gap = icon as i32 / 8;
    for (index, player) in players.iter().enumerate() {
        let x = left + (index / rows) as i32 * column_width;
        let y = ROSTER_TOP + (index % rows) as i32 * row_height;
        let status_color = match player.status {
            ReadyStatus::Ready => Color::GREEN,
            ReadyStatus::NotReady => Color::RED,
        };
        renderer.draw_sprite(Sprite::PlayerFace, Rect::new(x, y, icon, icon));
        renderer.fill_rect(Rect::new(x + icon as i32 + gap, y, icon, icon), status_color);
        if !player.name.is_empty() {
            // Between rounds the roster is also the buy screen: what each tank is worth
            let label = format!(
                "{} ${} w{}",
                player.name,
                match_state.wallet(&player.id),
                match_state.round_wins(&player.id)
            );
            let text_y = y + (icon as i32 - TEXT_HEIGHT).max(0) / 2;
            renderer.draw_text(&label, x + 2 * (icon as i32 + gap), text_y, Color::RED);
        }
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

    render_scoreboard(&players, world, renderer);

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

// The scoreboard sits in the top left corner of the round, over the map. Text only: the shells
// draw it 24 px tall and the tests count sprites.
const SCOREBOARD_LEFT: i32 = 10;
const SCOREBOARD_TOP: i32 = 10;
const SCOREBOARD_ROW_HEIGHT: i32 = TEXT_HEIGHT + 4;

/// Money and rounds won, so a player can see what the next buy phase affords.
fn render_scoreboard(
    players: &specs::ReadStorage<Player>,
    world: &World,
    renderer: &mut impl Renderer,
) {
    let match_state = world.read_resource::<Match>();

    for (row, player) in players.join().enumerate() {
        let line = format!(
            "{} ${} w{}",
            player.name,
            match_state.wallet(&player.id),
            match_state.round_wins(&player.id)
        );
        renderer.draw_text(
            &line,
            SCOREBOARD_LEFT,
            SCOREBOARD_TOP + row as i32 * SCOREBOARD_ROW_HEIGHT,
            Color::BLACK,
        );
    }
}
