use qrcode::QrCode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use tracing::info;

pub struct RoomCode {
    qr_code: QrCode,
}

impl RoomCode {
    pub fn new(room_id: String) -> Self {
        info!("URL from QRCode is \"{room_id}\".");
        RoomCode {
            qr_code: QrCode::new(room_id.as_bytes()).unwrap(),
        }
    }

    fn get_color(&self, x: usize, y: usize) -> Color {
        let colors = self.qr_code.to_colors();
        let color = colors.get(x * self.qr_code.width() + y).unwrap();

        match color {
            qrcode::Color::Dark => Color::BLACK,
            qrcode::Color::Light => Color::WHITE,
        }
    }

    pub fn get_qr_code_squares(&self, square_size: usize) -> Vec<(Rect, Color)> {
        let mut squares = Vec::new();
        for x in 0..self.qr_code.width() {
            for y in 0..self.qr_code.width() {
                squares.push(self.build_square(square_size, x, y))
            }
        }
        squares
    }

    fn build_square(&self, square_size: usize, x: usize, y: usize) -> (Rect, Color) {
        let rect = Rect::new(
            x as i32 * square_size as i32,
            y as i32 * square_size as i32,
            square_size as u32,
            square_size as u32,
        );

        let color = self.get_color(x, y);

        (rect, color)
    }
}
