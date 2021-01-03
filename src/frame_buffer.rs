/// Holds the pixel buffer and has methods for setting pixels, clearing the buffer and retrieving it.
#[derive(Debug, PartialEq)]
pub struct FrameBuffer {
    buffer: [u64; 32], // 64x32 display represented using 32 64-bit integers.
    prev_buffer: [u64; 32],
    wrap_x: bool,
    wrap_y: bool,
}

impl FrameBuffer {
    pub fn new(wrap_x: bool, wrap_y: bool) -> Self {
        FrameBuffer {
            buffer: [0; 32],
            prev_buffer: [0; 32],
            wrap_x,
            wrap_y,
        }
    }

    pub fn get_buffer(&mut self) -> Vec<u64> {
        let mut buf = Vec::new();

        for i in 0..32 {
            buf.push(self.prev_buffer[i] | self.buffer[i]);
        }

        self.prev_buffer = self.buffer.clone();
        buf
    }

    /// Set every bit (pixel) in the buffer to be 0.
    pub fn clear(&mut self) {
        self.buffer = [0; 32];
    }

    /// Draw sprite at given position
    pub fn draw_sprite(&mut self, sprite: &[u8], row: usize, col: usize) -> bool {
        let mut change = false;
        let shift_amount = 63i32 - col as i32 - 7i32;
        for (i, byte) in sprite.iter().enumerate() {
            let byte = self.shift_byte(*byte, shift_amount as i32);
            if self.draw_byte(row + i, byte) {
                change = true;
            }
        }
        change
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
    fn draw_byte(&mut self, row: usize, byte: u64) -> bool {
        if row < 32 {
            self.buffer[row] ^= byte;
            byte & self.buffer[row] != byte
        } else if self.wrap_y {
            self.buffer[row % 32] ^= byte;
            byte & self.buffer[row % 32] != byte
        } else{
            false
        }
    }

    /// Set the value of a pixel using a row and column.
    pub fn set_pixel(&mut self, row: usize, col: usize, status: bool) {
        let col = 63 - col;

        if status {
            self.buffer[row] |= 1 << col;
        } else {
            self.buffer[row] &= !(1 << col);
        }
    }

    // Get the status of a pixel using a row and column.
    pub fn get_pixel(&mut self, row: usize, col: usize) -> bool {
        self.check_bounds(row, col);

        let col = 63 - col;
        (self.buffer[row] >> col & 1) == 1
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
    fn test_creating_new_frame_buffer() {
        let frame_buffer = FrameBuffer::new(true, true);
        assert_eq!(frame_buffer.buffer.len(), 32);
        assert_eq!(frame_buffer.buffer[0], 0);
        assert_eq!(frame_buffer.buffer[16], 0);
        assert_eq!(frame_buffer.buffer[31], 0);
    }

    #[test]
    fn test_clearing_screen() {
        let mut screen = FrameBuffer::new(true, true);
        screen.buffer[0] = 1;
        screen.buffer[16] = 1;
        screen.buffer[31] = 1;

        screen.clear();
        assert_eq!(screen.buffer, [0; 32]);
    }

    #[test]
    fn test_setting_pixel() {
        let mut screen = FrameBuffer::new(true, true);

        screen.set_pixel(0, 63, true);
        assert_eq!(screen.buffer[0], 1);

        screen.set_pixel(0, 63, false);
        assert_eq!(screen.buffer[0], 0);

        screen.set_pixel(31, 63, true);
        assert_eq!(screen.buffer[31], 1);

        screen.set_pixel(31, 63, false);
        assert_eq!(screen.buffer[31], 0);
    }

    #[test]
    fn test_getting_pixel() {
        let mut screen = FrameBuffer::new(true, true);

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
        let mut screen = FrameBuffer::new(true, true);

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
        let mut screen = FrameBuffer::new(true, true);

        let sprite = vec![255, 255, 255];
        screen.draw_sprite(&sprite, 0, 0);

        assert_eq!(screen.get_pixel(0, 0), true);
        assert_eq!(screen.get_pixel(1, 4), true);
        assert_eq!(screen.get_pixel(2, 7), true);
    }

    #[test]
    fn test_drawing_sprite_to_bottom_right() {
        let mut screen = FrameBuffer::new(true, true);

        let sprite = vec![255, 255, 255];
        screen.draw_sprite(&sprite, 29, 56);

        assert_eq!(screen.get_pixel(29, 56), true);
        assert_eq!(screen.get_pixel(30, 60), true);
        assert_eq!(screen.get_pixel(31, 63), true);
    }

    #[test]
    fn test_vertical_wrapping() {
        let mut screen = FrameBuffer::new(true, true);

        let sprite = vec![255, 255, 255];
        screen.draw_sprite(&sprite, 31, 0);

        assert_eq!(screen.get_pixel(31, 0), true);
        assert_eq!(screen.get_pixel(0, 3), true);
        assert_eq!(screen.get_pixel(1, 7), true);
    }

    #[test]
    fn test_horizontal_wrapping() {
        let mut screen = FrameBuffer::new(true, true);

        let sprite = vec![255, 255, 255];
        screen.draw_sprite(&sprite, 15, 60);

        assert_eq!(screen.get_pixel(15, 60), true);
        assert_eq!(screen.get_pixel(16, 0), true);
        assert_eq!(screen.get_pixel(17, 2), true);
    }

    #[test]
    fn test_no_wrapping_vertically() {
        let mut screen = FrameBuffer::new(true, false);

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
        let mut screen = FrameBuffer::new(false, true);

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
