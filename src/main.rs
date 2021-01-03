#![allow(dead_code)]

mod cpu;
mod drivers;
mod frame_buffer;
mod keypad;

// Self imports
use cpu::CPU;
use drivers::{DisplayDriver, InputDriver};

// Std imports
use std::fs::File;
use std::io::Read;
use std::time::Duration;

// External imports
use structopt::StructOpt;

// Constants
pub const WRAP_X: bool = true; // Wrap horizontally when drawing sprites?
pub const WRAP_Y: bool = true; // Wrap vertically when drawing sprites?

pub const OFFSET: usize = 0x200; // Beginning of memory reserved for program.

fn main() {
    let mut vm = VM::new("./roms/tetris.ch8");
    vm.run(Mode::Release);
}

#[derive(Copy, Clone, Debug)]
enum Mode {
    Debug,
    Release,
}

struct VM {
    cpu: CPU,
    display_driver: DisplayDriver,
    input_driver: InputDriver,
}

impl VM {
    pub fn new(path: &str) -> Self {
        // Initialise CPU and load ROM.
        let mut cpu = CPU::default();
        cpu.load(rom_from_path(path));


        // Create SDL context and I/O drivers.
        let sdl_context = sdl2::init().unwrap();
        let mut display_driver = DisplayDriver::new(&sdl_context);
        let mut input_driver = InputDriver::new(&sdl_context);

        Self {
            cpu,
            display_driver,
            input_driver,
        }
    }

    pub fn run(&mut self, mode: Mode) {
        // Sleep duration. Ensure games run at reasonable speed.
        let sleep_duration = Duration::from_micros(1800);

        // Render every 9th frame. Ensure games run at ~60FPS.
        let mut cycle_counter = 0;

        while let Ok(keycode) = self.input_driver.poll() {
            match keycode {
                Some(255) => self.cpu.dbg(),
                Some(key) => self.cpu.set_key(key),
                _ => self.cpu.clear_keys(),
            }

            match mode {
                Mode::Release => {
                    self.cpu.cycle();
                    cycle_counter += 1;
                    std::thread::sleep(sleep_duration);

                    if cycle_counter == 9 {
                        self.display_driver.draw(self.cpu.get_framebuffer());
                        cycle_counter = 0;
                    }
                }

                Mode::Debug => {
                    if let Some(255) = keycode {
                        self.cpu.cycle();
                        self.display_driver.draw(self.cpu.get_framebuffer());
                    }
                }
            }
        }
    }
}

// Read ROM into &[u8] which can then be loaded into CPU memory.
fn rom_from_path(path: &str) -> Vec<u8> {
    let mut file = File::open(path).expect("unable to open file");
    let mut rom = Vec::new();

    file.read_to_end(&mut rom).expect("interrupted reading rom");
    rom
}
