use failure;
use std::{io, process, io::{BufRead, Write}};

use bar::{write_space, StatusBar};

#[derive(Debug, Deserialize)]
pub struct StdinConfig {
    length: u32,
}

/// A statusbar for cpu information. All data is gathered from /proc/stat and /proc/cpuinfo.
#[derive(Debug)]
pub struct Stdin {
    length: u32,
    char_width: u32,
    lspace: u32,
    rspace: u32,
}

impl Stdin {
    pub fn from_config(config: &StdinConfig, char_width: u32) -> Result<Stdin, failure::Error> {
        Ok(Stdin {
            length: config.length,
            char_width: char_width,
            lspace: 0,
            rspace: 0,
        })
    }
}

impl StatusBar for Stdin {
    fn run(&self, w: &mut process::ChildStdin) -> Result<(), failure::Error> {
        let stdin = io::stdin();
        let handle = stdin.lock();

        for line in handle.lines() {
            let line = line?;
            write_space(w, self.lspace)?;
            w.write(line.as_bytes())?;
            write_space(w, self.rspace)?;
            w.write(b"\n")?;
        }

        Ok(())
    }

    fn len(&self) -> u32 {
        self.lspace + self.length * self.char_width + self.rspace
    }

    fn get_lspace(&self) -> u32 {
        self.lspace
    }

    fn set_lspace(&mut self, lspace: u32) {
        self.lspace = lspace
    }

    fn set_rspace(&mut self, rspace: u32) {
        self.rspace = rspace
    }
}
