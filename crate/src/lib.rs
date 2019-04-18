mod utils;

use wasm_bindgen::prelude::*;
use rand::Rng;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(raw_module = "../../js/io-interfaces/audio.js")]
extern "C" {
    type Audio;

    #[wasm_bindgen(constructor)]
    fn new() -> Audio;

    #[wasm_bindgen(method)]
    fn start(this: &Audio);

    #[wasm_bindgen(method)]
    fn stop(this: &Audio);

    #[wasm_bindgen(method)]
    fn is_active(this: &Audio) -> bool;
}

#[wasm_bindgen(raw_module = "../../js/io-interfaces/keyboard.js")]
extern "C" {
    type Keyboard;

    #[wasm_bindgen(constructor)]
    fn new() -> Keyboard;

    #[wasm_bindgen(method)]
    fn is_key_pressed(this: &Keyboard, key: u8) -> bool;
}

pub const CHIP8_DISPLAY_WIDTH: usize = 64;
pub const CHIP8_DISPLAY_HEIGHT: usize = 32;

const CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[wasm_bindgen]
/// Representation of the CHIP8 emulator.
pub struct Emulator {
    // CHIP-8 supports 35 opcodes each of them is two bytes long and represents some command
    // that CHIP-8 has to execute.
    opcode: u16,
    // I stands for index register, that usually has a pointer to the memory.
    i: usize,
    // CHIP-8 has 4096 bytes of memory. Program is loaded to the 0x200 address. Lower addresses are
    // used to store font used by the CHIP-8 interpreter.
    memory: [u8; 4096],
    // 16 one byte long registers. V0 to VE are used to store some data and VF is used to store
    // carry flag.
    v: [u8; 16],
    // Stack to save current PC when jumping to another subroutine.
    stack: Vec<usize>,
    // Program counter points to the current opcode position in memory.
    pc: usize,
    // CHIP-8 display nested array.
    gfx: [[u8; CHIP8_DISPLAY_WIDTH]; CHIP8_DISPLAY_HEIGHT],
    draw_flag: bool,
    delay_timer: u8,
    sound_timer: u8,
    audio: Audio,
    keyboard: Keyboard,
}

#[wasm_bindgen]
impl Emulator {
    /// Create new Emulator.
    pub fn new() -> Emulator {
        Emulator {
            pc: 0x200,
            i: 0x200,
            opcode: 0,
            stack: vec![],
            v: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            gfx: [[0; CHIP8_DISPLAY_WIDTH]; CHIP8_DISPLAY_HEIGHT],
            draw_flag: false,
            audio: Audio::new(),
            keyboard: Keyboard::new(),
            memory: Emulator::prepare_memory(),
        }
    }

    /// Resets emulator properties to their initial values.
    ///
    /// # Example
    ///
    /// ```
    /// use wasm_chip8::{Emulator, CHIP8_DISPLAY_HEIGHT, CHIP8_DISPLAY_WIDTH};
    /// let mut emulator = Emulator::new();
    /// emulator.tick();
    /// emulator.reset();
    /// assert_eq!(emulator.gfx(), [[0; CHIP8_DISPLAY_WIDTH]; CHIP8_DISPLAY_HEIGHT]);
    /// ```
    pub fn reset(&mut self) {
        self.pc = 0x200;
        self.i = 0x200;
        self.opcode = 0;
        self.stack = vec![];
        self.v = [0; 16];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.gfx = [[0; CHIP8_DISPLAY_WIDTH]; CHIP8_DISPLAY_HEIGHT];
        self.draw_flag = false;
        self.memory = Emulator::prepare_memory();
    }

    /// Return pointer to the gfx array of 64 u8 elements.
    pub fn gfx(&self) -> *const [u8; 64] { self.gfx.as_ptr() }

    /// Loads program to the emulator's memory.
    ///
    /// # Arguments
    ///
    /// * `program` - A slice of bytes (u8) that represents program code.
    ///
    /// # Example
    ///
    /// ```
    /// use wasm_chip8::Emulator;
    /// let mut emulator = Emulator::new();
    /// emulator.load(&[0xff, 0xf0, 0xfe]);
    /// ```
    pub fn load(&mut self, program: &[u8]) {
        for (i, &byte) in program.iter().enumerate() {
            self.memory[i + 0x200] = byte;
        }
    }

    /// Run one step ("tick") of the program.
    ///
    /// Loads opcode from memory, processes it and sets pointer to the next opcode.
    pub fn tick(&mut self) {
        self.opcode = self.get_opcode();

        let firstnib = (self.opcode >> 12) as u8;
        let nnn = self.opcode & 0x0fff;
        let nn = (self.opcode & 0x00ff) as u8;
        let n = (self.opcode & 0x000f) as u8;
        let x = ((self.opcode & 0x0f00) >> 8) as usize;
        let y = ((self.opcode & 0x00f0) >> 4) as usize;
        let vx = self.v[x];
        let vy = self.v[y];

        match firstnib {
            0x0 => match nn {
                0xe0 => self.clear_screen(),
                0xee => self.return_from_subroutine(),
                _ => self.next_opcode(),
            },
            0x1 => self.jump(nnn as usize),
            0x2 => self.call_subroutine(nnn as usize),
            0x3 => self.skip_eq(vx, nn),
            0x4 => self.skip_neq(vx, nn),
            0x5 => self.skip_eq(vx, vy),
            0x6 => self.set_v(x, nn),
            0x7 => self.add_to_v(x, nn),
            0x8 => match n {
                0x0 => self.set_v(x, vy),
                0x1 => self.set_v(x, vx | vy),
                0x2 => self.set_v(x, vx & vy),
                0x3 => self.set_v(x, vx ^ vy),
                0x4 => self.add_vx_vy(x, y),
                0x5 => self.sub_vx_vy(x, y),
                0x6 => self.shift_vx_right(x),
                0x7 => self.sub_vy_vx(x, y),
                0xe => self.shift_vx_left(x),
                _ => self.next_opcode(),
            },
            0x9 => self.skip_neq(vx, vy),
            0xa => self.set_i(nnn as usize),
            0xb => self.jump((nnn + u16::from(self.v[0])) as usize),
            0xc => self.set_v(x, nn & rand::thread_rng().gen::<u8>()),
            0xd => self.draw_sprite(vx, vy, n),
            0xe => match nn {
                0x9e => self.skip_key_pressed(vx),
                0xa1 => self.skip_key_not_pressed(vx),
                _ => self.next_opcode(),
            },
            0xf => match nn {
                0x07 => self.set_v(x, self.delay_timer),
                0x0a => self.wait_key(x),
                0x15 => self.set_delay_timer(vx),
                0x18 => self.set_sound_timer(vx),
                0x1e => self.set_i(self.i + usize::from(vx)),
                0x29 => self.set_i(usize::from(vx * 5)),
                0x33 => self.set_bcd(vx),
                0x55 => self.store_v(x),
                0x65 => self.fill_v(x),
                _ => self.next_opcode(),
            },
            _ => self.next_opcode(),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if !self.audio.is_active() {
                self.audio.start();
            }

            self.sound_timer -= 1;

            if self.sound_timer == 0 {
                self.audio.stop()
            }
        }
    }

    fn prepare_memory() -> [u8; 4096] {
        let mut memory = [0; 4096];
        for (i, &byte) in CHIP8_FONTSET.iter().enumerate() {
            memory[i] = byte
        }
        memory
    }

    fn get_opcode(&self) -> u16 { (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16) }

    fn next_opcode(&mut self) { self.pc += 2; }

    fn skip_opcode(&mut self) { self.pc += 4; }

    fn clear_screen(&mut self) {
        self.gfx = [[0; CHIP8_DISPLAY_WIDTH]; CHIP8_DISPLAY_HEIGHT];
        self.draw_flag = true;
        self.next_opcode();
    }

    fn return_from_subroutine(&mut self) { self.pc = self.stack.pop().unwrap(); }

    fn jump(&mut self, address: usize) { self.pc = address; }

    fn call_subroutine(&mut self, address: usize) {
        self.stack.push(self.pc + 2);
        self.pc = address;
    }

    fn skip_if(&mut self, cond: bool) {
        if cond {
            self.skip_opcode();
        } else {
            self.next_opcode();
        }
    }

    fn skip_eq(&mut self, a: u8, b: u8) {
        self.skip_if(a == b);
    }

    fn skip_neq(&mut self, a: u8, b: u8) {
        self.skip_if(a != b);
    }

    fn set_v(&mut self, x: usize, value: u8) {
        self.v[x] = value;
        self.next_opcode();
    }

    fn add_to_v(&mut self, x: usize, value: u8) {
        self.v[x] = self.v[x].overflowing_add(value).0;
        self.next_opcode();
    }

    fn add_vx_vy(&mut self, x: usize, y: usize) {
        self.v[0xf] = if self.v[y] > (0xff - self.v[x]) { 1 } else { 0 };
        self.v[x] = self.v[x].overflowing_add(self.v[y]).0;
        self.next_opcode();
    }

    fn sub_vx_vy(&mut self, x: usize, y: usize) {
        self.v[0xf] = if self.v[y] > self.v[x] { 0 } else { 1 };
        self.v[x] = self.v[x].overflowing_sub(self.v[y]).0;
        self.next_opcode();
    }

    fn sub_vy_vx(&mut self, x: usize, y: usize) {
        self.v[0xf] = if self.v[y] < self.v[x] { 0 } else { 1 };
        self.v[x] = self.v[y] - self.v[x];
        self.next_opcode();
    }

    fn shift_vx_right(&mut self, x: usize) {
        self.v[0xf] = self.v[x] & 0x0f;
        self.v[x] >>= 1;
        self.next_opcode();
    }

    fn shift_vx_left(&mut self, x: usize) {
        self.v[0xf] = self.v[x] & 0xf0;
        self.v[x] <<= 1;
        self.next_opcode();
    }

    fn set_i(&mut self, value: usize) {
        self.i = value;
        self.next_opcode();
    }

    fn draw_sprite(&mut self, vx: u8, vy: u8, height: u8) {
        let from = self.i;
        let to = self.i + height as usize;
        let sprite = &self.memory[from..to];

        let mut flipped: u8 = 0;

        for y in 0..sprite.len() {
            for x in 0..8 {
                if sprite[y] & (0x80 >> x) != 0 {
                    let mut y = (vy + (y) as u8) as usize;
                    let mut x = (vx + x) as usize;

                    if y >= 32 {
                        y = 31;
                    }

                    if x >= 64 {
                        x = 63;
                    }

                    if self.gfx[y][x] == 1 {
                        flipped = 1;
                    }

                    self.gfx[y][x] ^= 1;
                }
            }
        }

        self.v[0xf] = flipped;
        self.draw_flag = true;
        self.next_opcode();
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        self.keyboard.is_key_pressed(key)
    }

    fn skip_key_pressed(&mut self, key: u8) {
        self.skip_if(self.is_key_pressed(key));
    }

    fn skip_key_not_pressed(&mut self, key: u8) {
        self.skip_if(!self.is_key_pressed(key));
    }

    fn wait_key(&mut self, x: usize) {
        for i in 0..16 {
            if self.is_key_pressed(i) {
                self.v[x] = i;
                self.next_opcode();
                break;
            }
        }
    }

    fn set_delay_timer(&mut self, value: u8) {
        self.delay_timer = value;
        self.next_opcode();
    }

    fn set_sound_timer(&mut self, value: u8) {
        self.sound_timer = value;
        self.next_opcode();
    }

    fn set_bcd(&mut self, vx: u8) {
        self.memory[self.i] = vx / 100;
        self.memory[self.i + 1] = (vx / 10) % 10;
        self.memory[self.i + 2] = (vx % 100) % 10;
        self.next_opcode();
    }

    fn store_v(&mut self, x: usize) {
        for i in 0..=x {
            self.memory[self.i + i] = self.v[i];
        }
        self.next_opcode();
    }

    fn fill_v(&mut self, x: usize) {
        for i in 0..=x {
            self.v[i] = self.memory[self.i + i];
        }
        self.next_opcode();
    }
}
