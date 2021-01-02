/// Holds the pixel buffer and has methods for setting pixels, clearing the buffer and retrieving it.
#[derive(Debug, PartialEq)]
pub struct Screen {
    /// 64x32 display represented using 32 64-bit integers.
    pixel_buffer: [u64; 32],
    wrap_x: bool,
    wrap_y: bool,
}

impl Screen {
    pub fn new(wrap_x: bool, wrap_y: bool) -> Self {
        Screen {
            pixel_buffer: [0; 32],
            wrap_x,
            wrap_y,
        }
    }

    /// Convert pixel_buffer to a Vec<u8>.
    pub fn get_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        for row in 0..32 {
            for col in (0..64).rev() {
                let rgba = if (self.pixel_buffer[row] >> col) & 1 == 1 {
                    [0xFF, 0xFF, 0xFF, 0xFF]
                } else {
                    [0x00, 0x00, 0x00, 0x00]
                };
                buffer.extend(&rgba);
            }
        }
        buffer
    }

    /// Set every bit (pixel) in the buffer to be 0.
    pub fn clear(&mut self) {
        self.pixel_buffer = [0; 32];
    }

    /// Draw sprite at given position
    pub fn draw_sprite(&mut self, sprite: &[u8], row: usize, col: usize) {
        let shift_amount = 63i32 - col as i32 - 7i32;
        for (i, byte) in sprite.iter().enumerate() {
            let byte = self.shift_byte(*byte, shift_amount as i32);
            self.draw_byte(row + i, byte);
        }
    }

    /// Cast a byte to a u64 and shift bits given amount. Wrap if flag is set.
    fn shift_byte(&self, byte: u8, shift_amount: i32) -> u64 {
        let byte = byte as u64;

        if shift_amount >= 0 {
            byte << shift_amount
        } else if self.wrap_x {
            byte.rotate_right(shift_amount.abs() as u32) // Shifts right and wraps bits back to front of num.
        } else {
            byte.wrapping_shr(shift_amount.abs() as u32) // Shifts right. Ignores bits that overflow. Weird name tbh.
        }
    }

    /// Draw a byte (cast to a u64) to the pixel buffer and wrap vertically if flag is set.
    fn draw_byte(&mut self, row: usize, byte: u64) {
        if row < 32 {
            self.pixel_buffer[row] ^= byte;
        } else if self.wrap_y {
            self.pixel_buffer[row % 32] ^= byte;
        }
    }

    /// Set the value of a pixel using a row and column.
    pub fn set_pixel(&mut self, row: usize, col: usize, status: bool) {
        let col = 63 - col;

        if status {
            self.pixel_buffer[row] |= 1 << col;
        } else {
            self.pixel_buffer[row] &= !(1 << col);
        }
    }

    // Get the status of a pixel using a row and column.
    pub fn get_pixel(&mut self, row: usize, col: usize) -> bool {
        self.check_bounds(row, col);

        let col = 63 - col;
        (self.pixel_buffer[row] >> col & 1) == 1
    }

    // Check if a given index is out of bounds.
    fn check_bounds(&self, row: usize, col: usize) {
        if row >= 32 || col > 64 {
            panic!("out of bounds for pixel buffer: ({}, {})", col, row);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creating_new_screen() {
        let screen = Screen::new(true, true);
        assert_eq!(screen.pixel_buffer.len(), 32);
        assert_eq!(screen.pixel_buffer[0], 0);
        assert_eq!(screen.pixel_buffer[16], 0);
        assert_eq!(screen.pixel_buffer[31], 0);
    }

    #[test]
    fn test_clearing_screen() {
        let mut screen = Screen::new(true, true);
        screen.pixel_buffer[0] = 1;
        screen.pixel_buffer[16] = 1;
        screen.pixel_buffer[31] = 1;

        screen.clear();
        assert_eq!(screen.pixel_buffer, [0; 32]);
    }

    #[test]
    fn test_setting_pixel() {
        let mut screen = Screen::new(true, true);

        screen.set_pixel(0, 63, true);
        assert_eq!(screen.pixel_buffer[0], 1);

        screen.set_pixel(0, 63, false);
        assert_eq!(screen.pixel_buffer[0], 0);

        screen.set_pixel(31, 63, true);
        assert_eq!(screen.pixel_buffer[31], 1);

        screen.set_pixel(31, 63, false);
        assert_eq!(screen.pixel_buffer[31], 0);
    }

    #[test]
    fn test_getting_pixel() {
        let mut screen = Screen::new(true, true);

        assert_eq!(screen.get_pixel(0, 0), false);
        assert_eq!(screen.get_pixel(31, 63), false);
        assert_eq!(screen.get_pixel(16, 32), false);

        screen.set_pixel(0, 31, true);
        assert_eq!(screen.get_pixel(0, 31), true);

        screen.set_pixel(0, 0, true);
        assert_eq!(screen.get_pixel(0, 0), true);

        screen.set_pixel(16, 32, true);
        assert_eq!(screen.get_pixel(16, 32), true);
    }

    #[test]
    fn test_drawing_sprite_to_empty_buffer() {
        let mut screen = Screen::new(true, true);

        let sprite = vec![255, 255, 255];
        screen.draw_sprite(&sprite, 15, 0);

        assert_eq!(screen.get_pixel(15, 0), true);
        assert_eq!(screen.get_pixel(16, 0), true);
        assert_eq!(screen.get_pixel(17, 0), true);

        assert_eq!(screen.get_pixel(15, 7), true);
        assert_eq!(screen.get_pixel(16, 7), true);
        assert_eq!(screen.get_pixel(17, 7), true);

        assert_eq!(screen.get_pixel(15, 3), true);
        assert_eq!(screen.get_pixel(16, 4), true);
        assert_eq!(screen.get_pixel(17, 5), true);

        assert_eq!(screen.get_pixel(15, 8), false);
        assert_eq!(screen.get_pixel(16, 8), false);
        assert_eq!(screen.get_pixel(17, 8), false);
    }

    #[test]
    fn test_drawing_sprite_to_top_left() {
        let mut screen = Screen::new(true, true);

        let sprite = vec![255, 255, 255];
        screen.draw_sprite(&sprite, 0, 0);

        assert_eq!(screen.get_pixel(0, 0), true);
        assert_eq!(screen.get_pixel(1, 4), true);
        assert_eq!(screen.get_pixel(2, 7), true);
    }

    #[test]
    fn test_drawing_sprite_to_bottom_right() {
        let mut screen = Screen::new(true, true);

        let sprite = vec![255, 255, 255];
        screen.draw_sprite(&sprite, 29, 56);

        assert_eq!(screen.get_pixel(29, 56), true);
        assert_eq!(screen.get_pixel(30, 60), true);
        assert_eq!(screen.get_pixel(31, 63), true);
    }

    #[test]
    fn test_vertical_wrapping() {
        let mut screen = Screen::new(true, true);

        let sprite = vec![255, 255, 255];
        screen.draw_sprite(&sprite, 31, 0);

        assert_eq!(screen.get_pixel(31, 0), true);
        assert_eq!(screen.get_pixel(0, 3), true);
        assert_eq!(screen.get_pixel(1, 7), true);
    }

    #[test]
    fn test_horizontal_wrapping() {
        let mut screen = Screen::new(true, true);

        let sprite = vec![255, 255, 255];
        screen.draw_sprite(&sprite, 15, 60);

        assert_eq!(screen.get_pixel(15, 60), true);
        assert_eq!(screen.get_pixel(16, 0), true);
        assert_eq!(screen.get_pixel(17, 2), true);
    }

    #[test]
    fn test_no_wrapping_vertically() {
        let mut screen = Screen::new(true, false);

        let sprite = vec![255, 255, 255];
        screen.draw_sprite(&sprite, 31, 60);

        assert_eq!(screen.get_pixel(31, 60), true);
        assert_eq!(screen.get_pixel(31, 0), true);
        assert_eq!(screen.get_pixel(31, 2), true);

        assert_eq!(screen.get_pixel(0, 60), false);
        assert_eq!(screen.get_pixel(0, 0), false);
        assert_eq!(screen.get_pixel(0, 2), false);
    }

    #[test]
    fn test_no_wrapping_horizontally() {
        let mut screen = Screen::new(false, true);

        let sprite = vec![255, 255, 255];
        screen.draw_sprite(&sprite, 31, 60);

        assert_eq!(screen.get_pixel(31, 60), true);
        assert_eq!(screen.get_pixel(31, 0), false);
        assert_eq!(screen.get_pixel(31, 2), false);

        assert_eq!(screen.get_pixel(0, 60), true);
        assert_eq!(screen.get_pixel(0, 0), false);
        assert_eq!(screen.get_pixel(0, 2), false);
    }
}
