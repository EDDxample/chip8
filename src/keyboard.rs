use std::io:: { Error, ErrorKind, Read, stdout };
use termion:: {
    AsyncReader,
    input::TermRead,
    raw::IntoRawMode,
    event::Key,
};

pub struct Keyboard {
    astdin: AsyncReader, // asynchronous thread to handle input
}

impl Keyboard {
    pub fn new(stdin: AsyncReader) -> Self {
        Keyboard { astdin: stdin }
    }

    pub fn tick(&mut self) -> Result<[bool; 16], Error> {
        let _stdout = stdout().into_raw_mode().unwrap();
        let mut state = [false; 16];
        
        loop { // loop until there are no valid keys
            let k = self.astdin.by_ref().keys().nth(0);
            let index = self.key2index(k);
            
            // errors
            if index == 0xEE { return Err(Error::new(ErrorKind::Interrupted, "exit CHIP8")) }
            if index == 0x11 { return Err(Error::new(ErrorKind::InvalidData, "invalid key")) }
            if index == 0x10 { break }

            state[index] = true;
        }

        Ok(state)
        
    }

    fn key2index(&mut self, keyopt: Option<Result<Key, Error>>) -> usize {
        match keyopt {
            None => 0x10, // stop
            Some(keyres) => match keyres {
                Err(_)  => 0x11, // InvalidData
                Ok(key) => match key {
                    Key::Ctrl('c') => 0xEE, // EXIT
                    Key::Char('1') => 0x0,
                    Key::Char('2') => 0x1,
                    Key::Char('3') => 0x2,
                    Key::Char('4') => 0x3,
                    Key::Char('q') => 0x4,
                    Key::Char('w') => 0x5,
                    Key::Char('e') => 0x6,
                    Key::Char('r') => 0x7,
                    Key::Char('a') => 0x8,
                    Key::Char('s') => 0x9,
                    Key::Char('d') => 0xa,
                    Key::Char('f') => 0xb,
                    Key::Char('z') => 0xc,
                    Key::Char('x') => 0xd,
                    Key::Char('c') => 0xe,
                    Key::Char('v') => 0xf,
                    _ => 0x10, // stop
                },
            },
        }
    }
}