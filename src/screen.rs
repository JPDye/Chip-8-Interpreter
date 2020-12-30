/// Holds the pixel buffer and has methods for setting pixels, clearing the buffer and retrieving it.
#[derive(Debug, PartialEq)]
pub struct Screen {
    /// 64x32 display represented using 32 64-bit integers.
    pixel_buffer: [u64; 32],
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            pixel_buffer: [0; 32],
        }
    }

    /// Set every bit (pixel) in the buffer to be 0.
    pub fn clear(&mut self) {
        self.pixel_buffer = [0; 32];
    }

    /// Set the value of a pixel using a row and column.
    pub fn set_pixel(&mut self, row: usize, col: usize, status: bool) {
        self.check_bounds(row, col);

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
        let screen = Screen::new();
        assert_eq!(screen.pixel_buffer.len(), 32);
        assert_eq!(screen.pixel_buffer[0], 0);
        assert_eq!(screen.pixel_buffer[16], 0);
        assert_eq!(screen.pixel_buffer[31], 0);
    }

    #[test]
    fn test_clearing_screen() {
        let mut screen = Screen::new();
        screen.pixel_buffer[0] = 1;
        screen.pixel_buffer[16] = 1;
        screen.pixel_buffer[31] = 1;

        screen.clear();
        assert_eq!(screen.pixel_buffer, [0; 32]);
    }

    #[test]
    fn test_setting_pixel() {
        let mut screen = Screen::new();

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
        let mut screen = Screen::new();

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
}
