use std::mem;

use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::{Texture, WindowCanvas},
    ttf::Font,
};
use specs::{Join, ReadExpect, ReadStorage};

use crate::{
    components::*,
    game_state::{Assets, State},
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
    fn get_state(&mut self) -> &ReadExpect<'a, State> {
        &self.system_data.0
    }

    fn get_players(&mut self) -> &ReadStorage<'a, Player> {
        &self.system_data.3
    }
}

pub fn render(
    assets: &mut Assets,
    mut data: SystemData,
    font: &Font,
    player_face: &Texture,
) -> Result<(), String> {
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
        canvas.fill_rect(rect)?;

        // Render player face next to the circle
        let face_dest_rect = Rect::new(460, y, 40, 40);
        canvas.copy(player_face, None, face_dest_rect)?;

        // Render player name next to the circle
        let texture_creator = canvas.texture_creator();

        let surface = font
            .render(&player.name)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string())?;
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        let font_rect = texture.query();
        let name_pos = Point::new(560, y + 10);

        canvas.copy(
            &texture,
            None,
            Rect::new(name_pos.x, name_pos.y, font_rect.width, font_rect.height),
        )?;

        // Increment the Y position for the next player
        y += 60;
    }

    canvas.present();

    Ok(())
}
