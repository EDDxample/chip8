use std::io::Read;
use std::fs::File;

fn main() {
    let mut f = File::open("roms/IBM Logo.ch8").unwrap();

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    
    let mut display = Display { pixels: [[' '; 64]; 32] };
    let mut cpu: Chip8 = Chip8 {
        ram: [0; 4096],
        stack: [0; 16],
        stack_ptr: 0,
        i: 0,
        v: [0; 16],
        pc: 0x200,
    
        sound_timer: 0,
        delay_timer: 0,  
    };

    cpu.load_rom(buffer);
    cpu.run(&mut display);

}

struct Chip8 {

    // Memory
    //   $000 -> $1FF: Chip Logic
    //   $200 -> $FFF: Program Data
    pub ram: [u8; 4096], // 4kb

    // Stack
    //   Used for nested function calls
    pub stack: [u16; 16],
    pub stack_ptr: u8,
    
    pub i: u16,
    pub v: [u8; 16], // registers
    pub pc: u16,

    // Timers: decrease at 60hz until the value is 0
    pub sound_timer: u8, // plays a beep
    pub delay_timer: u8, // pauses execution
}

impl Chip8 {

    pub fn get_opcode(&mut self) -> u16 {
        (self.ram[self.pc as usize] as u16) << 8 | (self.ram[1 + self.pc as usize] as u16)
    }


    // $000 -> $1FF: Chip Logic
    // $200 -> $FFF: Program Data
    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for i in 0..rom.len() {
            self.ram[0x200 + i] = *rom.get(i).unwrap();
        }
    }

    pub fn run(&mut self, display: &mut Display) {

        while true {

            if self.pc as usize >= self.ram.len() {
                println!("out of bounds: ${:03X}", self.pc);
                return;
            }

            if self.pc == 0x228 {
                println!("halt!");
                return;
            }

            let opcode: u16 = self.get_opcode();
            print!("${:03X}: {:04X} - ", self.pc, opcode);
            self.pc += 2;
            
            self.tick(opcode, display);
        }
    }

    pub fn tick(&mut self, opcode: u16, display: &mut Display) -> bool {

        match opcode & 0xF000 {
            0 => {
                if opcode == 0x00E0 {
                    println!("clear screen");
                    display.clear();
                }
            },
            // JMP
            0x1000 => {
                let addr = opcode & 0x0FFF;
                println!("move pc to ${:03X}", addr);
                self.pc = addr;
            },
            // set VX to NN
            0x6000 => {
                let x = opcode >> 8 & 0xF;
                let value: u8 = (opcode & 0x00FF) as u8;
                self.v[x as usize] += value;
                println!("set {} to register V{} - {:?}", value, x, self.v);
            }
            // add NN to VX 
            0x7000 => {
                let x = opcode >> 8 & 0xF;
                let value: u8 = (opcode & 0x00FF) as u8;
                self.v[x as usize] += value;
                println!("add {} to register V{} - {:?}", value, x, self.v);
            }
            // set I to the addres NNN
            0xA000 => {
                let addr = opcode & 0x0FFF;
                self.i = addr;
                println!("store address ${:03X} at register I", self.i);
            }
            0xD000 => {
                let vx = opcode >> 8 & 0xF;
                let vy = opcode >> 4 & 0xF;
                let n  = opcode & 0xF;
                println!("draw from registers V{} V{} ({}, {}) sprite of {} bytes at I: ${:03X}", vx, vy, self.v[vx as usize], self.v[vy as usize], n, self.i);
                
                let pix_x = self.v[vx as usize];
                let pix_y = self.v[vy as usize];

                for ii in 0..n as usize {

                    let pix_byte = self.ram[ii + self.i as usize];
                    
                    for i in 0..8 {
                        let bit = pix_byte & (1 << (7 - i));
                        display.set_pixel(pix_x + i, pix_y + ii as u8, bit > 0);
                    }
                }

                display.show();
            }
            _ => { println!("  x  not implemented yet"); },
        }

        return true;
            
    }
}

struct Display {
    pixels: [[char; 64]; 32],
}

impl Display {
    pub fn show(&mut self) {
        println!("+----------------------------------------------------------------+");
        for line in self.pixels.iter() {
            print!("|");
            for pixel in line { print!("{}", pixel); }
            println!("|");
        }
        println!("+----------------------------------------------------------------+");
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, value: bool) {
        self.pixels[y as usize % 32][x as usize % 64] = if value { '█' } else { ' ' };
    }

    pub fn clear(&mut self) {
        for x in 0..64 {
            for y in 0..32 {
                self.pixels[y][x] = ' ';
            }
        }
    }
}
