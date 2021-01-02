#![allow(dead_code)]

// Self imports
mod keypad;
mod screen;

mod cpu;
use cpu::CPU;

// Std imports
use std::fs::File;
use std::io::Read;

// Constants
pub const OFFSET: usize = 0x200; // Beginning of memory reserved for program.

pub const WRAP_X: bool = true; // Wrap horizontally when drawing sprites?
pub const WRAP_Y: bool = true; // Wrap vertically when drawing sprites?

const WIDTH: usize = 640;
const HEIGHT: usize = 320;

fn main() {
    let path = "./roms/BC_test.ch8";

    // Read ROM into Vec<u8> which can then be loaded into CPU memory.
    let mut file = File::open(path).expect("unable to open file");
    let mut rom = Vec::new();
    file.read_to_end(&mut rom).expect("interrupted reading rom");

    // Create CPU and load ROM.
    let mut cpu = CPU::default();
    cpu.load(rom);



}
