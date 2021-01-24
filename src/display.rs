use std::io::{stdout, Write};

use termion:: {
    raw::IntoRawMode,
};


pub struct Display {
    pixels: [[char; 64]; 32],
}

impl Display {
    pub fn new() -> Self {
        Display { pixels: [[' '; 64]; 32] }
    }
    #[allow(unused_must_use)]
    pub fn show(&mut self) {
        let mut _stdout = stdout().into_raw_mode().unwrap();
        let mut screen = String::with_capacity((64+2+2)*(32+2));
        for line in self.pixels.iter() {
            screen += "|";
            for pixel in line { screen.push(*pixel) }
            screen += "|\n\r";
        }
        write!(
            _stdout, "{}{}+----------------------------------------------------------------+\n\r{}+----------------------------------------------------------------+\n\r",
            termion::clear::All,
            termion::cursor::Goto(1,1),
            screen,
        );
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