use game_core::{Color, Rect, Renderer, Sprite};
use sdl2::{
    pixels,
    rect,
    render::{Canvas, Texture},
    ttf::Font,
    video::Window,
};

/// SDL2 adapter for the core's drawing port.
pub struct SdlRenderer<'a> {
    pub canvas: &'a mut Canvas<Window>,
    pub font: &'a Font<'a, 'a>,
    pub player_face: &'a Texture<'a>,
    pub missile: &'a Texture<'a>,
}

fn sdl_color(color: Color) -> pixels::Color {
    pixels::Color::RGB(color.r, color.g, color.b)
}

fn sdl_rect(r: Rect) -> rect::Rect {
    rect::Rect::new(r.x, r.y, r.w, r.h)
}

impl Renderer for SdlRenderer<'_> {
    fn clear(&mut self, color: Color) {
        self.canvas.set_draw_color(sdl_color(color));
        self.canvas.clear();
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.canvas.set_draw_color(sdl_color(color));
        self.canvas.fill_rect(sdl_rect(rect)).unwrap();
    }

    fn draw_line(&mut self, from: (i32, i32), to: (i32, i32), color: Color) {
        self.canvas.set_draw_color(sdl_color(color));
        self.canvas.draw_line(from, to).unwrap();
    }

    fn draw_sprite(&mut self, sprite: Sprite, rect: Rect) {
        let texture = match sprite {
            Sprite::PlayerFace => self.player_face,
            Sprite::Missile => self.missile,
        };
        self.canvas.copy(texture, None, sdl_rect(rect)).unwrap();
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, color: Color) {
        // SDL_ttf fails on an empty string
        if text.is_empty() {
            return;
        }
        let surface = self.font.render(text).blended(sdl_color(color)).unwrap();
        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
        let size = texture.query();
        self.canvas
            .copy(&texture, None, rect::Rect::new(x, y, size.width, size.height))
            .unwrap();
    }
}
