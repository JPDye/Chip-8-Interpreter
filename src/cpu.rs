// Self imports
use crate::keypad::Keypad;
use crate::screen::Screen;

use crate::OFFSET;
use crate::WRAP_X;
use crate::WRAP_Y;

// External imports
use rand::Rng;

/// The three things a Program Counter can do...
enum ProgramCounter {
    Next,
    Skip,
    Jump(usize),
}

impl ProgramCounter {
    fn skip_if(condition: bool) -> ProgramCounter {
        if condition {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }
}

/// Represents the CPU of a computer that could run Chip8 programs.
#[derive(Debug, PartialEq)]
pub struct CPU {
    // Memory consists of 4096 bytes. 0x000 to 0x1FF for interpreter (0x050 to 0x0A0 for font set). 0x200 onwards for program.
    memory: [u8; 4096],

    // Group of 16 8-bit registers (0x0 to 0xF). Register V[F] is a flag not for use by programs.
    v: [u8; 16],

    // Group of 16 16-bit registers for holding addresses of called subroutines. Stack pointer points to current level.
    stack: [usize; 16],
    sp: usize,

    // 16-bit register used to store memory addresses. Only 4k of memory so only 12 bits are used.
    i: usize,

    // 16-bit register used to store address of currently executing instruction. Using usize to reduce number of casts.
    pc: usize,

    // Two 8-bit registers used as timers. One for Delay, one for Sound. Decrement at 60Hz when set.
    delay_timer: u8,
    sound_timer: u8,

    // Display is monochrome and 64x32.
    screen: Screen,

    // 16 possible keys. Mapping found in Keycode file.
    keypad: Keypad,
}

impl Default for CPU {
    fn default() -> Self {
        let mut cpu = Self {
            memory: [0; 4096],
            v: [0; 16],
            sp: 0,
            stack: [usize::MAX; 16],
            i: 0,
            pc: OFFSET,
            delay_timer: 0,
            sound_timer: 0,
            screen: Screen::new(WRAP_X, WRAP_Y),
            keypad: Keypad::new(),
        };

        cpu.load_font();
        cpu
    }
}

impl CPU {
    pub fn cycle(&mut self) {
        self.execute_instruction(self.get_instruction())
    }

    /// Read a Vec<u8> ROM into memory.
    pub fn load(&mut self, rom: Vec<u8>) {
        self.memory[OFFSET..OFFSET + rom.len()].copy_from_slice(&rom); // Load ROM into program memory.
    }

    /// Get the current opcode. Two bytes. Big endian. First always at positive index.
    fn get_instruction(&self) -> usize {
        (self.memory[self.pc] as usize) << 8 | (self.memory[self.pc + 1] as usize)
    }

    /// Execute the instruction/opcode pointed to by the program counter
    fn execute_instruction(&mut self, instruction: usize) {
        let nibbles = (
            (instruction & 0xF000) >> 12,
            ((instruction & 0x0F00) >> 8) as usize,
            ((instruction & 0x00F0) >> 4) as usize,
            (instruction & 0x000F),
        );

        let kk = (instruction & 0x00FF) as u8;
        let nnn = instruction & 0x0FFF;

        let pc_change = match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.opcode_00e0(),
            (0x0, 0x0, 0xE, 0xE) => self.opcode_00ee(),
            (0x1, _, _, _) => self.opcode_1nnn(nnn),
            (0x2, _, _, _) => self.opcode_2nnn(nnn),
            (0x3, x, _, _) => self.opcode_3xkk(x, kk),
            (0x4, x, _, _) => self.opcode_4xkk(x, kk),
            (0x5, x, y, 0x0) => self.opcode_5xy0(x, y),
            (0x6, x, _, _) => self.opcode_6xkk(x, kk),
            (0x7, x, _, _) => self.opcode_7xkk(x, kk),
            (0x8, x, y, 0x0) => self.opcode_8xy0(x, y),
            (0x8, x, y, 0x1) => self.opcode_8xy1(x, y),
            (0x8, x, y, 0x2) => self.opcode_8xy2(x, y),
            (0x8, x, y, 0x3) => self.opcode_8xy3(x, y),
            (0x8, x, y, 0x4) => self.opcode_8xy4(x, y),
            (0x8, x, y, 0x5) => self.opcode_8xy5(x, y),
            (0x8, x, y, 0x6) => self.opcode_8xy6(x, y),
            (0x8, x, y, 0x7) => self.opcode_8xy7(x, y),
            (0x8, x, y, 0xE) => self.opcode_8xye(x, y),
            (0x9, x, y, 0x0) => self.opcode_9xy0(x, y),
            (0xA, _, _, _) => self.opcode_annn(nnn),
            (0xB, _, _, _) => self.opcode_bnnn(nnn),
            (0xC, x, _, _) => self.opcode_cxkk(x, kk),
            (0xD, x, y, n) => self.opcode_dxyn(x, y, n),
            (0xE, x, 0x9, 0xE) => self.opcode_ex9e(x),
            (0xE, x, 0xA, 0x1) => self.opcode_exa1(x),
            (0xF, x, 0x0, 0x7) => self.opcode_fx07(x),
            (0xF, x, 0x0, 0xA) => self.opcode_fx0a(x),
            (0xF, x, 0x1, 0x5) => self.opcode_fx15(x),
            (0xF, x, 0x1, 0x8) => self.opcode_fx18(x),
            (0xF, x, 1, 0xE) => self.opcode_fx1e(x),
            (0xF, x, 0x2, 0x9) => self.opcode_fx29(x),
            (0xF, x, 0x3, 0x3) => self.opcode_fx33(x),
            (0xF, x, 0x5, 0x5) => self.opcode_fx55(x),
            (0xF, x, 0x6, 0x5) => self.opcode_fx65(x),
            _ => panic!("{:#04x} is not a valid opcode", instruction),
        };

        match pc_change {
            ProgramCounter::Next => self.pc += 2,
            ProgramCounter::Skip => self.pc += 4,
            ProgramCounter::Jump(addr) => self.pc = addr,
        };
    }

    /// CLS --> Clear the screen.
    fn opcode_00e0(&mut self) -> ProgramCounter {
        self.screen.clear();
        ProgramCounter::Next
    }

    /// RET -> Exit subroutine. Set program counter to top address in the stack and subtract 1 from the stack pointer.
    fn opcode_00ee(&mut self) -> ProgramCounter {
        self.sp -= 1;
        ProgramCounter::Jump(self.stack[self.sp])
    }

    /// JP nnn -> Jump program counter to given address (plus the offset).
    fn opcode_1nnn(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Jump(OFFSET + nnn)
    }

    /// CALL nnn -> Add current program counter to stack and set program counter to given address.
    fn opcode_2nnn(&mut self, nnn: usize) -> ProgramCounter {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        ProgramCounter::Jump(OFFSET + nnn)
    }

    /// SE Vx kk --> Skip next instruction if Vx is equal to kk.
    fn opcode_3xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        ProgramCounter::skip_if(self.v[x] == kk)
    }

    /// SNE Vx kk --> Skip next instruction if Vx is not equal to kk.
    fn opcode_4xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        ProgramCounter::skip_if(self.v[x] != kk)
    }

    /// SE Vx Vy --> Skip next instruction if Vx is equal to Vy.
    fn opcode_5xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.v[x] == self.v[y])
    }

    /// LD Vx kk --> Load nn into Vx.
    fn opcode_6xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        self.v[x] = kk;
        ProgramCounter::Next
    }
    /// ADD Vx kk --> Add kk to the contents of Vx and store in Vx.
    fn opcode_7xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        self.v[x] = self.v[x].wrapping_add(kk);
        ProgramCounter::Next
    }

    /// LD Vx Vy --> Store value of Vy in Vx
    fn opcode_8xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] = self.v[y];
        ProgramCounter::Next
    }

    /// OR Vx Vy --> Store value of bitwise OR between Vx and Vy.
    fn opcode_8xy1(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] |= self.v[y];
        ProgramCounter::Next
    }

    /// AND Vx Vy --> Store value of bitwise AND between Vx and Vy.
    fn opcode_8xy2(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] &= self.v[y];
        ProgramCounter::Next
    }

    /// XOR Vx Vy --> Store value of bitwise XOR between Vx and Vy.
    fn opcode_8xy3(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[x] ^= self.v[y];
        ProgramCounter::Next
    }

    /// ADD Vx Vy --> Add Vx and Vy. Store result in Vx. Set VF to 1 upon overflow.
    fn opcode_8xy4(&mut self, x: usize, y: usize) -> ProgramCounter {
        let vx = self.v[x] as u16;
        let vy = self.v[y] as u16;
        let res = vx + vy;

        self.v[0xF] = if res > 255 { 1 } else { 0 };
        self.v[x] = res as u8;

        ProgramCounter::Next
    }

    /// SUB Vx Vy --> Store value of Vx - Vy and set VF to 1 if Vx is greater than Vy (i.e. no borrow occurred).
    fn opcode_8xy5(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
        ProgramCounter::Next
    }

    /// SHR Vx Vy --> Shift Vy one bit to the right and store result. Set VF if underflow occurs.
    fn opcode_8xy6(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = self.v[y] & 1;
        self.v[x] = self.v[y] >> 1;
        ProgramCounter::Next
    }

    /// SUBN Vx Vy --> Store value of Vy - Vx and set VF to 1 if Vy is greater than Vx (i.e. no borrow occurred).
    fn opcode_8xy7(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
        ProgramCounter::Next
    }

    /// SHL Vx Vy --> Shift Vy one bit and store. Set VF if overflow occurs.
    fn opcode_8xye(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.v[0xf] = (self.v[y] >> 7) & 1;
        self.v[x] = self.v[y] << 1;
        ProgramCounter::Next
    }

    /// SNE Vx Vy --> Skip the next instruction if Vx != Vy.
    fn opcode_9xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.v[x] != self.v[y])
    }

    /// LD I nnn --> Load nnn into the I register.
    fn opcode_annn(&mut self, nnn: usize) -> ProgramCounter {
        self.i = nnn;
        ProgramCounter::Next
    }

    /// JP V0 nnn --> Jump to location V0 + nnn.
    fn opcode_bnnn(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Jump(self.v[0] as usize + nnn)
    }

    /// RND Vx kk --> Generate a random byte and AND with nnn Store result in Vx.
    fn opcode_cxkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        let mut rng = rand::thread_rng();
        self.v[x] = rng.gen::<u8>() & kk;
        ProgramCounter::Next
    }

    /// DRW Vx Vy n --> Draw the sprite beginning at memory address I and ending at I + k at position (Vx, Vy).
    fn opcode_dxyn(&mut self, x: usize, y: usize, n: usize) -> ProgramCounter {
        let sprite = &self.memory[self.i..self.i + n];
        self.screen.draw_sprite(sprite, y, x);
        ProgramCounter::Next
    }

    /// SKP Vx --> Skip next instruction if the key with value Vx is pressed.
    fn opcode_ex9e(&mut self, x: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.keypad.is_pressed(self.v[x]))
    }

    /// SKNP Vx --> Skip next instruction if the key with the value Vx is not pressed.
    fn opcode_exa1(&mut self, x: usize) -> ProgramCounter {
        ProgramCounter::skip_if(!self.keypad.is_pressed(self.v[x]))
    }

    /// LD Vx DT --> The value of the delay timer is places into Vx.
    fn opcode_fx07(&mut self, x: usize) -> ProgramCounter {
        self.v[x] = self.delay_timer;
        ProgramCounter::Next
    }

    /// LD Vx K --> Wait for a keypress and store value of the key in Vx.
    fn opcode_fx0a(&mut self, x: usize) -> ProgramCounter {
        for k in 0..15 {
            if self.keypad.is_pressed(k) {
                self.v[x] = k;
                return ProgramCounter::Next;
            }
        }
        ProgramCounter::Jump(self.pc) // Eww. Maybe add ProgramCounter::Back.
    }

    /// LD DT Vx --> Set delay timer to value stored in Vx.
    fn opcode_fx15(&mut self, x: usize) -> ProgramCounter {
        self.delay_timer = self.v[x];
        ProgramCounter::Next
    }

    /// LD ST Vx --> Load value of Vx into sound timer.
    fn opcode_fx18(&mut self, x: usize) -> ProgramCounter {
        self.sound_timer = self.v[x];
        ProgramCounter::Next
    }

    /// ADD I Vx --> Store I + Vx in the I register.
    fn opcode_fx1e(&mut self, x: usize) -> ProgramCounter {
        self.i = self.i.wrapping_add(self.v[x] as usize);
        ProgramCounter::Next
    }

    /// LD F Vx --> Set I to the location of the sprite for hexadecimal digit store in Vx.
    fn opcode_fx29(&mut self, x: usize) -> ProgramCounter {
        if self.v[x] > 16 {
            panic!("OP F{}29: {} is not a valid character.", x, x);
        }

        self.i = (self.v[x] * 5) as usize;
        ProgramCounter::Next
    }

    /// LD B Vx --> Store the binary coded decimal representation of Vx in memory locations I, I + 1 and I + 2.
    fn opcode_fx33(&mut self, x: usize) -> ProgramCounter {
        self.memory[self.i] = self.v[x >> 8] / 100;
        self.memory[self.i] = (self.v[x >> 8] / 10) % 10;
        self.memory[self.i] = (self.v[x >> 8] % 100) % 10;
        ProgramCounter::Next
    }

    /// LD <I> Vx --> Store registers 0 up to Vx in memory starting at I.
    fn opcode_fx55(&mut self, x: usize) -> ProgramCounter {
        for i in 0..=x {
            let idx = self.i + i;
            self.memory[idx] = self.v[i];
        }
        ProgramCounter::Next
    }

    /// LD Vx <I> --> Read values of I to I + x into registers V0 to Vx.
    fn opcode_fx65(&mut self, x: usize) -> ProgramCounter {
        for i in 0..=x {
            let idx = self.i + i;
            self.v[i] = self.memory[idx];
        }
        ProgramCounter::Next
    }

    #[rustfmt::skip]
    fn load_font(&mut self) {
        // 0 to F. 5 Bytes per character. Index in memory is the character's hex value multiplied by 5.
        let font: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0,
            0x20, 0x60, 0x20, 0x20, 0x70,
            0xF0, 0x10, 0xF0, 0x80, 0xF0,
            0xF0, 0x10, 0xF0, 0x10, 0xF0,
            0x90, 0x90, 0xF0, 0x10, 0x10,
            0xF0, 0x80, 0xF0, 0x10, 0xF0,
            0xF0, 0x80, 0xF0, 0x90, 0xF0,
            0xF0, 0x10, 0x20, 0x40, 0x40,
            0xF0, 0x90, 0xF0, 0x90, 0xF0,
            0xF0, 0x90, 0xF0, 0x10, 0xF0,
            0xF0, 0x90, 0xF0, 0x90, 0x90,
            0xE0, 0x90, 0xE0, 0x90, 0xE0,
            0xF0, 0x80, 0x80, 0x80, 0xF0,
            0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0,
            0xF0, 0x80, 0xF0, 0x80, 0x80
        ];

        self.memory[0..80].copy_from_slice(&font);
    }
}

#[cfg(test)]
#[path = "./cpu_tests.rs"]
mod cpu_tests;
