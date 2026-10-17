use qrcode::QrCode;
use tracing::info;

use crate::render::{Color, Rect};

pub struct RoomCode {
    width: usize,
    // The modules, row by row. Rendering the code once keeps it out of every frame's work.
    modules: Vec<Color>,
}

impl RoomCode {
    pub fn new(room_id: String) -> Self {
        info!("URL from QRCode is \"{room_id}\".");
        let qr_code = QrCode::new(room_id.as_bytes()).unwrap();
        let modules = qr_code
            .to_colors()
            .into_iter()
            .map(|color| match color {
                qrcode::Color::Dark => Color::BLACK,
                qrcode::Color::Light => Color::WHITE,
            })
            .collect();

        RoomCode {
            width: qr_code.width(),
            modules,
        }
    }

    /// How wide the code is on screen at this square size, so nothing is drawn over it.
    pub fn size(&self, square_size: usize) -> u32 {
        (self.width * square_size) as u32
    }

    pub fn get_qr_code_squares(&self, square_size: usize) -> Vec<(Rect, Color)> {
        let mut squares = Vec::with_capacity(self.modules.len());
        for y in 0..self.width {
            for x in 0..self.width {
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

        // The modules are row major: reading them column major drew the code transposed, and a
        // transposed code does not scan.
        (rect, self.modules[y * self.width + x])
    }
}
