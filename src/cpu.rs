use std::{fs::File, io::Read, path::Path};

use crate::{
    audio::Audio,
    input::Keypad,
    video::{Display, FONT_SET},
};
use rand::{self, Rng};
use sdl2::{event::Event, EventPump, Sdl, TimerSubsystem};

const OPCODE_LENGTH: usize = 2;
const SPRITE_LENGTH: u16 = 5;

const CLOCK_SPEED: f32 = 500.0;
const RAM_SIZE: usize = 4096;
const ROM_SIZE: usize = 3584;
const PROGRAM_START: usize = 0x200;

pub struct CPU {
    ram: [u8; RAM_SIZE],
    pc: usize,

    v: [u8; 16],
    i: u16,

    stack: [usize; 16],
    sp: usize,

    keypad: Keypad,
    key_register: Option<usize>,

    audio: Audio,
    display: Display,

    timer: Timer,
    events: EventPump,
}

struct Timer {
    timer: TimerSubsystem,
    last_tick: u32,
}

impl Timer {
    fn new(timer: TimerSubsystem) -> Self {
        Timer {
            timer,
            last_tick: 0,
        }
    }

    fn skip(&mut self) -> bool {
        let tick = self.timer.ticks();
        let delta = (tick - self.last_tick) as f32;

        if delta >= 1000.0 / CLOCK_SPEED {
            self.last_tick = tick;
            false
        } else {
            true
        }
    }
}

impl CPU {
    pub fn new(sdl: &Sdl, rom_path: &str) -> Self {
        let timer = sdl.timer().unwrap();
        let events = sdl.event_pump().unwrap();

        let mut ram = [0u8; RAM_SIZE];

        for i in 0..FONT_SET.len() {
            ram[i] = FONT_SET[i]
        }

        let pat = Path::new(rom_path);
        let d = pat.display();
        println!("{}", d);

        let mut rom_file = File::open(rom_path).expect("File not found.");
        let mut rom = [0u8; ROM_SIZE];
        let _ = rom_file.read(&mut rom);

        for (i, &byte) in rom.iter().enumerate() {
            let address = PROGRAM_START + i;

            if address < RAM_SIZE {
                ram[address] = byte
            } else {
                break;
            }
        }

        CPU {
            ram,
            pc: PROGRAM_START,
            v: [0; 16],
            i: 0,
            stack: [0; 16],
            sp: 0,
            keypad: Keypad::new(),
            key_register: None,
            audio: Audio::new(sdl),
            display: Display::new(sdl),
            timer: Timer::new(timer),
            events,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.display.draw();

        for event in self.events.poll_iter() {
            self.keypad.listen(&event);

            if let Event::Quit { .. } = event {
                return false;
            }
        }

        if self.timer.skip() {
            return true;
        }

        if self.pc >= RAM_SIZE {
            return true;
        }

        if let Some(key_register) = self.key_register {
            if let Some(last_released) = self.keypad.consume_last_released() {
                self.v[key_register] = last_released;
                self.key_register = None
            }
        } else {
            self.audio.tick();

            let opcode = (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16);
            self.execute(opcode)
        }

        true
    }

    fn execute(&mut self, opcode: u16) {
        if opcode > 0 {
            println!("{:#06x}", opcode);
        }

        let nibble = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        let x = nibble.1 as usize;
        let y = nibble.2 as usize;

        let n = nibble.3;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match nibble {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(),
            (0x0, _, _, _) => self.op_0nnn(nnn),
            (0x1, _, _, _) => self.op_1nnn(nnn),
            (0x2, _, _, _) => self.op_2nnn(nnn),
            (0x3, _, _, _) => self.op_3xkk(x, nn),
            (0x4, _, _, _) => self.op_4xkk(x, nn),
            (0x5, _, _, 0x0) => self.op_5xy0(x, y),
            (0x6, _, _, _) => self.op_6xkk(x, nn),
            (0x7, _, _, _) => self.op_7xkk(x, nn),
            (0x8, _, _, 0x0) => self.op_8xy0(x, y),
            (0x8, _, _, 0x1) => self.op_8xy1(x, y),
            (0x8, _, _, 0x2) => self.op_8xy2(x, y),
            (0x8, _, _, 0x3) => self.op_8xy3(x, y),
            (0x8, _, _, 0x4) => self.op_8xy4(x, y),
            (0x8, _, _, 0x5) => self.op_8xy5(x, y),
            (0x8, _, _, 0x6) => self.op_8xy6(x, y),
            (0x8, _, _, 0x7) => self.op_8xy7(x, y),
            (0x8, _, _, 0xE) => self.op_8xye(x, y),
            (0x9, _, _, 0x0) => self.op_9xy0(x, y),
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xB, _, _, _) => self.op_bnnn(nnn),
            (0xC, _, _, _) => self.op_cxkk(x, nn),
            (0xD, _, _, _) => self.op_dxyn(x, y, n),
            (0xE, _, 0x9, 0xE) => self.op_ex9e(x),
            (0xE, _, 0xA, 0x1) => self.op_exa1(x),
            (0xF, _, 0x0, 0x7) => self.op_fx07(x),
            (0xF, _, 0x0, 0xA) => self.op_fx0a(x),
            (0xF, _, 0x1, 0x5) => self.op_fx15(x),
            (0xF, _, 0x1, 0x8) => self.opfx18(x),
            (0xF, _, 0x1, 0xE) => self.op_fx1e(x),
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),
            (0xF, _, 0x3, 0x3) => self.op_fx33(x),
            (0xF, _, 0x5, 0x5) => self.op_fx55(x),
            (0xF, _, 0x6, 0x5) => self.op_fx65(x),
            _ => panic!("Unknown opcode {:x?}", opcode),
        }
    }

    fn next(&mut self) {
        self.pc += OPCODE_LENGTH
    }

    fn skip_if(&mut self, condition: bool) {
        if condition {
            self.pc += OPCODE_LENGTH * 2
        } else {
            self.pc += OPCODE_LENGTH
        }
    }

    /// 0nnn - SYS addr
    ///
    /// Jump to a machine code routine at nnn.
    fn op_0nnn(&mut self, _nnn: u16) {
        self.next()
    }

    /// 00E0 - CLS
    ///
    /// Clear the display.
    fn op_00e0(&mut self) {
        self.display.clear();
        self.next()
    }

    /// 00EE - RET
    ///
    /// Return from a subroutine.
    /// - Set the program counter to the address at the top of the stack
    /// - Subtracts 1 from the stack pointer.
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp]
    }

    /// 1nnn - JP addr
    ///
    /// Jump to location nnn.
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn as usize
    }

    /// 2nnn - CALL addr
    ///
    /// Call subroutine at nnn.
    /// - Increment the stack pointer
    /// - Put the current PC on the top of the stack
    /// - Set PC to `nnn`.
    fn op_2nnn(&mut self, nnn: u16) {
        self.stack[self.sp] = self.pc + OPCODE_LENGTH;
        self.sp += 1;
        self.pc = nnn as usize
    }

    /// 3xkk - SE Vx, byte
    ///
    /// Skip next instruction if Vx = kk.
    fn op_3xkk(&mut self, x: usize, kk: u8) {
        self.skip_if(self.v[x] == kk)
    }

    /// 4xkk - SNE Vx, byte
    ///
    /// Skip next instruction if Vx != kk.
    fn op_4xkk(&mut self, x: usize, kk: u8) {
        self.skip_if(self.v[x] != kk)
    }

    /// 5xy0 - SE Vx, Vy
    ///
    /// Skip next instruction if Vx = Vy.
    fn op_5xy0(&mut self, x: usize, y: usize) {
        self.skip_if(self.v[x] == self.v[y])
    }

    /// 6xkk - LD Vx, byte
    ///
    /// Set Vx = kk.
    fn op_6xkk(&mut self, x: usize, kk: u8) {
        self.v[x] = kk;
        self.next()
    }

    /// 7xkk - ADD Vx, byte
    ///
    /// Set Vx = Vx + kk.
    fn op_7xkk(&mut self, x: usize, kk: u8) {
        let vx = self.v[x];
        self.v[x] = vx.wrapping_add(kk);
        self.next()
    }

    /// 8xy0 - LD Vx, Vy
    ///
    /// Set Vx = Vy.
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
        self.next()
    }

    /// 8xy1 - OR Vx, Vy
    ///
    /// Set Vx = Vx OR Vy.
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
        self.next()
    }

    /// 8xy2 - AND Vx, Vy
    ///
    /// Set Vx = Vx AND Vy.
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
        self.next()
    }

    /// 8xy3 - XOR Vx, Vy
    ///
    /// Set Vx = Vx XOR Vy.
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
        self.next()
    }

    /// 8xy4 - ADD Vx, Vy
    ///
    /// Set Vx = Vx + Vy, set VF = carry.
    /// - The values of Vx and Vy are added together.
    /// - If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0.
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let vx = (self.v[x] as u16) + (self.v[y] as u16);
        self.v[x] = vx as u8;
        self.v[0xF] = (vx > 0xFF) as u8;
        self.next()
    }

    /// 8xy5 - SUB Vx, Vy
    ///
    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    /// - If Vx > Vy, then VF is set to 1, otherwise 0.
    /// - Then Vy is subtracted from Vx, and the results stored in Vx.
    fn op_8xy5(&mut self, x: usize, y: usize) {
        let vx = self.v[x];
        let vy = self.v[y];
        self.v[0xF] = (vx > vy) as u8;
        self.v[x] = vx.wrapping_sub(vy);
        self.next()
    }

    /// 8xy6 - SHR Vx {, Vy}
    ///
    /// Set Vx = Vx SHR 1.
    /// - If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
    /// - Then Vx is divided by 2.
    fn op_8xy6(&mut self, x: usize, _y: usize) {
        let vx = self.v[x];
        self.v[0xF] = vx & 1;
        self.v[x] = vx >> 1;
        self.next()
    }

    /// 8xy7 - SUBN Vx, Vy
    ///
    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    /// - If Vy > Vx, then VF is set to 1, otherwise 0.
    /// - Then Vx is subtracted from Vy, and the results stored in Vx.
    fn op_8xy7(&mut self, x: usize, y: usize) {
        let vx = self.v[x];
        let vy = self.v[y];
        self.v[0xF] = (vy > vx) as u8;
        self.v[x] = vy.wrapping_sub(vx);
        self.next()
    }

    /// 8xyE - SHL Vx {, Vy}
    ///
    /// Set Vx = Vx SHL 1.
    /// - If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0.
    /// - Then Vx is multiplied by 2.
    fn op_8xye(&mut self, x: usize, _y: usize) {
        let vx = self.v[x];
        self.v[0xF] = (vx & 0b10000000) >> 7;
        self.v[x] = vx << 1;
        self.next()
    }

    /// 9xy0 - SNE Vx, Vy
    ///
    /// Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self, x: usize, y: usize) {
        self.skip_if(self.v[x] != self.v[y])
    }

    /// Annn - LD I, addr
    ///
    /// Set I = nnn.
    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
        self.next()
    }

    /// Bnnn - JP V0, addr
    ///
    /// Jump to location nnn + V0.
    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = (nnn as usize) + (self.v[0x0] as usize)
    }

    /// Cxkk - RND Vx, byte
    ///
    /// Set Vx = random byte AND kk.
    /// - Generate a random number from 0 to 255.
    /// - Set Vx with the result AND kk
    fn op_cxkk(&mut self, x: usize, kk: u8) {
        self.v[x] = rand::thread_rng().gen::<u8>() & kk;
        self.next()
    }

    /// Dxyn - DRW Vx, Vy, nibble
    ///
    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) {
        let mut vf = false;
        let i = self.i as usize;

        let vx = self.v[x] as usize;
        let vy = self.v[y] as usize;

        for yline in 0..(n as usize) {
            let sy = vy + yline;
            let pixels = self.ram[i + yline];

            for xline in 0..8 {
                let sx = vx + xline;
                let pixel = (pixels >> (7 - xline)) & 1;
                vf |= self.display.set(sx, sy, pixel == 1);
            }
        }

        self.v[0xF] = vf as u8;
        self.next()
    }

    /// Ex9E - SKP Vx
    ///
    /// Skip next instruction if key with the value of Vx is pressed.
    fn op_ex9e(&mut self, x: usize) {
        self.skip_if(self.keypad.pressed[self.v[x] as usize])
    }

    /// ExA1 - SKNP Vx
    ///
    /// Skip next instruction if key with the value of Vx is not pressed.
    fn op_exa1(&mut self, x: usize) {
        self.skip_if(!(self.keypad.pressed[self.v[x] as usize]))
    }

    /// Fx07 - LD Vx, DT
    ///
    /// Set Vx = delay timer value.
    fn op_fx07(&mut self, x: usize) {
        self.v[x] = self.audio.delay;
        self.next()
    }

    /// Fx0A - LD Vx, K
    ///
    /// Wait for a key press, store the value of the key in Vx.
    fn op_fx0a(&mut self, x: usize) {
        self.key_register = Some(x);
        self.next()
    }

    /// Fx15 - LD DT, Vx
    ///
    /// Set delay timer = Vx.
    fn op_fx15(&mut self, x: usize) {
        self.audio.delay = self.v[x];
        self.next()
    }

    /// Fx18 - LD ST, Vx
    ///
    /// Set sound timer = Vx.
    fn opfx18(&mut self, x: usize) {
        self.audio.sound = self.v[x];
        self.next()
    }

    /// Fx1E - ADD I, Vx
    ///
    /// Set I = I + Vx.
    fn op_fx1e(&mut self, x: usize) {
        self.i += self.v[x] as u16;
        self.next()
    }

    /// Fx29 - LD F, Vx
    ///
    /// Set I = location of sprite for digit Vx.
    fn op_fx29(&mut self, x: usize) {
        self.i = (self.v[x] as u16) * SPRITE_LENGTH;
        self.next();
    }

    /// Fx33 - LD B, Vx
    ///
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    /// - Take the decimal value of Vx.
    /// - Place the hundreds digit in memory at location in I.
    /// - Place the tens digit at location I+1.
    /// - Place the ones digit at location I+2.
    fn op_fx33(&mut self, x: usize) {
        let vx = self.v[x];
        let i = self.i as usize;
        self.ram[i] = vx / 100;
        self.ram[i + 1] = (vx % 100) / 10;
        self.ram[i + 2] = vx % 10;
        self.next()
    }

    /// Fx55 - LD [I], Vx
    ///
    /// Store registers V0 through Vx in memory starting at location I.
    fn op_fx55(&mut self, x: usize) {
        let i = self.i as usize;

        for vi in 0..=x {
            self.ram[i + vi] = self.v[vi]
        }

        self.next()
    }

    /// Fx65 - LD Vx, [I]
    ///
    /// Read registers V0 through Vx from memory starting at location I.
    fn op_fx65(&mut self, x: usize) {
        let i = self.i as usize;

        for vi in 0..=x {
            self.v[vi] = self.ram[i + vi]
        }

        self.next()
    }
}
