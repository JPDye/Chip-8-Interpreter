use sdl2::{self, pixels, rect::Rect, render::Canvas, video::Window};

const SCALE_FACTOR: u32 = 10;
const SCREEN_WIDTH: u32 = 64 * SCALE_FACTOR;
const SCREEN_HEIGHT: u32 = 32 * SCALE_FACTOR;

pub struct DisplayDriver {
    canvas: Canvas<Window>,
}

impl DisplayDriver {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Chip8 in Rust", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Self { canvas }
    }

    pub fn draw(&mut self, pixels: Vec<u64>) {
        for (y, row) in pixels.iter().enumerate() {
            for (x, col) in (0..64).rev().enumerate() {
                let pixel = (row >> col) & 1;

                let rgb = if pixel == 0 {
                    pixels::Color::RGB(0, 0, 0)
                } else {
                    pixels::Color::RGB(0, 250, 0)
                };

                let x = x as u32 * SCALE_FACTOR;
                let y = y as u32 * SCALE_FACTOR;

                let rect = Rect::new(x as i32, y as i32, SCALE_FACTOR, SCALE_FACTOR);

                self.canvas.set_draw_color(rgb);
                let _ = self.canvas.fill_rect(rect);
            }
        }
        self.canvas.present();
    }
}
