use std::fs::File;
use std::thread::sleep;
use std::time::Duration;

use std::io:: { Error, Read, stdout };
use termion:: {
    async_stdin,
    input::TermRead,
    raw::IntoRawMode,
};

mod keyboard; use keyboard::Keyboard;
mod display; use display::Display;
mod cpu; use cpu::Chip8;


fn main() -> Result<(), Error> {
    let mut f = File::open("roms/test_opcode.ch8").unwrap();

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();
    
    let mut keyboard = Keyboard::new(async_stdin());
    let mut display = Display::new();
    let mut cpu = Chip8::new();

    cpu.load_rom(buffer);
    
    
    
    loop {
        let state = keyboard.tick()?;
        cpu.tick(state, &mut display);
        sleep(Duration::from_millis(1000/30));
    }


    Ok(())
}

