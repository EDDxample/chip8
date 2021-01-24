
pub struct Display {
    pixels: [[char; 64]; 32],
}

impl Display {
    pub fn new() -> Self {
        Display { pixels: [[' '; 64]; 32] }
    }

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