/*

------------------------------------------------------------------------------------------------------------------------

The computers which originally used the Chip-8 Language had a 16-key hexadecimal keypad arranged in a 4 by 4 grid.
Modern keyboards do not have the same layout and even using different keys, the offsets of each row mean the
originally keypad cannot truly be recreated.

------------------------------------------------------------------------------------------------------------------------

                                               ---------------
                                                1   2   3   C
                                               ---------------
                                                4   5   6   D
                                               ---------------
                                                7   8   9   E
                                               ---------------
                                                A   0   B   F
                                               ---------------

------------------------------------------------------------------------------------------------------------------------

                                               ---------------
                                                1   2   3   4
                                               ---------------
                                                Q   W   E   R
                                               ---------------
                                                A   S   D   F
                                               ---------------
                                                Z   X   C   V
                                               ---------------

------------------------------------------------------------------------------------------------------------------------
*/

#[derive(Debug, PartialEq)]
pub struct Keypad {
    keys: u16,
}

impl Keypad {
    pub fn new() -> Self {
        Self { keys: 0 }
    }

    pub fn set_pressed(&mut self, k: u8) {
        self.keys = 1 << k;
    }

    pub fn is_pressed(&self, k: u8) -> bool {
        (self.keys >> k) & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pressed_func() {
        let mut keypad = Keypad::new();

        keypad.keys = 1 << 0;
        assert_eq!(keypad.is_pressed(0x0), true);

        keypad.keys = 1 << 8;
        assert_eq!(keypad.is_pressed(0x8), true);

        keypad.keys = 1 << 0xF;
        assert_eq!(keypad.is_pressed(0xF), true);
    }

    #[test]
    fn test_set_pressed_func() {
        let mut keypad = Keypad::new();

        keypad.set_pressed(0);
        assert_eq!(keypad.is_pressed(0), true);

        keypad.set_pressed(8);
        assert_eq!(keypad.is_pressed(8), true);

        keypad.set_pressed(0xF);
        assert_eq!(keypad.is_pressed(0xF), true);
    }
}
