#![allow(dead_code)]

use rand::Rng;

use std::fs::File;
use std::io::Read;

mod screen;
use screen::Screen;

// Constants
const OFFSET: usize = 0x200; // Beginning of memory reserved for program.

fn main() {
    let path = "./roms/BC_test.ch8";
    VM::run(path);
}

/// VM for Chip8. Instantiates a CPU, sets configuration variables, draws UI for selecting roms.
/// Handles drawing the CPU's pixel buffer to the screen, playing sounds and retrieving user input.
struct VM {}

impl VM {
    pub fn run(path: &str) {
        // Read ROM into Vec<u8> which can then be loaded into CPU memory.
        let mut file = File::open(path).expect("unable to open file");
        let mut rom = Vec::new();
        file.read_to_end(&mut rom).expect("interrupted reading rom");

        // Create CPU and load ROM.
        let mut cpu = CPU::default();
        cpu.load(rom);

        cpu.run();
    }
}

/// Represents the CPU of a computer that could run Chip8 programs.
#[derive(Debug, PartialEq)]
struct CPU {
    // CPU clock speed in MHz.
    clock: u16,

    // Memory consists of 4096 bytes. 0x000 to 0x1FF for interpreter (0x050 to 0x0A0 for font set). 0x200 onwards for program.
    memory: [u8; 4096],

    // Group of 16 8-bit registers (0x0 to 0xF). Register V[F] is a flag not for use by programs.
    v: [u8; 16],

    // Group of 16 16-bit registers for holding addresses of called subroutines. Stack pointer points to current level.
    stack: [u16; 16],
    sp: usize,

    // 16-bit register used to store memory addresses. Only 4k of memory so only 12 bits are used.
    i: u16,

    // 16-bit register used to store address of currently executing instruction. Using usize to reduce number of casts.
    pc: usize,

    // Two 8-bit registers used as timers. One for Delay, one for Sound. Decrement at 60Hz when set.
    delay_timer: u8,
    sound_timer: u8,

    screen: Screen, // Display is monochrome and 64x32.
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            clock: 540,
            memory: [0; 4096],
            v: [0; 16],
            sp: 0,
            stack: [u16::MAX; 16],
            i: 0,
            pc: OFFSET,
            delay_timer: 0,
            sound_timer: 0,
            screen: Screen::new(),
        }
    }
}

impl CPU {
    pub fn run(&mut self) {}

    /// Read a Vec<u8> ROM into memory.
    pub fn load(&mut self, rom: Vec<u8>) {
        self.memory[OFFSET..OFFSET + rom.len()].copy_from_slice(&rom); // Load ROM into program memory.
    }

    /// Get the current opcode. Two bytes. Big endian. First always at positive index.
    fn get_instruction(&self) -> u16 {
        (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16)
    }

    // Execute the instruction/opcode pointed to by the program counter
    fn execute_instruction(&mut self) {
        let instruction = self.get_instruction();
        let nibbles = (
            (instruction & 0xF000) >> 12,
            ((instruction & 0x0F00) >> 8) as usize,
            ((instruction & 0x00F0) >> 4) as usize,
            (instruction & 0x000F),
        );

        match nibbles {
            // CLS --> Clear the screen.
            (0x0, 0x0, 0xE, 0x0) => {
                dbg!(format!("{:#04x} -> CLS", instruction));

                self.screen.clear();
                self.pc += 2;
            }

            // RET -> Exit subroutine. Set program counter to top address in the stack. Move stack pointer back.
            (0x0, 0x0, 0xE, 0xE) => {
                dbg!(format!("{:#04x} -> RET", instruction));

                self.pc = self.stack[self.sp] as usize;
                self.stack[self.sp] = u16::MAX;
                self.sp -= 1;
            }

            // JP nnn -> Set the program counter to address given by the three lowest nibbles.
            (0x1, _, _, _) => {
                dbg!(format!(
                    "{:#04x} -> JP {:#03x}",
                    instruction,
                    instruction & 0x0FFF
                ));

                self.pc = OFFSET + (instruction & 0x0FFF) as usize;
            }

            // CALL nnn -> Add current PC to stack. Set PC to given address.
            (0x2, _, _, _) => {
                dbg!(format!(
                    "{:#04x} -> CALL {:#03x}",
                    instruction,
                    instruction & 0x0FFF
                ));

                self.sp += 1;
                self.stack[self.sp] = self.pc as u16;
                self.pc = OFFSET + (instruction & 0x0FFF) as usize;
            }

            // SE Vx nn --> Skip next instruction if specified register is equal to specified value.
            (0x3, x, _, _) => {
                dbg!(format!(
                    "{:#04x} -> SE V{} {:#02x}",
                    instruction,
                    x,
                    instruction & 0x00FF
                ));

                self.pc += if self.v[x] == (instruction & 0x00FF) as u8 {
                    4
                } else {
                    2
                }
            }

            // SNE Vx nn --> Skip next instruction if Vx is not equal to specified value.
            (0x4, x, _, _) => {
                dbg!(format!(
                    "{:#04x} -> SNE V{} {:#02x}",
                    instruction,
                    x,
                    instruction & 0x00FF
                ));

                self.pc += if self.v[x] != (instruction & 0x00FF) as u8 {
                    4
                } else {
                    2
                }
            }

            // SE Vx Vy --> Skip next instruction if Vx is equal to Vy.
            (0x5, x, y, 0x0) => {
                dbg!(format!("{:#04x} -> SE V{} V{}", instruction, x, y));

                self.pc += if self.v[x] == self.v[y] { 4 } else { 2 }
            }

            // LD Vx nn --> Put a value into a specified V register.
            (0x6, x, _, _) => {
                dbg!(format!(
                    "{:#04x} -> LD V{} {:#02x}",
                    instruction,
                    x,
                    instruction & 0x00FF
                ));

                self.v[x] = (instruction & 0x00FF) as u8;

                self.pc += 2;
            }

            // ADD Vx nn --> Add specified value to register Vx.
            (0x7, x, _, _) => {
                dbg!(format!(
                    "{:#04x} -> ADD V{} {:#02x}",
                    instruction,
                    x,
                    instruction & 0x00FF
                ));

                self.v[x] = self.v[x].wrapping_add((instruction & 0x00FF) as u8);
                self.pc += 2;
            }

            // LD Vx Vy --> Store value of Vy in Vx
            (0x8, x, y, 0x0) => {
                dbg!(format!("{:#04x} -> LD V{} V{}", instruction, x, y));

                self.v[x] = self.v[y];

                self.pc += 2;
            }

            // OR Vx Vy --> Store value of bitwise OR between Vx and Vy.
            (0x8, x, y, 0x1) => {
                dbg!(format!("{:#04x} -> OR V{} V{}", instruction, x, y));

                self.v[x] = self.v[x] | self.v[y];

                self.pc += 2
            }

            // AND Vx Vy --> Store value of bitwise AND between Vx and Vy.
            (0x8, x, y, 0x2) => {
                dbg!(format!("{:#04x} -> AND V{} V{}", instruction, x, y));

                self.v[x] = self.v[x] & self.v[y];

                self.pc += 2;
            }

            // XOR Vx Vy --> Store value of bitwise XOR between Vx and Vy.
            (0x8, x, y, 0x3) => {
                dbg!(format!("{:#04x} -> XOR V{} V{}", instruction, x, y));

                self.v[x] = self.v[x] ^ self.v[y];

                self.pc += 2;
            }

            // ADD Vx Vy --> Add Vy to Vx. Store least significant byte. Set VF if overflow occurred.
            (0x8, x, y, 0x4) => {
                dbg!(format!("{:#04x} -> ADD V{} V{}", instruction, x, y));

                let res = self.v[x] as u16 + self.v[y] as u16;

                self.v[0xF] = if res > 255 { 1 } else { 0 };
                self.v[x] = res as u8;

                self.pc += 2;
            }

            // SUB Vx Vy --> Store value of Vx - Vy and set VF to 1 if Vx is greater than Vy (i.e. no borrow occurred).
            (0x8, x, y, 0x5) => {
                dbg!(format!("{:#04x} -> SUB V{} V{}", instruction, x, y));

                self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
                self.v[x] = self.v[x].wrapping_sub(self.v[y]);

                self.pc += 2;
            }

            // SHR Vx Vy --> Shift Vy one bit to the right and store result. Set Vf based on smallest bit.
            (0x8, x, y, 0x6) => {
                dbg!(format!("{:#04x} -> SHR V{} V{}", instruction, x, y));

                self.v[0xF] = if self.v[y] & 1 == 1 { 1 } else { 0 };
                self.v[x] = self.v[y] >> 1;

                self.pc += 2;
            }

            // SUBN Vx Vy --> Store value of Vy - Vx and set VF to 1 if Vy is greater than Vx (i.e. no borrow occurred).
            (0x8, x, y, 0x7) => {
                dbg!(format!("{:#04x} -> SUBN V{} V{}", instruction, x, y));

                self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
                self.v[x] = self.v[y].wrapping_sub(self.v[x]);

                self.pc += 2;
            }

            // SHL Vx Vy --> Shift Vy one bit and store. Set Vf based on largest bit.
            (0x8, x, y, 0xE) => {
                dbg!(format!("{:#04x} -> SHL V{} V{}", instruction, x, y));

                self.v[0xf] = if (self.v[y] >> 7) & 1 == 1 { 1 } else { 0 };
                self.v[x] = self.v[y] << 1;

                self.pc += 2;
            }

            // SNE Vx Vy --> Skip the next instruction if Vx != Vy.
            (0x9, x, y, 0x0) => {
                dbg!(format!("{:#04x} -> SNE V{} V{}", instruction, x, y));

                self.pc += if self.v[x] != self.v[y] { 4 } else { 2 }
            }

            // LD I nnn --> Load nnn into the I register.
            (0xA, _, _, _) => {
                dbg!(format!(
                    "{:#04x} -> LD I {}",
                    instruction,
                    instruction & 0x0FFF
                ));

                self.i = instruction & 0x0FFF;
                self.pc += 2;
            }

            // JP V0 nnn --> Jump to location V0 + nnn.
            (0xB, _, _, _) => {
                dbg!(format!(
                    "{:#04x} -> JP V0 {}",
                    instruction,
                    instruction & 0x0FFF
                ));

                self.pc = (self.v[0] as u16 + (instruction & 0x0FFF)) as usize;

                if self.pc < 0x200 {
                    panic!("memory address {:#04x} is not valid", self.pc);
                }
            }

            // RND Vx kk --> Generate a random byte and mask using kk.
            (0xC, x, _, _) => {
                dbg!(format!(
                    "{:#04x} -> RND V{} {:#02x}",
                    instruction,
                    x,
                    instruction & 0x00FF
                ));

                let mut rng = rand::thread_rng();
                let num = rng.gen_range(0..266);
                self.v[x] = (num & (instruction & 0x00FF)) as u8;

                self.pc += 2;
            }

            // ADD I Vx --> Store I + Vx in the I register.
            (0xF, x, 1, 0xE) => {
                dbg!(format!("{:#04x} -> ADD I V{}", instruction, x));

                self.i = self.i.wrapping_add(self.v[x] as u16);
                self.pc += 2;
            }

            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    fn create_test_cpu() -> CPU {
        let path = "./roms/BC_test.ch8";

        // Read ROM into Vec<u8> which can then be loaded into CPU memory.
        let mut file = File::open(path).expect("unable to open file");
        let mut rom = Vec::new();
        file.read_to_end(&mut rom).expect("interrupted reading rom");

        // Create CPU and load ROM.
        let mut cpu = CPU::default();
        cpu.load(rom);

        cpu
    }

    fn load_and_execute_instruction(cpu: &mut CPU, instr: u16) {
        cpu.pc = 0x200;
        cpu.memory[0x200] = (instr >> 8) as u8;
        cpu.memory[0x201] = instr as u8;
        cpu.execute_instruction();
    }

    #[test]
    fn test_creating_default_cpu() {
        let cpu = CPU::default();
        let expected = CPU {
            clock: 540,
            memory: [0; 4096],
            v: [0; 16],
            sp: 0,
            stack: [u16::MAX; 16],
            i: 0,
            pc: OFFSET,
            delay_timer: 0,
            sound_timer: 0,
            screen: Screen::new(),
        };
        assert_eq!(cpu, expected);
    }

    #[test]
    fn test_loading_rom() {
        let cpu = create_test_cpu();
        assert_eq!(cpu.memory[0x200], 0x00);
        assert_eq!(cpu.memory[0x201], 0xE0);
        assert_eq!(cpu.memory[0x202], 0x63);
    }

    #[test]
    fn test_fetching_instructions() {
        let mut cpu = create_test_cpu();

        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.get_instruction(), 0x00E0);

        cpu.pc = 0x202;
        assert_eq!(cpu.get_instruction(), 0x6300);

        cpu.pc = 0x204;
        assert_eq!(cpu.get_instruction(), 0x6401);
    }

    #[test]
    /// Screen should be cleared.
    fn test_CLS_opcode() {
        // Initialise test by creating a CPU and turning some pixels on.
        let mut cpu = CPU::default();
        cpu.screen.set_pixel(0, 0, true);
        cpu.screen.set_pixel(16, 32, true);
        cpu.screen.set_pixel(31, 63, true);

        // Execute the given instruction.
        load_and_execute_instruction(&mut cpu, 0x00E0);

        // Check the screen was cleared.
        assert_eq!(cpu.screen.get_pixel(0, 0), false);
        assert_eq!(cpu.screen.get_pixel(16, 32), false);
        assert_eq!(cpu.screen.get_pixel(31, 63), false);

        // Check PC advanced 2 memory addresses since instructions are 2 bytes long.
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// PC should jump to address pointed to on stack by SP.
    fn test_RET_opcode() {
        // Initialise test by creating CPU and adding memory address to the stack.
        let mut cpu = CPU::default();
        cpu.stack[1] = 0x202;
        cpu.sp = 1;

        load_and_execute_instruction(&mut cpu, 0x00EE);

        assert_eq!(cpu.stack[1], u16::MAX);
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.sp, 0);
    }

    #[test]
    /// PC should jump to memory address NNN.
    fn test_JP_NNN_opcode() {
        let mut cpu = CPU::default();
        load_and_execute_instruction(&mut cpu, 0x1102);

        assert_eq!(cpu.pc, 0x302);
    }

    #[test]
    /// PC should jump to memory address NNN and add previous address to the stack.
    fn test_CALL_NNN_opcode() {
        let mut cpu = CPU::default();
        load_and_execute_instruction(&mut cpu, 0x2102);

        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[cpu.sp], 0x200);
        assert_eq!(cpu.pc, 0x302);
    }

    #[test]
    /// PC should skip next instruction if Vx == NN.
    fn test_SE_Vx_NN_opcode() {
        let mut cpu = CPU::default();

        // Skip instruction (advanced PC by 4) if V[3] == 4;
        cpu.v[0x3] = 4;
        load_and_execute_instruction(&mut cpu, 0x3304);
        assert_eq!(cpu.pc, 0x204);

        // V[3] == 2 so don't skip instruction (advance PC by 2).
        cpu.v[0x3] = 2;
        load_and_execute_instruction(&mut cpu, 0x3304);
        assert_eq!(cpu.pc, 0x202)
    }

    #[test]
    /// PC should skip next instruction if Vx != NN.
    fn test_SNE_Vx_NN_opcode() {
        let mut cpu = CPU::default();

        cpu.v[0x3] = 4;
        load_and_execute_instruction(&mut cpu, 0x4304);
        assert_eq!(cpu.pc, 0x202);

        cpu.v[0x3] = 2;
        load_and_execute_instruction(&mut cpu, 0x4304);
        assert_eq!(cpu.pc, 0x204)
    }

    #[test]
    /// PC should skip next instruction if Vx == Vn.
    fn test_SE_Vx_Vy_opcode() {
        let mut cpu = CPU::default();

        cpu.v[0x3] = 4;
        cpu.v[0x4] = 4;
        load_and_execute_instruction(&mut cpu, 0x5340);
        assert_eq!(cpu.pc, 0x204);

        cpu.v[0x3] = 4;
        cpu.v[0x4] = 5;
        load_and_execute_instruction(&mut cpu, 0x5340);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Vx should contain NN.
    fn test_LD_Vx_NN_opcode() {
        let mut cpu = CPU::default();

        load_and_execute_instruction(&mut cpu, 0x6D12);
        assert_eq!(cpu.v[0xD], 0x12);
        assert_eq!(cpu.pc, 0x202);

        load_and_execute_instruction(&mut cpu, 0x6401);
        assert_eq!(cpu.v[0x4], 0x01);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// NN should be added to the value of Vx. Wrap upon overflow.
    fn test_ADD_Vx_NN_opcode() {
        let mut cpu = CPU::default();

        load_and_execute_instruction(&mut cpu, 0x7311);
        assert_eq!(cpu.v[0x3], 0x11);
        assert_eq!(cpu.pc, 0x202);

        cpu.v[0x4] = 8;
        load_and_execute_instruction(&mut cpu, 0x7401);
        assert_eq!(cpu.v[0x4], 0x09);
        assert_eq!(cpu.pc, 0x202);

        cpu.v[0x8] = 0xFF;
        load_and_execute_instruction(&mut cpu, 0x7804);
        assert_eq!(cpu.v[0x8], 0x03);
    }

    #[test]
    /// Value in Vy should be stored in Vx.
    fn test_LD_Vx_Vy_opcode() {
        let mut cpu = CPU::default();

        cpu.v[1] = 1;
        cpu.v[2] = 2;
        load_and_execute_instruction(&mut cpu, 0x8120);
        assert_eq!(cpu.v[1], 2);
        assert_eq!(cpu.v[2], 2);
        assert_eq!(cpu.pc, 0x202);

        load_and_execute_instruction(&mut cpu, 0x8130);
        assert_eq!(cpu.v[1], 0);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Vx should store result of Vx OR Vy.
    fn test_OR_Vx_Vy_opcode() {
        let mut cpu = CPU::default();

        cpu.v[1] = 73;
        cpu.v[2] = 23;
        load_and_execute_instruction(&mut cpu, 0x8121);

        assert_eq!(cpu.v[1], 95);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Vx should store result of Vx AND Vy.
    fn test_AND_Vx_Vy_opcode() {
        let mut cpu = CPU::default();

        cpu.v[1] = 73;
        cpu.v[2] = 23;
        load_and_execute_instruction(&mut cpu, 0x8122);

        assert_eq!(cpu.v[1], 1);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Vx should store result of Vx XOR Vy.
    fn test_XOR_Vx_Vy_opcode() {
        let mut cpu = CPU::default();
        cpu.v[1] = 73;
        cpu.v[2] = 23;
        load_and_execute_instruction(&mut cpu, 0x8123);

        assert_eq!(cpu.v[1], 94);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Vx should store result of Vx + Vy. Set VF if overflow occurs.
    fn test_ADD_Vx_Vy_opcode() {
        let mut cpu = CPU::default();
        cpu.v[1] = 73;
        cpu.v[2] = 23;
        load_and_execute_instruction(&mut cpu, 0x8124);

        assert_eq!(cpu.v[1], 96);
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, 0x202);

        cpu.v[3] = 255;
        cpu.v[4] = 90;
        load_and_execute_instruction(&mut cpu, 0x8344);

        assert_eq!(cpu.v[3], 89);
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Vx should store the result of Vx - Vy. Set CF to 1 if an underflow DOES NOT occur.
    fn test_SUB_Vx_Vy_opcode() {
        let mut cpu = CPU::default();
        cpu.v[1] = 73;
        cpu.v[2] = 23;
        load_and_execute_instruction(&mut cpu, 0x8125);

        assert_eq!(cpu.v[1], 50);
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, 0x202);

        cpu.v[3] = 1;
        cpu.v[4] = 22;
        load_and_execute_instruction(&mut cpu, 0x8345);

        assert_eq!(cpu.v[3], 235);
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Set Vf to 1 if smallest bit of Vy is 1. Vx should contain Vy >> 1.
    fn test_SHR_Vx_Vy_opcode() {
        let mut cpu = CPU::default();
        cpu.v[2] = 64;
        load_and_execute_instruction(&mut cpu, 0x8126);

        assert_eq!(cpu.v[1], 32);
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, 0x202);

        cpu.v[2] = 3;
        load_and_execute_instruction(&mut cpu, 0x8126);

        assert_eq!(cpu.v[1], 1);
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Vx should store Vy - Vx. VD should contain 1 if underflow DID NOT occur.
    fn test_SUBN_Vx_Vy_opcode() {
        let mut cpu = CPU::default();
        cpu.v[1] = 73;
        cpu.v[2] = 23;
        load_and_execute_instruction(&mut cpu, 0x8127);

        assert_eq!(cpu.v[1], 206);
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, 0x202);

        cpu.v[1] = 23;
        cpu.v[2] = 73;
        load_and_execute_instruction(&mut cpu, 0x8127);

        assert_eq!(cpu.v[1], 50);
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Set VF to 1 if largest bit of Vy is 1. Vx should contain Vy << 1.
    fn test_SHL_Vx_Vy_opcode() {
        let mut cpu = CPU::default();

        cpu.v[2] = 32;
        load_and_execute_instruction(&mut cpu, 0x812E);

        assert_eq!(cpu.v[1], 64);
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, 0x202);

        cpu.v[2] = u8::MAX;
        load_and_execute_instruction(&mut cpu, 0x812E);

        assert_eq!(cpu.v[1], 254);
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Should skip next instruction if Vx != Vy.
    fn test_SNE_Vx_Vy_opcode() {
        let mut cpu = CPU::default();

        cpu.v[1] = 1;
        cpu.v[2] = 2;
        load_and_execute_instruction(&mut cpu, 0x9120);
        assert_eq!(cpu.pc, 0x204);

        cpu.v[1] = 2;
        cpu.v[2] = 2;
        load_and_execute_instruction(&mut cpu, 0x9120);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Should store address NNN in the I register.
    fn test_LD_I_NNN_opcode() {
        let mut cpu = CPU::default();

        load_and_execute_instruction(&mut cpu, 0xA304);
        assert_eq!(cpu.i, 0x304);
        assert_eq!(cpu.pc, 0x202);

        load_and_execute_instruction(&mut cpu, 0xA444);
        assert_eq!(cpu.i, 0x444);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    /// Should jump PC to V0 + NNN.
    fn test_JP_V0_NNN_opcode() {
        let mut cpu = CPU::default();

        cpu.v[0] = 0x22;
        load_and_execute_instruction(&mut cpu, 0xB200);
        assert_eq!(cpu.pc, 0x222);

        cpu.v[0] = 0xFF;
        load_and_execute_instruction(&mut cpu, 0xB300);
        assert_eq!(cpu.pc, 0x3FF);
    }

    #[test]
    /// TODO -- Should generate a random number and do a bitwise AND with KK.
    fn test_RND_Vx_KK_opcode() {}

    #[test]
    /// Should store I + Vx in I.
    fn test_ADD_I_Vx_opcode() {
        let mut cpu = CPU::default();

        cpu.v[0] = 25;
        load_and_execute_instruction(&mut cpu, 0xF01E);
        assert_eq!(cpu.i, 25);
        assert_eq!(cpu.pc, 0x202);

        cpu.i = 12;
        cpu.v[1] = 59;
        load_and_execute_instruction(&mut cpu, 0xF11E);
        assert_eq!(cpu.i, 71);
        assert_eq!(cpu.pc, 0x202);
    }
}
