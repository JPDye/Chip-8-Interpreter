#![allow(dead_code)]

// Self imports
mod keypad;
mod screen;

mod cpu;
use cpu::CPU;

// Std imports
use std::{thread, time};
use std::fs::File;
use std::io::Read;

// External imports
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

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


    // Create event loop, initialise window and initialise pixel buffer.
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(640, 320);
        WindowBuilder::new()
            .with_title("Chip-8 in Rust!")
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(64, 32, surface_texture).unwrap()
    };

    // Cycle counter. Redraw on every 9th frame. Sleep 1800 microseconds every cycle.
    let mut cycle_counter = 0;
    let sleep_duration = time::Duration::from_micros(1800);


    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            pixels.get_frame().copy_from_slice(&cpu.get_buffer());
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }
        }

        cpu.cycle();
        cycle_counter += 1;
        thread::sleep(sleep_duration);
        if cycle_counter == 9 {
            window.request_redraw();
            cycle_counter = 0;
        }
    });
}
