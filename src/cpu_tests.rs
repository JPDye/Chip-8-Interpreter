#![allow(non_snake_case)]

// Self imports
use crate::cpu::CPU;
use crate::keypad::Keypad;
use crate::screen::Screen;

use crate::OFFSET;

// Std imports
use std::fs::File;
use std::io::Read;

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
    cpu.execute_instruction(instr as usize);
}

#[test]
fn test_creating_default_cpu() {
    let cpu = CPU::default();

    let mut expected = CPU {
        memory: [0; 4096],
        v: [0; 16],
        sp: 0,
        stack: [usize::MAX; 16],
        i: 0,
        pc: OFFSET,
        delay_timer: 0,
        sound_timer: 0,
        screen: Screen::new(true, true),
        keypad: Keypad::new(),
    };
    expected.load_font();

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
    cpu.stack[0] = 0x202;
    cpu.sp = 1;

    load_and_execute_instruction(&mut cpu, 0x00EE);
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
    assert_eq!(cpu.stack[0], 0x200);
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
/// Should generate a random number and do a bitwise AND with KK.
fn test_RND_Vx_KK_opcode() {
    let mut cpu = CPU::default();

    for _ in 0..4 {
        load_and_execute_instruction(&mut cpu, 0xC003);
        assert!(cpu.v[0] <= 3);
    }

    for _ in 0..4 {
        load_and_execute_instruction(&mut cpu, 0xC07F);
        assert!(cpu.v[0] <= 127);
    }
}

#[test]
/// Should draw the sprite at given position. Sprite is a 0 for this case.
fn test_DRW_Vx_Vy_N_opcode() {
    let mut cpu = CPU::default();

    cpu.i = 0;
    load_and_execute_instruction(&mut cpu, 0xD005);

    assert_eq!(cpu.screen.get_pixel(0, 0), true);
    assert_eq!(cpu.screen.get_pixel(0, 1), true);
    assert_eq!(cpu.screen.get_pixel(0, 2), true);
    assert_eq!(cpu.screen.get_pixel(0, 3), true);
    assert_eq!(cpu.screen.get_pixel(0, 4), false);

    assert_eq!(cpu.screen.get_pixel(1, 0), true);
    assert_eq!(cpu.screen.get_pixel(1, 1), false);
    assert_eq!(cpu.screen.get_pixel(1, 2), false);
    assert_eq!(cpu.screen.get_pixel(1, 3), true);

    assert_eq!(cpu.screen.get_pixel(2, 0), true);
    assert_eq!(cpu.screen.get_pixel(2, 1), false);
    assert_eq!(cpu.screen.get_pixel(2, 2), false);
    assert_eq!(cpu.screen.get_pixel(2, 3), true);

    assert_eq!(cpu.screen.get_pixel(3, 0), true);
    assert_eq!(cpu.screen.get_pixel(3, 1), false);
    assert_eq!(cpu.screen.get_pixel(3, 2), false);
    assert_eq!(cpu.screen.get_pixel(3, 3), true);

    assert_eq!(cpu.screen.get_pixel(4, 0), true);
    assert_eq!(cpu.screen.get_pixel(4, 1), true);
    assert_eq!(cpu.screen.get_pixel(4, 2), true);
    assert_eq!(cpu.screen.get_pixel(4, 3), true);
    assert_eq!(cpu.screen.get_pixel(4, 4), false);
}

#[test]
/// Should skip the next instruction if key pressed has value Vx.
fn test_SKP_Vx_opcode() {
    let mut cpu = CPU::default();

    load_and_execute_instruction(&mut cpu, 0xE09E);
    assert_eq!(cpu.pc, 0x202);

    cpu.v[0] = 0xD;
    cpu.keypad.set_pressed(0xD);
    load_and_execute_instruction(&mut cpu, 0xE09E);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
/// Should skip the next instruction if key pressed does not have value Vx.
fn test_SKNP_Vx_opcode() {
    let mut cpu = CPU::default();

    load_and_execute_instruction(&mut cpu, 0xE0A1);
    assert_eq!(cpu.pc, 0x204);

    cpu.v[0] = 0xD;
    cpu.keypad.set_pressed(0xD);
    load_and_execute_instruction(&mut cpu, 0xE0A1);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
/// Should store the value of the delay timer into Vx.
fn test_LD_Vx_DT() {
    let mut cpu = CPU::default();

    cpu.delay_timer = 23;
    load_and_execute_instruction(&mut cpu, 0xF007);
    assert_eq!(cpu.v[0], 23);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
///  Should wait until a key is pressed and store the value in Vx.
fn test_LD_Vx_K() {
    let mut cpu = CPU::default();

    load_and_execute_instruction(&mut cpu, 0xf00a);
    assert_eq!(cpu.pc, 0x200);

    cpu.execute_instruction(0xf00a);
    assert_eq!(cpu.pc, 0x200);

    cpu.keypad.set_pressed(0xD);
    cpu.execute_instruction(0xf00a);
    assert_eq!(cpu.v[0], 0xD);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
/// Should store the value of Vx in the delay timer.
fn test_LD_DT_Vx() {
    let mut cpu = CPU::default();

    cpu.v[0] = 23;
    load_and_execute_instruction(&mut cpu, 0xF015);
    assert_eq!(cpu.v[0], 23);
    assert_eq!(cpu.pc, 0x202);
}

/// Should store the value of Vx in the delay timer.
fn test_LD_ST_Vx() {
    let mut cpu = CPU::default();

    cpu.v[0] = 23;
    load_and_execute_instruction(&mut cpu, 0xF018);
    assert_eq!(cpu.sound_timer, 23);
    assert_eq!(cpu.pc, 0x202);
}

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

#[test]
/// Should set I-register to the index of the hexadecimal sprite with value of Vx. In practice I = Vx * 5;
fn test_LD_F_Vx() {
    let mut cpu = CPU::default();

    cpu.v[0] = 0;
    load_and_execute_instruction(&mut cpu, 0xF029);
    assert_eq!(cpu.i, 0);
    assert_eq!(cpu.pc, 0x202);

    cpu.v[0] = 5;
    load_and_execute_instruction(&mut cpu, 0xF029);
    assert_eq!(cpu.i, 25);
}

#[test]
/// TODO -- Should store binary-coded decimal representation of Vx in memory at locations I, I + 1 and I + 2.
fn test_LD_B_Vx() {}

#[test]
/// Should store registers V0 to Vx in memory starting at location I.
fn test_LD__I__Vx() {
    let mut cpu = CPU::default();

    cpu.i = 0x300;

    cpu.v[0] = 0;
    cpu.v[1] = 23;
    cpu.v[6] = 16;
    cpu.v[8] = 9;
    cpu.v[12] = 4;
    cpu.v[15] = 255;

    load_and_execute_instruction(&mut cpu, 0xFF55);

    assert_eq!(cpu.memory[0x300], 0);
    assert_eq!(cpu.memory[0x301], 23);
    assert_eq!(cpu.memory[0x306], 16);
    assert_eq!(cpu.memory[0x308], 9);
    assert_eq!(cpu.memory[0x30C], 4);
    assert_eq!(cpu.memory[0x30F], 255);

    assert_eq!(cpu.pc, 0x202);
}

#[test]
/// Should read values of I to I+x into registers V0 to Vx.
fn test_LD_Vx__I__Vx() {
    let mut cpu = CPU::default();

    cpu.i = 0x300;
    cpu.memory[0x300] = 1;
    cpu.memory[0x301] = 9;
    cpu.memory[0x304] = 12;
    cpu.memory[0x306] = 3;
    cpu.memory[0x307] = 0;
    cpu.memory[0x309] = 45;
    cpu.memory[0x30C] = 200;
    cpu.memory[0x30F] = 7;

    load_and_execute_instruction(&mut cpu, 0xFF65);

    assert_eq!(cpu.v[0x0], 1);
    assert_eq!(cpu.v[0x1], 9);
    assert_eq!(cpu.v[0x4], 12);
    assert_eq!(cpu.v[0x6], 3);
    assert_eq!(cpu.v[0x7], 0);
    assert_eq!(cpu.v[0x9], 45);
    assert_eq!(cpu.v[0xC], 200);
    assert_eq!(cpu.v[0xF], 7);

    assert_eq!(cpu.pc, 0x202);
}
