use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::{Canvas, Texture},
    ttf::Font,
    video::Window,
};
use specs::{Join, ReadExpect, ReadStorage};

use crate::{
    components::*,
    state::{
        game_state::{Assets, Phase, State},
        BlockKind, Map,
    },
};

type SystemDataType<'a> = (
    ReadExpect<'a, State>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, Circle>,
    ReadStorage<'a, Player>,
);

pub struct SystemData<'a> {
    system_data: SystemDataType<'a>,
}

impl<'a> SystemData<'a> {
    pub fn new(system_data: SystemDataType<'a>) -> Self {
        SystemData { system_data }
    }
    fn get_state(&self) -> &ReadExpect<'a, State> {
        &self.system_data.0
    }

    fn get_position(&self) -> &ReadStorage<'a, Position> {
        &self.system_data.1
    }

    fn get_players(&self) -> &ReadStorage<'a, Player> {
        &self.system_data.3
    }
}

pub fn render(
    assets: &mut Assets,
    data: SystemData,
    font: &Font,
    player_face: &Texture,
) -> Result<(), String> {
    match data.get_state().phase {
        Phase::BeforeNextGame => {
            render_before_game(assets, data, font, player_face);
        }
        Phase::InGame => render_game(assets, data, font, player_face),
        Phase::BreakInGame => {}
    }

    Ok(())
}

fn render_before_game(assets: &mut Assets, data: SystemData, font: &Font, player_face: &Texture) {
    let canvas = &mut assets.canvas;

    canvas.set_draw_color(Color::RGB(173, 216, 230));
    canvas.clear();

    let squares = data.get_state().room_code.get_qr_code_squares(10);
    squares.iter().for_each(|(square, color)| {
        canvas.set_draw_color(*color);
        let mut square = square.clone();
        square.offset(10, 10);
        canvas.fill_rect(square).unwrap();
    });

    let mut y = 200; // Calculate starting Y position

    for player in data.get_players().join() {
        let circle_color = match player.status {
            ReadyStatus::Ready => Color::GREEN,
            ReadyStatus::NotReady => Color::RED,
        };

        // Calculate the rectangle position and size
        let rect = Rect::new(500, y, 40, 40);

        // Draw the filled rectangle
        canvas.set_draw_color(circle_color);
        canvas.fill_rect(rect).unwrap();

        // Render player face next to the circle
        let face_dest_rect = Rect::new(460, y, 40, 40);
        canvas.copy(player_face, None, face_dest_rect).unwrap();

        // Render player name next to the circle
        let texture_creator = canvas.texture_creator();

        let surface = font
            .render(&player.name)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string())
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();
        let font_rect = texture.query();
        let name_pos = Point::new(560, y + 10);

        canvas
            .copy(
                &texture,
                None,
                Rect::new(name_pos.x, name_pos.y, font_rect.width, font_rect.height),
            )
            .unwrap();

        // Increment the Y position for the next player
        y += 60;
    }

    canvas.present();
}

fn render_game(assets: &mut Assets, data: SystemData, font: &Font, player_face: &Texture) {
    let canvas = &mut assets.canvas;

    canvas.set_draw_color(Color::RGB(173, 216, 150));
    canvas.clear();

    render_map(&data.get_state().map, canvas);

    for (player, position) in (data.get_players(), data.get_position()).join() {
        // Render player face next to the circle
        let face_dest_rect = Rect::new(position.x as i32, position.y as i32, 50, 50);
        canvas.copy(player_face, None, face_dest_rect).unwrap();

        // Render player name next to the circle
        let texture_creator = canvas.texture_creator();

        let surface = font
            .render(&player.name)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string())
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();
        let font_rect = texture.query();

        canvas
            .copy(
                &texture,
                None,
                Rect::new(
                    position.x as i32,
                    position.y as i32,
                    font_rect.width,
                    font_rect.height,
                ),
            )
            .unwrap();
    }

    canvas.present();
}

fn render_map(map: &Map, canvas: &mut Canvas<Window>) {
    let (window_width, window_height) = canvas.window().size();

    // Calculate the size of each block based on the window dimensions and the block matrix size
    let block_width = window_width / 16 as u32;
    let block_height = window_height / 9 as u32;

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    for (row_index, row) in map.block_matrix.iter().enumerate() {
        for (col_index, block_kind) in row.iter().enumerate() {
            let block_x = col_index as i32 * block_width as i32;
            let block_y = row_index as i32 * block_height as i32;

            match block_kind {
                BlockKind::Wall => {
                    // Draw a wall block
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                    canvas
                        .fill_rect(Rect::new(block_x, block_y, block_width, block_height))
                        .unwrap();
                }
                BlockKind::Ground => {
                    // Draw a ground block
                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                    canvas
                        .fill_rect(Rect::new(block_x, block_y, block_width, block_height))
                        .unwrap();
                }
            }
        }
    }
}
