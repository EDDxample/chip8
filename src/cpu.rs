use crate::display::Display;

pub struct Chip8 {

    // Memory
    //   $000 -> $1FF: Chip logic
    //   $200 -> $FFF: Program Data
    ram: [u8; 4096], // 4kb

    // Stack
    //   Used for nested function calls
    stack: [u16; 16],
    stack_ptr: u8,

    i: u16,
    v: [u8; 16], // registers
    pc: u16,

    // Timers: decrease at 60hz until the value is 0
    sound_timer: u8, // plays a beep
    delay_timer: u8, // pauses execution

    wait_for_key: bool,
    key_reg: usize,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut temp = Chip8 {
            ram: [0; 4096],
            stack: [0; 16],
            stack_ptr: 0,
            i: 0,
            v: [0; 16],
            pc: 0x200,
        
            sound_timer: 0,
            delay_timer: 0,
            wait_for_key: false,
            key_reg: 0,
        };

        temp.load_font();
        
        temp
    }

    // $000 -> $1FF: Chip logic
    // $200 -> $FFF: Program Data
    pub fn load_font(&mut self) {
        let font = [
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
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];

        for i in 0..font.len() {
            self.ram[0x050 + i] = *font.get(i).unwrap();
        }
    }
    
    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for i in 0..rom.len() {
            self.ram[0x200 + i] = *rom.get(i).unwrap();
        }
    }
    

    pub fn get_opcode(&mut self) -> u16 {
        (self.ram[self.pc as usize] as u16) << 8 | (self.ram[1 + self.pc as usize] as u16)
    }

    pub fn tick(&mut self, kbd_state: [bool; 16], display: &mut Display) {

        if self.pc as usize >= self.ram.len() {
            println!("out of bounds: ${:03X}", self.pc);
            return;
        }

        if self.wait_for_key {
            let mut key_pressed = false;
            for i in 0..16 {
                key_pressed = kbd_state[i];
                if key_pressed {
                    self.v[self.key_reg] = i as u8;
                    break
                }
            }
            if key_pressed {
                self.wait_for_key = false;
            } else { return }
        }
        
        let opcode: u16 = self.get_opcode();
        // print!("${:03X}: {:04X} - ", self.pc, opcode);
        self.pc += 2;
        
        self.process_opcode(opcode, kbd_state, display);
    }

    pub fn process_opcode(&mut self, opcode: u16, kbd_state: [bool; 16], display: &mut Display) -> bool {
        // different ways to read the opcode
        let addr = opcode & 0x0FFF;
        let nn   = (opcode & 0x00FF) as u8;
        let n    = opcode & 0x000F;
        let x    = (opcode >> 8 & 0xF) as usize;
        let y    = (opcode >> 4 & 0xF) as usize;

        
        let old_shifts = false; // if true, sets VX = VY before shifting in 8XY6 and 8XYE opcodes
        let old_bnnn = false; // if true, sets Jumps to NNN + the value of V0, otherwise it uses VX
        let log = false; // prints stuff


        match opcode & 0xF000 {
            0 => {
                // clear screen
                if opcode == 0x00E0 {
                    if log { println!("clear screen") };
                    display.clear();

                // return from subroutine
                } else if opcode == 0x00EE {
                    self.stack_ptr -= 1;
                    self.pc = self.stack[self.stack_ptr as usize];
                    if log { println!("return from subroutine to ${:03X} - {:?}", self.pc+2, self.stack) };
                }
            },
            // JMP
            0x1000 => {
                if log { println!("move pc to ${:03X}", addr) };
                self.pc = addr;
            },
            // call subroutine
            0x2000 => {
                self.stack[self.stack_ptr as usize] = self.pc;
                self.stack_ptr += 1;
                self.pc = addr;
                if log { println!("move to subroutine at ${:03X} - {:?}", self.pc, self.stack) };
            }
            // skip if VX == NN
            0x3000 => {
                if self.v[x] == nn as u8 { self.pc += 2; }
            }
            // skip if VX != NN
            0x4000 => {
                if self.v[x] != nn as u8 { self.pc += 2; }
            }
            // skip if VX == VY
            0x5000 => {
                if self.v[x] == self.v[y] { self.pc += 2; }
            }
            // set VX to NN
            0x6000 => {
                self.v[x] = nn as u8;
                if log { println!("set {} to register V{} - {:?}", nn, x, self.v) };
            }
            // add NN to VX 
            0x7000 => {
                self.v[x] = nn.overflowing_add(self.v[x]).0;
                if log { println!("add {} to register V{} - {:?}", nn, x, self.v) };
            }
            // arithmetic
            0x8000 => {
                match n {
                    // VX = VY
                    0x0 => self.v[x] = self.v[y],
                    // VX |= VY
                    0x1 => self.v[x] |= self.v[y],
                    // VX &= VY
                    0x2 => self.v[x] &= self.v[y],
                    // VX ^= VY
                    0x3 => self.v[x] ^= self.v[y],

                    // VX += VY       (VF = 1 when carry, 0 otherwise)
                    0x4 => {
                        let op = self.v[x].overflowing_add(self.v[y]);
                        self.v[x] = op.0;
                        self.v[0xF] = op.1 as u8;
                    }
                    // VX -= VY       (VF = 1 when borrow, 0 otherwise)
                    0x5 => {
                        let minuend = self.v[x];
                        let subtrahend = self.v[y];
                        let flag = minuend > subtrahend;
                        self.v[x] = minuend.overflowing_sub(subtrahend).0;
                        self.v[0xF] = flag as u8;
                        
                    }
                    // VX >>= 1       (VF is set to the deleted bit)
                    0x6 => {
                        if log { println!("shifting! might need to change the old_shift flag to make it work properly") };
                        if old_shifts { self.v[x] = self.v[y] }
                        self.v[0xF] = self.v[x] & 1;
                        self.v[x] >>= 1;
                    }
                    // VX = VY - VX   (VF = 1 when borrow, 0 otherwise)
                    0x7 => {
                        let minuend = self.v[y];
                        let subtrahend = self.v[x];
                        let flag = minuend > subtrahend;
                        self.v[x] = minuend.overflowing_sub(subtrahend).0;
                        self.v[0xF] = flag as u8;
                    }
                    // VX <<= 1       (VF is set to the deleted bit)
                    0xE => {
                        if log { println!("shifting! might need to change the old_shift flag to make it work properly") };
                        if old_shifts { self.v[x] = self.v[y] }
                        self.v[0xF] = self.v[x] >> 7 & 1;
                        self.v[x] <<= 1;
                    }
                    _ => { println!("  x  opcode 0x8??? not implemented yet"); },
                }
            }
            // skip if VX != VY
            0x9000 => {
                if self.v[x] != self.v[y] { self.pc += 2; }
            }
            // set I to the address NNN
            0xA000 => {
                self.i = addr;
                if log { println!("store address ${:03X} at register I", self.i) };
            }
            0xB000 => {
                self.pc = addr + if old_bnnn { self.v[0] } else { self.v[x] } as u16;
            }
            // VX = Random & NN
            0xC000 => {
                self.v[x] = 7 & nn;
            }
            // draw sprite of n bytes from address at I at display's vx vy
            0xD000 => {
                if log { println!("draw from registers V{} V{} ({}, {}) sprite of {} bytes at I: ${:03X}", x, y, self.v[x], self.v[y], n, self.i) };
                
                let pix_x = self.v[x];
                let pix_y = self.v[y];

                for ii in 0..n as usize {

                    let pix_byte = self.ram[ii + self.i as usize];
                    
                    for i in 0..8 {
                        let bit = pix_byte & (1 << (7 - i));
                        display.set_pixel(pix_x + i, pix_y + ii as u8, bit > 0);
                    }
                }

                display.show();
            }
            0xE000 => {
                match nn {
                    // skip if pressed
                    0x9E => { if  kbd_state[self.v[x] as usize] { self.pc += 2 } }
                    // skip if not pressed
                    0xA1 => { if !kbd_state[self.v[x] as usize] { self.pc += 2 } }
                    _ => { println!("  x  invalid keyop EX{}", nn); }
                }

            }
            0xF000 => {
                match nn {
                    0x07 => { self.v[x] = self.delay_timer }
                    0x0A => { self.wait_for_key = true; self.key_reg = x; } // halt until keypress
                    0x15 => { self.delay_timer = self.v[x] }
                    0x18 => { self.sound_timer = self.v[x] }
                    0x1E => { self.i = self.i.overflowing_add(self.v[x] as u16).0; }
                    0x29 => { self.i = 0x050 + (self.v[x] as u16) * 5 } // I = sprite_addr[Vx]
                    0x33 => { // set_BCD(Vx);
                        let vx = self.v[x];
                        let mut digit = 100;
                        // println!("==F {} 33==\nsaving {}", x, vx);
                        for ii in 0..3 {
                            self.ram[ii + self.i as usize] = ((vx / digit) % 10) as u8;
                            digit /= 10;
                        }
                        // println!("{} {} {}", self.ram[self.i as usize + 0], self.ram[self.i as usize + 1], self.ram[self.i as usize + 2]);
                    } 
                    0x55 => { // reg_dump(Vx,&I)
                        // println!("==F {} 55==\n{:?}", x, self.v);
                        for ii in 0..x+1 {
                            self.ram[ii + self.i as usize] = self.v[ii];
                            // print!("{},", self.ram[ii + self.i as usize]);
                        }
                        // println!();

                    }
                    0x65 => { // reg_load(Vx,&I)
                        for ii in 0..x+1 {
                            self.v[ii] = self.ram[ii + self.i as usize];
                        }
                     } 
                    _ => {  }
                }
            }
            _ => { println!("  x  not implemented yet"); },
        }
        return true; 
    }
}