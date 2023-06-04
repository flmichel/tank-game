use sdl2::{
    pixels::Color,
    rect::Point,
    render::{Texture, WindowCanvas},
};
use specs::{Join, ReadExpect, ReadStorage};

use crate::{components::*, game_state::State};

// Type alias for the data needed by the renderer
pub type SystemData<'a> = (
    ReadExpect<'a, State>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, Circle>,
    ReadStorage<'a, Player>,
);

pub fn render(canvas: &mut WindowCanvas, data: SystemData) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(173, 216, 230));
    canvas.clear();

    let squares = data.0.room_code.get_qr_code_squares(10);
    squares.iter().for_each(|(square, color)| {
        canvas.set_draw_color(*color);
        let mut square = square.clone();
        square.offset(10, 10);
        canvas.fill_rect(square).unwrap();
    });

    canvas.present();

    Ok(())
}
